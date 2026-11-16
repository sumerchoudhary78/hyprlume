use super::super::pairing::Pairing;
use bluer::agent::{Agent, ReqError};
use std::sync::Arc;

pub(super) fn agent(pairing: Arc<Pairing>) -> Agent {
    let pin = pairing.clone();
    let passkey = pairing.clone();
    let confirm = pairing.clone();
    let authorize = pairing.clone();
    let service = pairing.clone();
    let display_pin = pairing.clone();
    Agent {
        request_default: false,
        request_pin_code: Some(Box::new(move |request| {
            let pairing = pin.clone();
            Box::pin(async move {
                let value = pairing
                    .ask(
                        request.adapter,
                        request.device.to_string(),
                        "pin",
                        String::new(),
                    )
                    .await
                    .map_err(|_| ReqError::Canceled)?
                    .ok_or(ReqError::Rejected)?;
                if value.expose().is_empty()
                    || !value.expose().bytes().all(|ch| ch.is_ascii_alphanumeric())
                {
                    return Err(ReqError::Rejected);
                }
                Ok(value.expose().to_owned())
            })
        })),
        request_passkey: Some(Box::new(move |request| {
            let pairing = passkey.clone();
            Box::pin(async move {
                let value = pairing
                    .ask(
                        request.adapter,
                        request.device.to_string(),
                        "passkey",
                        String::new(),
                    )
                    .await
                    .map_err(|_| ReqError::Canceled)?
                    .ok_or(ReqError::Rejected)?;
                value
                    .expose()
                    .parse::<u32>()
                    .ok()
                    .filter(|key| *key <= 999_999)
                    .ok_or(ReqError::Rejected)
            })
        })),
        request_confirmation: Some(Box::new(move |request| {
            let pairing = confirm.clone();
            Box::pin(async move {
                pairing
                    .ask(
                        request.adapter,
                        request.device.to_string(),
                        "confirm",
                        format!("{:06}", request.passkey),
                    )
                    .await
                    .map(|_| ())
                    .map_err(|_| ReqError::Rejected)
            })
        })),
        request_authorization: Some(Box::new(move |request| {
            let pairing = authorize.clone();
            Box::pin(async move {
                pairing
                    .ask(
                        request.adapter,
                        request.device.to_string(),
                        "authorize",
                        String::new(),
                    )
                    .await
                    .map(|_| ())
                    .map_err(|_| ReqError::Rejected)
            })
        })),
        authorize_service: Some(Box::new(move |request| {
            let pairing = service.clone();
            Box::pin(async move {
                pairing
                    .ask(
                        request.adapter,
                        request.device.to_string(),
                        "authorize",
                        String::new(),
                    )
                    .await
                    .map(|_| ())
                    .map_err(|_| ReqError::Rejected)
            })
        })),
        display_pin_code: Some(Box::new(move |request| {
            let pairing = display_pin.clone();
            Box::pin(async move {
                tokio::select! {
                    _ = request.cancel => Ok(()),
                    result = pairing.ask(request.adapter, request.device.to_string(), "display", request.pincode) => result.map(|_| ()).map_err(|_| ReqError::Canceled),
                }
            })
        })),
        display_passkey: Some(Box::new(move |request| {
            let pairing = pairing.clone();
            Box::pin(async move {
                tokio::select! {
                    _ = request.cancel => Ok(()),
                    result = pairing.ask(request.adapter, request.device.to_string(), "display", format!("{:06}", request.passkey)) => result.map(|_| ()).map_err(|_| ReqError::Canceled),
                }
            })
        })),
        ..Default::default()
    }
}
