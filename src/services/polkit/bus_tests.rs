use super::authority::{Agent, AgentError, AGENT_PATH};
use crate::services::test_bus::Bus;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, time::timeout};
use zbus::{connection::Builder, zvariant::Value, Proxy};

const INTERFACE: &str = "org.freedesktop.PolicyKit1.AuthenticationAgent";

async fn begin(proxy: &Proxy<'_>, cookie: &str) -> zbus::Result<()> {
    proxy
        .call(
            "BeginAuthentication",
            &(
                "org.hyprlume.qa.authentication",
                "Fixture <b>plain text</b>",
                "",
                HashMap::<String, String>::new(),
                cookie,
                vec![("unix-user", HashMap::from([("uid", Value::from(1000u32))]))],
            ),
        )
        .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn challenges_are_bound_to_the_authority_and_cancel_only_the_matching_request() {
    let mut stage = "private bus startup";
    let result = timeout(Duration::from_secs(12), async {
        let bus = Bus::new().await;
        stage = "D-Bus connections";
        let authority = bus.connect().await;
        let other = bus.connect().await;
        let (sender, mut requests) = mpsc::channel(1);
        let worker = Builder::address(bus.address.as_str())
            .unwrap()
            .serve_at(
                AGENT_PATH,
                Agent::new(authority.unique_name().unwrap().to_owned(), sender),
            )
            .unwrap()
            .build()
            .await
            .unwrap();
        let destination = worker.unique_name().unwrap().as_str().to_owned();
        let trusted = Proxy::new(&authority, destination.clone(), AGENT_PATH, INTERFACE)
            .await
            .unwrap();
        let untrusted = Proxy::new(&other, destination, AGENT_PATH, INTERFACE)
            .await
            .unwrap();
        stage = "untrusted sender rejection";
        assert!(begin(&untrusted, "forged")
            .await
            .unwrap_err()
            .to_string()
            .contains("NotAuthorized"));
        assert!(requests.try_recv().is_err());
        stage = "trusted challenge admission";
        let call = trusted.clone();
        let mut first = tokio::spawn(async move { begin(&call, "fixture-cookie").await });
        let mut request = tokio::select! {
            result = &mut first => panic!("Authority challenge returned before it was admitted: {result:?}"),
            request = requests.recv() => request.unwrap(),
        };
        assert_eq!(request.users, [1000]);
        assert_eq!(request.message, "Fixture <b>plain text</b>");
        assert!(timeout(Duration::from_secs(3), begin(&trusted, "second"))
            .await.expect("An overlapping challenge blocked the authority")
            .unwrap_err()
            .to_string()
            .contains("Another authentication request"));
        stage = "CancelAuthentication calls";
        assert!(untrusted
            .call::<_, _, ()>("CancelAuthentication", &("fixture-cookie",))
            .await
            .is_err());
        trusted
            .call::<_, _, ()>("CancelAuthentication", &("old-cookie",))
            .await
            .unwrap();
        assert!(!*request.cancelled.borrow());
        trusted
            .call::<_, _, ()>("CancelAuthentication", &("fixture-cookie",))
            .await
            .unwrap();
        timeout(Duration::from_secs(3), request.cancelled.changed())
            .await.expect("Cancellation did not reach the active challenge").unwrap();
        assert!(*request.cancelled.borrow());
        request
            .reply
            .send(Err(AgentError::Cancelled("Fixture cancelled".into())))
            .unwrap();
        stage = "authority reply";
        assert!(first
            .await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("Cancelled"));
    })
    .await;
    result.unwrap_or_else(|_| panic!("Timed out during {stage}"));
}
