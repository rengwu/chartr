//! Real Codex/Claude TUIs with local deterministic model responses. Opt in with:
//! cargo test -p chartr-conversations --test live_terminal_input -- --ignored --nocapture
use chartr_conversations::{
    NativeSession, Observation, Provider, ProviderPaths, Role, Status, Store,
};
use chartr_herdr::{
    Namespace, Sidecar,
    control::{AgentInputTarget, Client},
};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Cleanup {
    children: Vec<Child>,
    client: Client,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = self.client.stop_daemon();
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn api(client: &Client, method: &str, params: Value) -> Value {
    let mut socket = UnixStream::connect(client.namespace().socket()).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    writeln!(socket, "{}", json!({"id":"fixture","method":method,"params":params})).unwrap();
    let mut line = String::new();
    BufReader::new(socket).read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}

fn screen(client: &Client, pane: &str) -> String {
    api(client, "pane.read", json!({"pane_id":pane,"source":"detection","lines":80}))["read"]["text"]
        .as_str().unwrap().to_owned()
}

#[test]
#[ignore = "requires installed Codex and Claude CLIs; uses isolated configuration and a local model"]
fn codex_and_claude_submit_to_the_original_tui_and_preserve_drafts() {
    // macOS Unix sockets have a short path limit; keep the private namespace short.
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let root_path = temp.path().canonicalize().unwrap();
    let root = root_path.as_path();
    let script = root.join("model.py");
    std::fs::write(&script, include_str!("fixture_model.py")).unwrap();
    let mut model = Command::new("python3")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut port = String::new();
    BufReader::new(model.stdout.take().unwrap()).read_line(&mut port).unwrap();
    let codex_home = root.join("codex");
    let claude_home = root.join("claude");
    std::fs::create_dir_all(&codex_home).unwrap();
    std::fs::create_dir_all(&claude_home).unwrap();
    std::fs::write(
        codex_home.join("config.toml"),
        format!(
            r#"
model = "fixture"
model_provider = "fixture"
model_context_window = 32000
model_max_output_tokens = 1024
[model_providers.fixture]
name = "Local fixture"
base_url = "http://127.0.0.1:{}/v1"
wire_api = "responses"
requires_openai_auth = false
[projects.{}]
trust_level = "trusted"
"#,
            port.trim(),
            serde_json::to_string(root.to_str().unwrap()).unwrap()
        ),
    )
    .unwrap();
    std::fs::write(
        claude_home.join(".claude.json"),
        json!({"hasCompletedOnboarding":true,"theme":"dark","hasAcknowledgedCostThreshold":true,
            "projects":{root.to_str().unwrap():{"hasTrustDialogAccepted":true}}})
        .to_string(),
    )
    .unwrap();
    let namespace = Namespace::rooted(root.join("config/chartr/herdr"));
    namespace.prepare().unwrap();
    let sidecar = Sidecar::at(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/herdr")
            .canonicalize()
            .unwrap(),
    )
    .unwrap();
    let client = Client::new(sidecar.clone(), namespace.clone());
    let mut command = Command::new(sidecar.path());
    command.arg("server").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    for (key, value) in namespace.env() {
        match value {
            Some(value) => {
                command.env(key, value);
            }
            None => {
                command.env_remove(key);
            }
        }
    }
    // These are child-only provider configuration roots, with no user credentials.
    command
        .env("CODEX_HOME", &codex_home)
        .env_remove("CODEX_THREAD_ID")
        .env("CLAUDE_CONFIG_DIR", &claude_home)
        .env("ANTHROPIC_API_KEY", "local-fixture")
        .env("ANTHROPIC_BASE_URL", format!("http://127.0.0.1:{}", port.trim()))
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1");
    let daemon = command.spawn().unwrap();
    let _cleanup = Cleanup { children: vec![model, daemon], client: client.clone() };
    client.reconnect(Duration::from_secs(10)).unwrap();
    let workspace = client.open_workspace(root, Some("Input proof")).unwrap();
    client.install_agent_integration("codex").unwrap();
    let paths =
        ProviderPaths { codex: codex_home, claude: claude_home, opencode: root.join("unused") };
    let mut store = Store::open(&root.join("history.sqlite"), paths).unwrap();
    for provider in [Provider::Codex, Provider::Claude] {
        let session = client.start_session(&workspace, None).unwrap();
        let claude_id = "c598a087-7626-4d6d-b090-a1e427609040";
        let command = match provider {
            Provider::Codex => {
                "codex --dangerously-bypass-hook-trust 'Initial terminal message'".to_owned()
            }
            _ => format!(
                "claude --bare --session-id {claude_id} --model claude-sonnet-4-6 --tools ''"
            ),
        };
        api(
            &client,
            "pane.send_text",
            json!({"pane_id":session.id.0,"text":format!("{command}\n")}),
        );
        let deadline = Instant::now() + Duration::from_secs(25);
        let mut accepted_fixture_key = false;
        let target = loop {
            if provider == Provider::Claude && !accepted_fixture_key {
                let visible = screen(&client, &session.id.0);
                if visible.contains("Do you want to use this API key?")
                    && visible.contains("local-fixture")
                {
                    api(
                        &client,
                        "pane.send_keys",
                        json!({"pane_id":session.id.0,"keys":["Up","Enter"]}),
                    );
                    accepted_fixture_key = true;
                }
            }
            let info =
                client.sessions(None).unwrap().into_iter().find(|s| s.id == session.id).unwrap();
            let mut readiness = format!("{info:?}");
            if provider == Provider::Claude
                && info.agent.as_deref() == Some("claude")
                && info.agent_session.is_none()
            {
                // --bare intentionally skips hooks; the test launched this exact
                // native id explicitly. Production uses the SessionStart hook.
                api(
                    &client,
                    "pane.report_agent_session",
                    json!({"pane_id":session.id.0,"source":"herdr:claude","agent":"claude","agent_session_id":claude_id,"session_start_source":"startup"}),
                );
            }
            if let (Some(native), Some(pid)) = (info.agent_session, info.foreground_pid) {
                let target = AgentInputTarget {
                    pane: session.id.clone(),
                    terminal: session.terminal.clone(),
                    provider: provider.slug().into(),
                    native_id: native.value,
                    pid,
                };
                match client.check_agent_input(&target) {
                    Ok(()) => break target,
                    Err(error) => readiness = format!("{target:?}: {error}"),
                }
            }
            assert!(
                Instant::now() < deadline,
                "{provider:?} not ready ({readiness}): {}",
                api(
                    &client,
                    "pane.read",
                    json!({"pane_id":session.id.0,"source":"visible","format":"ansi","strip_ansi":false,"lines":40})
                )
            );
            std::thread::sleep(Duration::from_millis(150));
        };
        let observe = || Observation {
            space: None,
            runtime: target.pane.0.clone(),
            terminal: target.terminal.0.clone(),
            provider,
            native: Some(NativeSession { id: target.native_id.clone(), path: None }),
            cwd: Some(root.into()),
            title: None,
            status: Status::Idle,
            pid: Some(target.pid),
        };
        store.reconcile(vec![observe()], 1).unwrap();
        let id = store.for_runtime(&target.pane.0).unwrap().to_owned();
        let text = "First line from rich chat\nSecond line with Unicode: café λ";
        store.begin_terminal_delivery(&id, "fixture-delivery".into(), text.into()).unwrap();
        client.send_agent_input(&target, text).unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            store.reconcile(vec![observe()], 2).unwrap();
            let row = store.get(&id).unwrap();
            if row.delivery.is_none()
                && row
                    .messages
                    .iter()
                    .any(|m| m.role == Role::Assistant && m.text.contains("Same terminal."))
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "{provider:?} no confirmed response: {:?}; screen: {}",
                row,
                screen(&client, &session.id.0)
            );
            std::thread::sleep(Duration::from_millis(150));
        }
        assert_eq!(
            store
                .get(&id)
                .unwrap()
                .messages
                .iter()
                .filter(|m| m.role == Role::User && m.text == text)
                .count(),
            1
        );
        assert_eq!(
            client.foreground_pid(&target.pane).unwrap(),
            Some(target.pid),
            "processes: {}",
            api(&client, "pane.process_info", json!({"pane_id":target.pane.0}))
        );
        assert!(screen(&client, &session.id.0).contains("Same terminal."));
        let deadline = Instant::now() + Duration::from_secs(10);
        while client.check_agent_input(&target).is_err() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(100));
        }
        api(
            &client,
            "pane.send_text",
            json!({"pane_id":session.id.0,"text":"keep my terminal draft"}),
        );
        std::thread::sleep(Duration::from_millis(300));
        let error = client.send_agent_input(&target, "must not replace the draft").unwrap_err();
        assert!(!error.uncertain);
        assert!(screen(&client, &session.id.0).contains("keep my terminal draft"));
        let mut stale = target.clone();
        stale.native_id = "another-session".into();
        assert!(client.send_agent_input(&stale, "wrong conversation").is_err());
        println!(
            "PASS {provider:?}: same native id/PID, multiline Unicode, transcript confirmation, response in TUI, draft preserved, stale identity rejected"
        );
    }
}
