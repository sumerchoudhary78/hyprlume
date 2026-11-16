use hyprlume::{config::Config, planner, protocol::Operation, server::App};
use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

async fn endpoint(content: &str) -> (String, tokio::task::JoinHandle<serde_json::Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://{}/v1/chat/completions",
        listener.local_addr().unwrap()
    );
    let body = json!({"choices":[{"message":{"content":content}}]}).to_string();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut input = Vec::new();
        let (end, length) = loop {
            let mut buffer = [0; 2048];
            let n = stream.read(&mut buffer).await.unwrap();
            assert!(n > 0);
            input.extend_from_slice(&buffer[..n]);
            if let Some(end) = input.windows(4).position(|s| s == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&input[..end]).to_ascii_lowercase();
                let length: usize = headers
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                break (end + 4, length);
            }
        };
        while input.len() < end + length {
            let mut buffer = [0; 2048];
            let n = stream.read(&mut buffer).await.unwrap();
            assert!(n > 0);
            input.extend_from_slice(&buffer[..n]);
        }
        let request = serde_json::from_slice(&input[end..end + length]).unwrap();
        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        request
    });
    (url, task)
}

#[tokio::test]
async fn model_plans_need_confirmation_even_when_execution_is_requested() {
    let (url, received) = endpoint(r#"{"actions":[{"type":"workspace","id":2}]}"#).await;
    let mut config = Config::default();
    config.llm.endpoint = Some(url);
    let app = App::new(config, true);
    app.handle(Operation::Command {
        text: "put me on my second workspace".into(),
        dry_run: false,
    })
    .await
    .unwrap();
    let request = received.await.unwrap();
    assert_eq!(request["response_format"]["type"], "json_schema");
    assert_eq!(
        request["messages"][1]["content"],
        "put me on my second workspace"
    );
    assert_eq!(request["temperature"], 0);
    assert_eq!(
        request["response_format"]["json_schema"]["schema"]["properties"]["actions"]["maxItems"],
        4
    );
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while app.state().phase != "review" {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(app.state().confirmation.is_some());
    assert!(app.state().receipts.is_empty());
}

#[tokio::test]
async fn an_untrusted_model_cannot_invent_executables() {
    let (url, received) = endpoint(r#"{"actions":[{"type":"launch","app":"sh -c evil"}]}"#).await;
    let mut config = Config::default();
    config.llm.endpoint = Some(url);
    assert!(planner::plan("do a task", &config).await.is_err());
    received.await.unwrap();
}

#[tokio::test]
async fn an_oversized_model_response_is_rejected() {
    let (url, received) = endpoint(&"x".repeat(40000)).await;
    let mut config = Config::default();
    config.llm.endpoint = Some(url);
    assert!(planner::plan("do a task", &config)
        .await
        .unwrap_err()
        .to_string()
        .contains("32 KiB"));
    received.await.unwrap();
}
