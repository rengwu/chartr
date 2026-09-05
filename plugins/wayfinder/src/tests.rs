use super::*;
use gpui::TestAppContext;
use std::{
    cell::{Cell, RefCell},
    fs,
    rc::Rc,
};
use zeddy_plugin::{
    PreparedTerminal, TerminalLauncher,
    services::{PluginSettings, ServiceExport, Services, Skill},
};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".plan/maps/design");
    fs::create_dir_all(dir.join("tickets")).unwrap();
    fs::write(dir.join("map.md"),"# A web map\n\n## Destination\nMake the workflow visible.\n\n## Notes\nUse source methods.\n\n## Decisions so far\n\n## Not yet specified\n- **Deployment.** Decide later. <clears-with: 02>\n\n## Out of scope\n").unwrap();
    for (number, blockers) in [(1, "[]"), (2, "[01]")] {
        fs::write(dir.join(format!("tickets/{number:02}-question.md")),format!("---\ntype: research\nblocked_by: {blockers}\n---\n# Question {number}\n\n## Question\nWhat does the source say?\n\n## Done when\nFindings recorded.\n")).unwrap();
    }
    root
}
fn context(root: &Path, send: impl Fn(&[u8]) -> Result<(), String> + 'static) -> InstanceContext {
    let services = Services::default();
    services.publish(
        AGENT_SERVICE,
        vec![ServiceExport::new(Agents::new(
            |_| Ok(vec!["Test agent".into()]),
            |name, prompt, _| {
                if name != "Test agent" {
                    return Err("Unknown agent".into());
                }
                Ok(prompt.as_bytes().to_vec())
            },
        ))],
    );
    services.publish(
        SKILLS_SERVICE,
        vec![ServiceExport::new(Skills::new(|_| {
            gpui::Task::ready(Ok(SkillCatalog {
                skills: vec![Skill {
                    source: "chartr-skills".into(),
                    name: "wayfinder".into(),
                    directory: "/test/skills/wayfinder".into(),
                    commit: "abc123".into(),
                    body: "Resolve one ticket at a time.".into(),
                    shadowed: false,
                }],
                warnings: Vec::new(),
            }))
        }))],
    );
    let send = Rc::new(send);
    InstanceContext {
        instance_id: 1,
        space: "fixture".into(),
        space_name: "Fixture".into(),
        project_dir: Some(root.to_owned()),
        bound_session: None,
        terminal: TerminalLauncher::new(|_, _| panic!("Must prepare first")).with_prepare(
            move |_| {
                let send = send.clone();
                gpui::Task::ready(Ok(PreparedTerminal::new("test-session".into(), move |input| {
                    send(input)
                })))
            },
        ),
        services,
        plugin_settings: PluginSettings::new(|_, _, _| {}),
    }
}
fn window(cx: &mut TestAppContext) -> AnyWindowHandle {
    cx.add_empty_window();
    cx.windows()[0]
}
fn selection() -> Selection {
    Selection { slug: "design".into(), ticket: 1, method: None, note: String::new() }
}
async fn preview(bridge: &mut Bridge, window: AnyWindowHandle, cx: &mut AsyncApp) -> u64 {
    bridge
        .handle(Action::Preview { selection: selection() }, "document", window, || true, cx)
        .await
        .unwrap()["preview"]
        .as_u64()
        .unwrap()
}

#[gpui::test]
async fn web_launcher_claims_before_input_and_missing_providers_keep_maps(cx: &mut TestAppContext) {
    let root = fixture();
    let path = root.path().join(".plan/maps/design/tickets/01-question.md");
    let received = Rc::new(RefCell::new(Vec::new()));
    let saved = received.clone();
    let context = context(root.path(), move |input| {
        assert!(fs::read_to_string(&path).unwrap().contains("claimed_by: \"test-session\""));
        saved.borrow_mut().extend_from_slice(input);
        Ok(())
    });
    let services = context.services.clone();
    let mut bridge = Bridge::new(context);
    let window = window(cx);
    let cx = &mut cx.to_async();
    let id = preview(&mut bridge, window, cx).await;
    let result = bridge
        .handle(
            Action::Launch { preview: id, agent: "Test agent".into() },
            "document",
            window,
            || true,
            cx,
        )
        .await
        .unwrap();
    assert_eq!(result["session"], "test-session");
    assert!(!received.borrow().is_empty());
    services.remove(AGENT_SERVICE);
    services.remove(SKILLS_SERVICE);
    let snapshot = bridge.handle(Action::Snapshot, "document", window, || true, cx).await.unwrap();
    assert_eq!(snapshot["maps"].as_array().unwrap().len(), 1);
    assert!(snapshot["agent_error"].is_string());
    assert!(snapshot["skill_error"].is_string());
}

#[gpui::test]
async fn failed_delivery_releases_only_its_own_claim(cx: &mut TestAppContext) {
    let root = fixture();
    let mut bridge = Bridge::new(context(root.path(), |_| Err("Input channel closed".into())));
    let window = window(cx);
    let cx = &mut cx.to_async();
    let id = preview(&mut bridge, window, cx).await;
    assert_eq!(
        bridge
            .handle(
                Action::Launch { preview: id, agent: "Test agent".into() },
                "document",
                window,
                || true,
                cx
            )
            .await
            .unwrap_err(),
        "Input channel closed"
    );
    assert_eq!(model::discover(root.path()).unwrap()[0].tickets[0].status, model::Status::Open);
}

#[gpui::test]
async fn changed_sources_or_files_reject_stale_web_previews(cx: &mut TestAppContext) {
    let root = fixture();
    let context = context(root.path(), |_| panic!("Stale input must not launch"));
    let services = context.services.clone();
    let mut bridge = Bridge::new(context);
    let window = window(cx);
    let cx = &mut cx.to_async();
    let id = preview(&mut bridge, window, cx).await;
    let file = root.path().join(".plan/maps/design/map.md");
    let source = fs::read_to_string(&file).unwrap();
    fs::write(&file, format!("{source}\nAn edit after preview.\n")).unwrap();
    assert!(
        bridge
            .handle(
                Action::Launch { preview: id, agent: "Test agent".into() },
                "document",
                window,
                || true,
                cx
            )
            .await
            .unwrap_err()
            .contains("changed after preview")
    );
    let id = preview(&mut bridge, window, cx).await;
    services.remove(SKILLS_SERVICE);
    assert!(
        bridge
            .handle(
                Action::Launch { preview: id, agent: "Test agent".into() },
                "document",
                window,
                || true,
                cx
            )
            .await
            .is_err()
    );
    assert_eq!(model::discover(root.path()).unwrap()[0].tickets[0].status, model::Status::Open);
}

#[gpui::test]
async fn source_revocation_or_closing_during_prepare_never_delivers(cx: &mut TestAppContext) {
    let window = window(cx);
    let cx = &mut cx.to_async();
    for revoke in [true, false] {
        let root = fixture();
        let mut context = context(root.path(), |_| panic!("No input after revocation"));
        let services = context.services.clone();
        let alive = Rc::new(Cell::new(true));
        let closing = alive.clone();
        context.terminal = TerminalLauncher::new(|_, _| {}).with_prepare(move |_| {
            if revoke {
                services.remove(SKILLS_SERVICE);
            } else {
                closing.set(false);
            }
            gpui::Task::ready(Ok(PreparedTerminal::new("preparing-session".into(), |_| {
                panic!("No input")
            })))
        });
        let mut bridge = Bridge::new(context);
        let id = preview(&mut bridge, window, cx).await;
        assert!(
            bridge
                .handle(
                    Action::Launch { preview: id, agent: "Test agent".into() },
                    "document",
                    window,
                    || alive.get(),
                    cx
                )
                .await
                .is_err()
        );
        assert_eq!(model::discover(root.path()).unwrap()[0].tickets[0].status, model::Status::Open);
    }
}

#[gpui::test]
async fn preview_tokens_are_document_bound_one_shot_and_blocked_tickets_cannot_preview(
    cx: &mut TestAppContext,
) {
    let root = fixture();
    let mut bridge = Bridge::new(context(root.path(), |_| panic!("No input")));
    let window = window(cx);
    let cx = &mut cx.to_async();
    let id = preview(&mut bridge, window, cx).await;
    for document in ["another-document", "document"] {
        assert!(
            bridge
                .handle(
                    Action::Launch { preview: id, agent: "Test agent".into() },
                    document,
                    window,
                    || true,
                    cx
                )
                .await
                .is_err()
        );
    }
    let mut selection = selection();
    selection.ticket = 2;
    assert!(
        bridge
            .handle(Action::Preview { selection }, "document", window, || true, cx)
            .await
            .unwrap_err()
            .contains("not ready")
    );
}

#[test]
fn missing_and_ruled_out_blockers_cycles_and_duplicate_ids_never_become_frontier() {
    let root = fixture();
    let dir = root.path().join(".plan/maps/design/tickets");
    let first = dir.join("01-question.md");
    let source = fs::read_to_string(&first).unwrap();
    fs::write(&first, format!("{source}\n## Ruled out\nOutside the destination.\n")).unwrap();
    assert!(!model::discover(root.path()).unwrap()[0].ticket(2).unwrap().frontier);
    fs::write(&first, source.replace("blocked_by: []", "blocked_by: [02]")).unwrap();
    assert_eq!(model::discover(root.path()).unwrap()[0].frontier(), 0);
    fs::write(&first, source.clone()).unwrap();
    fs::write(dir.join("01-duplicate.md"), source).unwrap();
    let maps = model::discover(root.path()).unwrap();
    assert_eq!(maps[0].frontier(), 0);
    assert!(maps[0].warnings.iter().any(|w| w.contains("Duplicate")));
}

#[test]
fn markdown_never_executes_project_content_or_fetches_images() {
    let html = markdown(
        "# Hello\n\n<script>window.chartr.invoke('wayfinder.launch')</script>\n\n[bad](javascript:alert%281%29) ![alt](https://example.com/image.png)\n\n**Bold** and `code`.\n",
    );
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<strong>Bold</strong>"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("<img"));
}

#[test]
fn links_can_open_only_http_or_files_inside_this_space() {
    let root = fixture();
    let base = root.path().join(".plan/maps/design/map.md");
    assert!(
        open_target(root.path(), &base, Some("tickets/01-question.md"))
            .unwrap()
            .starts_with("file:")
    );
    assert_eq!(
        open_target(root.path(), &base, Some("https://example.com/path")).unwrap(),
        "https://example.com/path"
    );
    for target in [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "//example.com",
        "../../../../../../etc/passwd",
    ] {
        assert!(open_target(root.path(), &base, Some(target)).is_err());
    }
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), base.parent().unwrap().join("escape")).unwrap();
    assert!(open_target(root.path(), &base, Some("escape")).is_err());
}

#[test]
fn the_web_bridge_requires_an_explicit_grant_and_an_owning_pane() {
    use zeddy_plugin::manifest::Permissions;
    let root = fixture();
    let context = context(root.path(), |_| Ok(()));
    assert!(Bridge::for_web(Some(context.clone()), &Permissions::default()).is_none());
    let grant = Permissions { wayfinder: true, ..Permissions::default() };
    assert!(Bridge::for_web(None, &grant).is_none());
    assert!(Bridge::for_web(Some(context), &grant).is_some());
    assert!(grant.summary().contains("registered-agent launching"));
}

#[test]
fn preview_requires_an_existing_map_and_ticket_selection() {
    for options in [
        json!({"action": "wayfinder.preview"}),
        json!({"action": "wayfinder.preview", "slug": null, "ticket": null}),
        json!({"action": "wayfinder.preview", "slug": "design"}),
        json!({"action": "wayfinder.preview", "ticket": 1}),
    ] {
        assert!(serde_json::from_value::<Action>(options).is_err());
    }
    assert!(
        serde_json::from_value::<Action>(json!({
            "action": "wayfinder.preview", "slug": "design", "ticket": 1
        }))
        .is_ok()
    );
}
