use super::{
    authority,
    login::{self, Login},
    pam::{Attempt, Event},
    wire::{self, Input, Output},
};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use polkit_agent_rs::polkit::UnixUser;
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
use zbus::{
    fdo::{self, DBusProxy},
    names::OwnedUniqueName,
    zvariant::Value,
    Connection, Proxy,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_ATTEMPTS: u8 = 3;

pub(super) async fn run() -> Result<()> {
    let mut input = wire::receive()?;
    let connection = Connection::system()
        .await
        .context("Connect to the system bus")?;
    let bus = DBusProxy::new(&connection).await?;
    let mut authorities = bus
        .receive_name_owner_changed_with_args(&[(0, authority::SERVICE)])
        .await?;
    let mut logins = bus
        .receive_name_owner_changed_with_args(&[(0, login::SERVICE)])
        .await?;
    let login_owner = bus.get_name_owner(login::SERVICE.try_into()?).await?;
    let login = tokio::time::timeout(
        Duration::from_secs(5),
        Login::connect(&connection, login_owner.clone()),
    )
    .await??;
    let owner = authority_owner(&bus).await?;
    let (sender, requests) = mpsc::channel(1);
    connection
        .object_server()
        .at(
            authority::AGENT_PATH,
            authority::Agent::new(owner.clone(), sender),
        )
        .await?;
    let proxy = Proxy::new(
        &connection,
        owner.clone(),
        authority::PATH,
        authority::INTERFACE,
    )
    .await?;
    let subject = (
        "unix-session",
        HashMap::from([("session-id", Value::from(login.id.as_str()))]),
    );
    tokio::time::timeout(
        Duration::from_secs(5),
        proxy.call::<_, _, ()>(
            "RegisterAuthenticationAgent",
            &(&subject, "", authority::AGENT_PATH),
        ),
    )
    .await?
    .context("Register with Polkit (check whether another agent owns this login)")?;
    wire::send(Output::Registered)?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let result = tokio::select! {
        result = serve(&login, requests, &mut input) => result,
        _ = terminate.recv() => Ok(()),
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = async {
            loop {
                let (event, expected) = tokio::select! {
                    event = authorities.next() => (event, &owner),
                    event = logins.next() => (event, &login_owner),
                };
                let event = event.context("The system bus disconnected")?;
                if event.args()?.old_owner().as_ref().is_some_and(|name| name.as_str() == expected.as_str()) {
                    anyhow::bail!("The system authority or login manager disconnected");
                }
            }
        } => result,
    };
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        proxy.call::<_, _, ()>(
            "UnregisterAuthenticationAgent",
            &(&subject, authority::AGENT_PATH),
        ),
    )
    .await;
    result
}

async fn authority_owner(bus: &DBusProxy<'_>) -> Result<OwnedUniqueName> {
    let name = authority::SERVICE.try_into()?;
    match bus.get_name_owner(name).await {
        Ok(owner) => Ok(owner),
        Err(fdo::Error::NameHasNoOwner(_)) => {
            bus.start_service_by_name(authority::SERVICE.try_into()?, 0)
                .await?;
            Ok(bus.get_name_owner(authority::SERVICE.try_into()?).await?)
        }
        Err(error) => Err(error.into()),
    }
}

async fn serve(
    login: &Login<'_>,
    mut requests: mpsc::Receiver<authority::Challenge>,
    input: &mut mpsc::Receiver<Input>,
) -> Result<()> {
    let mut available = false;
    let mut id = 0u32;
    loop {
        tokio::select! {
            message = input.recv() => match message {
                Some(Input::Available {value}) => available = value,
                Some(_) => (),
                None => return Ok(()),
            },
            request = requests.recv() => {
                let Some(mut request) = request else { return Ok(()); };
                id = id.checked_add(1).context("Authentication request limit reached")?;
                let outcome = if available && login.available().await && !*request.cancelled.borrow() {
                    let result = tokio::time::timeout(REQUEST_TIMEOUT, converse(login, &mut request, id, input, &mut available)).await;
                    wire::send(Output::Close {id})?;
                    result.unwrap_or(Ok(false))
                } else { Ok(false) };
                let reply = if matches!(outcome, Ok(true)) { Ok(()) } else {
                    Err(authority::AgentError::Cancelled("Authentication was cancelled or failed".into()))
                };
                let _ = request.reply.send(reply);
                outcome?;
            }
        }
    }
}

async fn converse(
    login: &Login<'_>,
    request: &mut authority::Challenge,
    id: u32,
    input: &mut mpsc::Receiver<Input>,
    available: &mut bool,
) -> Result<bool> {
    let accounts: Vec<_> = request
        .users
        .iter()
        .map(|uid| {
            let user = UnixUser::new(*uid);
            user.name()
                .map(|name| wire::label(&name, 80))
                .unwrap_or_else(|| format!("UID {uid}"))
        })
        .collect();
    let mut changes = login.properties.receive_signal("PropertiesChanged").await?;
    if !login.available().await || *request.cancelled.borrow_and_update() {
        return Ok(false);
    }
    wire::send(Output::Open {
        id,
        action: &request.action,
        message: &request.message,
        accounts: &accounts,
    })?;
    let mut attempt: Option<Attempt> = None;
    let mut attempts = 0;
    let mut prompt = 0u32;
    let mut awaiting = false;
    loop {
        tokio::select! {
            biased;
            _ = request.cancelled.changed() => return Ok(false),
            signal = changes.next() => {
                if signal.is_none() || !login.available().await { return Ok(false); }
            }
            message = input.recv() => match message {
                None => return Ok(false),
                Some(Input::Available {value}) => {
                    *available = value;
                    if !value { return Ok(false); }
                }
                Some(Input::Cancel {id: target}) if target == id => return Ok(false),
                Some(Input::Start {id: target, account}) if target == id && attempt.is_none() => {
                    let Some(&uid) = request.users.get(account) else { continue; };
                    if !login.available().await || *request.cancelled.borrow() { return Ok(false); }
                    attempts += 1;
                    attempt = Some(Attempt::start(uid, request.cookie.clone())?);
                }
                Some(Input::Respond {id: target, prompt: step, response})
                    if target == id && step == prompt && awaiting && response.value().is_some() => {
                    if !login.available().await || *request.cancelled.borrow() { return Ok(false); }
                    awaiting = false;
                    if let Some(attempt) = &attempt {
                        attempt.responses.try_send(response).map_err(|_| anyhow::anyhow!("The authentication helper stopped"))?;
                    }
                }
                Some(_) => (),
            },
            event = async {
                match &mut attempt {
                    Some(attempt) => attempt.events.recv().await,
                    None => std::future::pending().await,
                }
            } => match event {
                Some(Event::Prompt(label, echo)) => {
                    prompt = prompt.checked_add(1).context("PAM prompt limit reached")?;
                    awaiting = true;
                    wire::send(Output::Prompt {id, prompt, label: &label, echo})?;
                }
                Some(Event::Notice(message, error)) => wire::send(Output::Notice {id, message: &message, error})?,
                Some(Event::Finished(true)) => {
                    return Ok(login.available().await && !*request.cancelled.borrow());
                }
                Some(Event::Finished(false)) if attempts < MAX_ATTEMPTS => {
                    attempt = None;
                    awaiting = false;
                    wire::send(Output::Retry {id})?;
                }
                _ => return Ok(false),
            }
        }
    }
}
