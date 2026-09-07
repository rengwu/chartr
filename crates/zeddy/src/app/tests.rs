use super::bundled_plugins::materialize_bundled_agent;
use super::{
    ErrorNoticeKey, ErrorSeverity, PersistedSpaceKind, Registry, Snapshot, SplitDirection,
    cleanup_empty_implicit_root, pane_drop_direction_for_position, reconcile_error_notices,
    regex_escape_literal, relative_error_time, resolve_terminal_path, split_direction_for_position,
};
use crate::{persistence::PersistedSpace, workspace::WorkspaceTabs};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[test]
fn terminal_search_treats_user_text_as_a_literal() {
    assert_eq!(regex_escape_literal("a.b[c]+(d)?\\e"), "a\\.b\\[c\\]\\+\\(d\\)\\?\\\\e");
}

#[test]
fn error_times_stay_compact_as_they_age() {
    let first_seen = Instant::now();
    assert_eq!(relative_error_time(first_seen, first_seen), "now");
    assert_eq!(relative_error_time(first_seen, first_seen + Duration::from_secs(90)), "1m ago");
    assert_eq!(relative_error_time(first_seen, first_seen + Duration::from_secs(7_200)), "2h ago");
    assert_eq!(
        relative_error_time(first_seen, first_seen + Duration::from_secs(172_800)),
        "2d ago"
    );
}

#[test]
fn dismissed_errors_return_only_after_the_condition_clears() {
    let key = ErrorNoticeKey {
        source: "Space · example".to_owned(),
        message: "attach failed".to_owned(),
        severity: ErrorSeverity::Warning,
    };
    let first_seen = Instant::now();
    let mut seen_at = HashMap::new();
    let mut dismissed = HashSet::new();

    assert_eq!(
        reconcile_error_notices(vec![key.clone()], &mut seen_at, &mut dismissed, first_seen,).len(),
        1
    );
    dismissed.insert(key.clone());
    assert!(
        reconcile_error_notices(
            vec![key.clone()],
            &mut seen_at,
            &mut dismissed,
            first_seen + Duration::from_secs(1),
        )
        .is_empty()
    );

    reconcile_error_notices(
        Vec::new(),
        &mut seen_at,
        &mut dismissed,
        first_seen + Duration::from_secs(2),
    );
    let recurring = reconcile_error_notices(
        vec![key],
        &mut seen_at,
        &mut dismissed,
        first_seen + Duration::from_secs(3),
    );
    assert_eq!(recurring.len(), 1);
    assert_eq!(recurring[0].first_seen, first_seen + Duration::from_secs(3));
}

#[test]
fn the_bundled_agent_materializes_as_a_native_plugin() {
    let temporary = tempfile::tempdir().unwrap();
    let dir = temporary.path().join("com.chartr.agent");
    std::fs::create_dir_all(&dir).unwrap();
    for legacy_web_asset in ["index.html", "styles.css", "app.js"] {
        std::fs::write(dir.join(legacy_web_asset), "legacy").unwrap();
    }

    let manifest = materialize_bundled_agent(&dir).unwrap();
    assert_eq!(manifest.id, "com.chartr.agent");
    assert_eq!(manifest.kind, zeddy_plugin::manifest::Kind::Native);
    assert_eq!(manifest.icon, "ChipIcon");
    assert!(manifest.icon_path(&dir).is_file());
    assert!(!dir.join("icons/Blockchain01Icon.svg").exists());
    assert!(!dir.join("index.html").exists());
    assert!(!dir.join("styles.css").exists());
    assert!(!dir.join("app.js").exists());
}

#[test]
fn terminal_paths_resolve_relative_locations_without_inventing_an_editor() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("example.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();
    let target = terminal::PathLikeTarget {
        maybe_path: "example.rs:12:3".to_owned(),
        working_directory: Some(directory.path().to_path_buf()),
    };
    assert_eq!(resolve_terminal_path(&target), Some(file));
}

#[test]
fn panes_outside_the_pointer_do_not_overwrite_the_hovered_panes_drop_target() {
    assert_eq!(pane_drop_direction_for_position(100., 100., -0.1, 50.), None);
    assert_eq!(pane_drop_direction_for_position(100., 100., 100.1, 50.), None);
    assert_eq!(pane_drop_direction_for_position(100., 100., 50., -0.1), None);
    assert_eq!(pane_drop_direction_for_position(100., 100., 50., 100.1), None);
    assert_eq!(pane_drop_direction_for_position(100., 100., 50., 50.), Some(None));
    assert_eq!(
        pane_drop_direction_for_position(100., 100., 5., 50.),
        Some(Some(SplitDirection::Left))
    );
}

#[test]
fn zed_drop_zone_has_a_center_and_four_edge_bands() {
    assert_eq!(split_direction_for_position(100., 100., 50., 50.), None);
    assert_eq!(split_direction_for_position(100., 100., 19.9, 50.), Some(SplitDirection::Left));
    assert_eq!(split_direction_for_position(100., 100., 80.1, 50.), Some(SplitDirection::Right));
    assert_eq!(split_direction_for_position(100., 100., 50., 19.9), Some(SplitDirection::Up));
    assert_eq!(split_direction_for_position(100., 100., 50., 80.1), Some(SplitDirection::Down));
}

#[test]
fn zed_drop_zone_uses_the_shorter_side_and_excludes_the_boundary() {
    assert_eq!(split_direction_for_position(400., 100., 20., 50.), None);
    assert_eq!(split_direction_for_position(400., 100., 19.9, 50.), Some(SplitDirection::Left));
    assert_eq!(split_direction_for_position(400., 100., 200., 20.), None);
    assert_eq!(split_direction_for_position(400., 100., 200., 19.9), Some(SplitDirection::Up));
}

#[test]
fn zed_drop_zone_resolves_corners_to_the_nearest_edge() {
    assert_eq!(split_direction_for_position(100., 100., 5., 5.), Some(SplitDirection::Up));
    assert_eq!(split_direction_for_position(100., 100., 96., 8.), Some(SplitDirection::Right));
    assert_eq!(split_direction_for_position(100., 100., 92., 97.), Some(SplitDirection::Down));
    assert_eq!(split_direction_for_position(100., 100., 3., 90.), Some(SplitDirection::Left));
}

#[test]
fn migration_removes_the_empty_root_space_from_registry_and_state() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("spaces.toml");
    let mut registry = Registry::load(&file).unwrap();
    registry.register(Path::new("/")).unwrap();
    let mut saved = Snapshot {
        spaces: vec![PersistedSpace {
            key: "folder:/".to_owned(),
            name: "/".to_owned(),
            path: Some(PathBuf::from("/")),
            kind: PersistedSpaceKind::Folder,
            layout: WorkspaceTabs::new(),
            items: Vec::new(),
            expanded: true,
        }],
        ..Snapshot::default()
    };
    saved.window.active_space = Some("folder:/".to_owned());

    assert!(cleanup_empty_implicit_root(&mut registry, &mut saved, Path::new("/")).unwrap());
    assert!(registry.spaces().is_empty());
    assert!(saved.spaces.is_empty());
    assert_eq!(saved.window.active_space.as_deref(), Some("ad-hoc"));
    assert!(Registry::load(file).unwrap().spaces().is_empty());
}

#[test]
fn migration_keeps_a_root_space_that_owns_items() {
    let temp = tempfile::tempdir().unwrap();
    let mut registry = Registry::load(temp.path().join("spaces.toml")).unwrap();
    registry.register(Path::new("/")).unwrap();
    let mut saved = Snapshot {
        spaces: vec![PersistedSpace {
            key: "folder:/".to_owned(),
            name: "/".to_owned(),
            path: Some(PathBuf::from("/")),
            kind: PersistedSpaceKind::Folder,
            layout: WorkspaceTabs::new(),
            items: vec![crate::persistence::PersistedItem::Plugin {
                item_id: 1,
                plugin: "example.plugin".to_owned(),
                pane: "main".to_owned(),
                state: None,
                bound_session: None,
            }],
            expanded: true,
        }],
        ..Snapshot::default()
    };

    assert!(!cleanup_empty_implicit_root(&mut registry, &mut saved, Path::new("/")).unwrap());
    assert_eq!(registry.spaces().len(), 1);
    assert_eq!(saved.spaces.len(), 1);
}
