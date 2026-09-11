//! Opt-in same-runtime proof; requires installed OpenCode and target/debug/herdr.
//! Every provider data/config directory and the Herdr daemon are isolated.
//! cargo test -p chartr-conversations --test live_opencode -- --ignored --nocapture
use chartr_conversations::{
    NativeSession, Observation, OpenCode, Provider, ProviderPaths, Role, Status, Store,
    endpoints_for_process,
};
use chartr_herdr::{Namespace, Sidecar, control::Client};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Children(Vec<Child>);
impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn api(client: &Client, method: &str, params: Value) -> Value {
    let mut stream = UnixStream::connect(client.namespace().socket()).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    writeln!(stream, "{}", json!({"id":"test", "method":method, "params":params})).unwrap();
    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response).unwrap();
    let value: Value = serde_json::from_str(&response).unwrap();
    assert!(value.get("error").is_none_or(Value::is_null), "{value}");
    value["result"].clone()
}

#[test]
#[ignore = "requires installed OpenCode; starts isolated real PTYs and a local model fixture"]
fn two_manual_agents_keep_identity_and_input_in_the_same_runtime() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let model = root.join("fixture.py");
    std::fs::write(&model, include_str!("fixture_model.py")).unwrap();
    let mut responder = Command::new("python3")
        .arg(&model)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut port = String::new();
    BufReader::new(responder.stdout.take().unwrap()).read_line(&mut port).unwrap();
    let mut children = Children(vec![responder]);
    let sidecar = Sidecar::at(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/herdr")
            .canonicalize()
            .unwrap(),
    )
    .unwrap();
    let ns = Namespace::rooted(root.join("config/chartr/herdr"));
    ns.prepare().unwrap();
    let mut config = json!({"model":"fixture/fixture", "small_model":"fixture/fixture", "enabled_providers":["fixture"], "share":"disabled", "autoupdate":false,
        "provider":{"fixture":{"npm":"@ai-sdk/openai-compatible", "name":"Local fixture", "options":{"baseURL":format!("http://127.0.0.1:{}/v1",port.trim()), "apiKey":"fixture"},"models":{"fixture":{"name":"Fixture","limit":{"context":32000,"output":1024}}}}}});
    config["provider"]["fixture"]["models"]["registered"] =
        json!({"name":"Registered fixture","limit":{"context":32000,"output":1024}});
    let mut daemon = Command::new(sidecar.path());
    daemon.arg("server").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    for (key, value) in ns.env() {
        match value {
            Some(value) => daemon.env(key, value),
            None => daemon.env_remove(key),
        };
    }
    for (key, value) in [
        ("XDG_DATA_HOME", root.join("data")),
        ("XDG_STATE_HOME", root.join("state")),
        ("XDG_CACHE_HOME", root.join("cache")),
        ("OPENCODE_CONFIG_DIR", root.join("opencode-config")),
    ] {
        daemon.env(key, value);
    }
    daemon
        .env("OPENCODE_CONFIG_CONTENT", config.to_string())
        .env("OPENCODE_DISABLE_AUTOUPDATE", "1")
        .env("OPENCODE_DISABLE_MODELS_FETCH", "1")
        .env("OPENCODE_DISABLE_PROJECT_CONFIG", "1")
        .env("OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER", "1");
    children.0.push(daemon.spawn().unwrap());
    let client = Client::new(sidecar, ns);
    client.reconnect(Duration::from_secs(10)).unwrap();
    let workspace = client.open_workspace(root, Some("Conversation proof")).unwrap();
    let shell = client.start_session(&workspace, None).unwrap();
    Command::new("git").args(["init", "-q"]).arg(root).status().unwrap();
    let lazygit = client.start_session(&workspace, None).unwrap();
    api(&client, "pane.send_text", json!({"pane_id":lazygit.id.0,"text":"lazygit\n"}));
    let mut sessions = Vec::new();
    for index in 0..2 {
        let session = client.start_session(&workspace, None).unwrap();
        // Typed into an ordinary shell, as a user would launch it manually.
        api(
            &client,
            "pane.send_text",
            json!({"pane_id":session.id.0,"text":"opencode --port 0 --hostname 127.0.0.1 --model fixture/registered --agent plan\n"}),
        );
        let deadline = Instant::now() + Duration::from_secs(40);
        let (pid, adapter) = loop {
            if let Some(pid) = client.foreground_pid(&session.id).unwrap() {
                if let Some(adapter) = endpoints_for_process(pid)
                    .unwrap()
                    .iter()
                    .filter_map(|url| OpenCode::new(url, root).ok())
                    .find(|adapter| adapter.health().is_ok())
                {
                    break (pid, adapter);
                }
            }
            assert!(
                Instant::now() < deadline,
                "OpenCode {index} failed to expose server: {}",
                client.read_scrollback(&session.id, 120).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(150));
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        while !client.read_visible(&session.id).unwrap().contains("tab agents") {
            assert!(Instant::now() < deadline, "TUI did not become ready");
            std::thread::sleep(Duration::from_millis(100));
        }
        let native = adapter.create().unwrap();
        let title = adapter.read(&native).unwrap().0;
        adapter.select(&native).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !client.read_visible(&session.id).unwrap().contains(&title) {
            assert!(Instant::now() < deadline, "TUI did not select {native}");
            std::thread::sleep(Duration::from_millis(100));
        }
        client.report_opencode_session(&session.id, &native).unwrap();
        sessions.push((session, pid, adapter, native));
    }
    assert_ne!(sessions[0].3, sessions[1].3);
    let paths = ProviderPaths {
        codex: root.join("codex"),
        claude: root.join("claude"),
        opencode: root.join("data/opencode"),
    };
    let mut store = Store::open(&root.join("history.sqlite"), paths.clone()).unwrap();
    let observations = || {
        client
            .sessions(None)
            .unwrap()
            .into_iter()
            .filter(|s| s.agent.as_deref() == Some("opencode"))
            .map(|s| Observation {
                space: None,
                runtime: s.id.0,
                terminal: s.terminal.0,
                provider: Provider::OpenCode,
                native: s
                    .agent_session
                    .map(|native| NativeSession { id: native.value, path: None }),
                cwd: s.cwd,
                title: s.conversation_title,
                status: Status::Idle,
                pid: s.foreground_pid,
            })
            .collect()
    };
    store.reconcile(observations(), 1).unwrap();
    assert_eq!(store.list().len(), 2);
    let target = store.for_runtime(&sessions[0].0.id.0).unwrap().to_owned();
    assert!(store.get(&target).unwrap().can_send());
    let (adapter, native) = store.live_client(&target).unwrap();
    api(
        &client,
        "pane.send_text",
        json!({"pane_id":sessions[0].0.id.0,"text":"draft stays in terminal"}),
    );
    std::thread::sleep(Duration::from_millis(250));
    adapter
        .send_with_options(
            &native,
            &OpenCode::message_id(),
            "First line\nSecond line",
            Some("fixture/registered"),
            Some("plan"),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        store.reconcile(observations(), 2).unwrap();
        let row = store.get(&target).unwrap();
        if row.messages.iter().any(|m| m.role == Role::Assistant && m.complete) {
            break;
        }
        assert!(Instant::now() < deadline, "response timed out: {:?}", row);
        std::thread::sleep(Duration::from_millis(150));
    }
    let row = store.get(&target).unwrap();
    assert_eq!(row.messages.iter().filter(|m| m.role == Role::User).count(), 1);
    assert_eq!(
        row.messages.iter().find(|m| m.role == Role::User).unwrap().text,
        "First line\nSecond line"
    );
    assert!(sessions[1].2.read(&sessions[1].3).unwrap().2.is_empty());
    for (session, pid, adapter, _) in &sessions {
        assert!(
            endpoints_for_process(*pid).unwrap().contains(&adapter.endpoint().to_owned()),
            "Original server process ended: {}",
            api(&client, "pane.process_info", json!({"pane_id":session.id.0}))
        );
    }
    assert!(client.sessions(None).unwrap().iter().any(|s| s.id == shell.id));
    let deadline = Instant::now() + Duration::from_secs(10);
    while adapter.ready(&native).is_err() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(100));
    }
    adapter
        .send(&native, &OpenCode::message_id(), "Ask a question with the question tool.")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let question = loop {
        let requests = adapter.pending(&native).unwrap();
        if let Some(question) = requests
            .into_iter()
            .find(|r| matches!(r, chartr_conversations::Request::Question { .. }))
        {
            break question;
        }
        assert!(Instant::now() < deadline, "question did not appear");
        std::thread::sleep(Duration::from_millis(150));
    };
    // The first turn's registered profile must survive a later ordinary send,
    // instead of reverting to the server's default model or build agent.
    let mut url =
        url::Url::parse(&format!("{}/session/{native}/message", adapter.endpoint())).unwrap();
    url.query_pairs_mut().append_pair("directory", &root.canonicalize().unwrap().to_string_lossy());
    let records: Value = serde_json::from_str(
        &ureq::get(url.as_str()).call().unwrap().body_mut().read_to_string().unwrap(),
    )
    .unwrap();
    let users: Vec<_> = records
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["info"]["role"] == "user")
        .collect();
    assert_eq!(users.len(), 2);
    for record in users {
        assert_eq!(record["info"]["agent"], "plan");
        assert_eq!(record["info"]["model"], json!({"providerID":"fixture","modelID":"registered"}));
    }
    assert!(adapter.ready(&native).is_err(), "pending questions disable ordinary send");
    adapter.answer(&native, &question, "Conversation").unwrap();
    assert!(
        adapter.answer(&native, &question, "Terminal").is_err(),
        "an obsolete answer must be rejected"
    );
    std::thread::sleep(Duration::from_millis(500));
    let screen = client.read_scrollback(&sessions[0].0.id, 120).unwrap();
    assert!(
        screen.contains("Same running conversation."),
        "The actual TUI must display the same exchange: {screen}"
    );
    assert!(screen.contains("draft stays in terminal"), "Unsent TUI draft changed: {screen}");
    assert!(
        client.sessions(None).unwrap().iter().any(|s| s.id == lazygit.id
            && s.running.as_deref().is_some_and(|name| name.contains("lazygit"))),
        "lazygit must keep running without a conversation row"
    );
    store.set_draft(&target, "retained draft".into()).unwrap();
    drop(store);
    let mut store = Store::open(&root.join("history.sqlite"), paths).unwrap();
    store.reconcile(observations(), 3).unwrap();
    assert_eq!(store.get(&target).unwrap().draft, "retained draft");
    assert_eq!(store.list().len(), 2);
    println!(
        "PASS: two real OpenCode PTYs, exact distinct IDs, multiline send, streamed answer, no cross-routing, unchanged PIDs, shell retained, real question answered, stale answer rejected, history/draft reconnect"
    );
    client.stop_daemon().unwrap();
}
