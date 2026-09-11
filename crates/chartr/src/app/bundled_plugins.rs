//! Bundled plugin catalog and packaged assets.

use super::*;

#[cfg(test)]
mod settings_surface_tests;

const BUNDLED_AGENT_ID: &str = "com.chartr.agent";
const BUNDLED_SKILLS_ID: &str = "com.chartr.skills";
const BUNDLED_PROMPTS_ID: &str = "com.chartr.prompts";
const BUNDLED_WAYFINDER_ID: &str = "com.chartr.wayfinder";

pub(super) fn load_plugin_catalog(settings: &SettingsStore, cx: &mut App) -> Catalog {
    load_plugin_catalog_at(settings, &plugin_paths(), cx)
}

fn load_plugin_catalog_at(settings: &SettingsStore, paths: &Paths, cx: &mut App) -> Catalog {
    let mut catalog = chartr_plugin_host::load_all_where(paths, |_| false, cx);
    for rejected in &catalog.rejected {
        eprintln!("chartr rejected plugin {}: {}", rejected.dir.display(), rejected.why);
    }

    if !catalog.contains(BUNDLED_AGENT_ID)
        && !settings.resolved().plugin(BUNDLED_AGENT_ID).uninstalled
    {
        let dir = paths.bundled.join(BUNDLED_AGENT_ID);
        match materialize_bundled_agent(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                paths,
                false,
                crate::agent_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }

    if !catalog.contains(BUNDLED_SKILLS_ID)
        && !settings.resolved().plugin(BUNDLED_SKILLS_ID).uninstalled
    {
        let dir = paths.bundled.join(BUNDLED_SKILLS_ID);
        match materialize_bundled_skills(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                paths,
                false,
                crate::skills_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }
    if !catalog.contains(BUNDLED_PROMPTS_ID)
        && !settings.resolved().plugin(BUNDLED_PROMPTS_ID).uninstalled
    {
        let dir = paths.bundled.join(BUNDLED_PROMPTS_ID);
        match materialize_bundled_prompts(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                paths,
                false,
                crate::prompts_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }
    let markdown_id = "com.chartr.markdown-prompt";
    if !catalog.contains(markdown_id) && !settings.resolved().plugin(markdown_id).uninstalled {
        let dir = paths.bundled.join(markdown_id);
        match materialize_bundled_markdown_prompt(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                paths,
                false,
                crate::markdown_prompt_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }
    if !catalog.contains(BUNDLED_WAYFINDER_ID)
        && !settings.resolved().plugin(BUNDLED_WAYFINDER_ID).uninstalled
    {
        let dir = paths.bundled.join(BUNDLED_WAYFINDER_ID);
        match materialize_bundled_wayfinder(&dir) {
            Ok(_) => catalog.add_directory(&dir, paths, false, cx),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }
    let id = "com.chartr.companion";
    if !catalog.contains(id) && !settings.resolved().plugin(id).uninstalled {
        let dir = paths.bundled.join(id);
        let result = (|| -> Result<chartr_plugin::Manifest, String> {
            std::fs::create_dir_all(dir.join("icons")).map_err(|e| e.to_string())?;
            for (name, bytes) in [
                (
                    "chartr-plugin.toml",
                    include_bytes!("../../../../plugins/companion/chartr-plugin.toml").as_slice(),
                ),
                (
                    "icons/ChipIcon.svg",
                    include_bytes!("../../../../plugins/companion/icons/ChipIcon.svg").as_slice(),
                ),
            ] {
                write_bundled_file(&dir.join(name), bytes).map_err(|e| e.to_string())?;
            }
            chartr_plugin::Manifest::read(&dir).map_err(|e| e.to_string())
        })();
        match result {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                paths,
                false,
                crate::companion_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(chartr_plugin_host::Rejected { dir, why }),
        }
    }
    catalog.enable_requested(paths, |id| settings.resolved().plugin(id).enabled, cx);
    catalog
}

pub(super) fn materialize_bundled_wayfinder(
    dir: &std::path::Path,
) -> Result<chartr_plugin::Manifest, String> {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("icons"))?;
        write_bundled_file(
            &dir.join("chartr-plugin.toml"),
            include_bytes!("../../../../plugins/wayfinder/chartr-plugin.toml"),
        )?;
        write_bundled_file(
            &dir.join("icons/Orbit01Icon.svg"),
            include_bytes!("../../../../plugins/wayfinder/icons/Orbit01Icon.svg"),
        )?;
        write_bundled_file(
            &dir.join("TRACKER-CONVENTION.md"),
            include_bytes!("../../../../plugins/wayfinder/TRACKER-CONVENTION.md"),
        )?;
        for (name, bytes) in [
            ("index.html", include_bytes!("../../../../plugins/wayfinder/index.html").as_slice()),
            ("styles.css", include_bytes!("../../../../plugins/wayfinder/styles.css").as_slice()),
            ("app.js", include_bytes!("../../../../plugins/wayfinder/app.js").as_slice()),
            ("starmap.js", include_bytes!("../../../../plugins/wayfinder/starmap.js").as_slice()),
            (
                "icons/ui.svg",
                include_bytes!("../../../../plugins/wayfinder/icons/ui.svg").as_slice(),
            ),
            (
                "icons/HUGEICONS_NOTICE.md",
                include_bytes!("../../../../plugins/wayfinder/icons/HUGEICONS_NOTICE.md")
                    .as_slice(),
            ),
        ] {
            write_bundled_file(&dir.join(name), bytes)?;
        }
        Ok(())
    };
    write().map_err(|error| format!("Cannot prepare Wayfinder: {error}"))?;
    chartr_plugin::Manifest::read(dir).map_err(|error| error.to_string())
}

fn materialize_bundled_markdown_prompt(
    dir: &std::path::Path,
) -> Result<chartr_plugin::Manifest, String> {
    std::fs::create_dir_all(dir.join("icons")).map_err(|e| e.to_string())?;
    write_bundled_file(
        &dir.join("chartr-plugin.toml"),
        include_bytes!("../../../../plugins/markdown-prompt/chartr-plugin.toml"),
    )
    .map_err(|e| e.to_string())?;
    write_bundled_file(
        &dir.join("icons/BookOpen01Icon.svg"),
        include_bytes!("../../../../plugins/markdown-prompt/icons/BookOpen01Icon.svg"),
    )
    .map_err(|e| e.to_string())?;
    chartr_plugin::Manifest::read(dir).map_err(|e| e.to_string())
}

fn materialize_bundled_prompts(dir: &std::path::Path) -> Result<chartr_plugin::Manifest, String> {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("icons"))?;
        write_bundled_file(
            &dir.join("chartr-plugin.toml"),
            include_bytes!("../../../../plugins/prompts/chartr-plugin.toml"),
        )?;
        write_bundled_file(
            &dir.join("icons/BookOpen01Icon.svg"),
            include_bytes!("../../../../plugins/prompts/icons/BookOpen01Icon.svg"),
        )
    };
    write().map_err(|why| format!("cannot prepare the bundled Prompts plugin: {why}"))?;
    chartr_plugin::Manifest::read(dir).map_err(|why| why.to_string())
}

fn materialize_bundled_skills(dir: &std::path::Path) -> Result<chartr_plugin::Manifest, String> {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("icons"))?;
        write_bundled_file(
            &dir.join("chartr-plugin.toml"),
            include_bytes!("../../../../plugins/skills/chartr-plugin.toml"),
        )?;
        write_bundled_file(
            &dir.join("icons/BookOpen01Icon.svg"),
            include_bytes!("../../../../plugins/skills/icons/BookOpen01Icon.svg"),
        )
    };
    write().map_err(|why| format!("cannot prepare the bundled Skills plugin: {why}"))?;
    chartr_plugin::Manifest::read(dir).map_err(|why| why.to_string())
}

pub(super) fn materialize_bundled_agent(
    dir: &std::path::Path,
) -> Result<chartr_plugin::Manifest, String> {
    std::fs::create_dir_all(dir)
        .map_err(|why| format!("cannot prepare the bundled Agent plugin: {why}"))?;
    write_bundled_file(
        &dir.join("chartr-plugin.toml"),
        include_bytes!("../../../../plugins/agent/chartr-plugin.toml"),
    )
    .map_err(|why| format!("cannot prepare the bundled Agent plugin: {why}"))?;
    std::fs::create_dir_all(dir.join("icons"))
        .map_err(|why| format!("cannot prepare the bundled Agent plugin: {why}"))?;
    write_bundled_file(
        &dir.join("icons/ChipIcon.svg"),
        include_bytes!("../../../../plugins/agent/icons/ChipIcon.svg"),
    )
    .map_err(|why| format!("cannot prepare the bundled Agent plugin: {why}"))?;
    match std::fs::remove_file(dir.join("icons/Blockchain01Icon.svg")) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "cannot remove the bundled Agent plugin's legacy Blockchain01Icon.svg: {error}"
            ));
        }
    }
    for legacy_web_asset in ["index.html", "styles.css", "app.js"] {
        match std::fs::remove_file(dir.join(legacy_web_asset)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "cannot remove the bundled Agent plugin's legacy {legacy_web_asset}: {error}"
                ));
            }
        }
    }
    chartr_plugin::Manifest::read(dir).map_err(|why| why.to_string())
}

fn write_bundled_file(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|current| current == contents) {
        return Ok(());
    }
    std::fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn rejected_installed_overrides_do_not_create_a_second_bundled_entry(
        cx: &mut gpui::TestAppContext,
    ) {
        let scratch = tempfile::tempdir().unwrap();
        let paths = Paths::under(scratch.path());
        let directory = paths.installed.join(BUNDLED_WAYFINDER_ID);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("chartr-plugin.toml"), "invalid manifest").unwrap();
        let catalog = cx.update(|cx| load_plugin_catalog_at(&SettingsStore::bare(), &paths, cx));
        assert!(catalog.get(BUNDLED_WAYFINDER_ID).is_none());
        assert!(!catalog.disabled.contains_key(BUNDLED_WAYFINDER_ID));
        assert_eq!(catalog.rejected.iter().filter(|rejected| rejected.dir == directory).count(), 1);
    }

    #[gpui::test]
    fn removed_bundles_stay_removed_and_dependents_stay_blocked_after_restart(
        cx: &mut gpui::TestAppContext,
    ) {
        let scratch = tempfile::tempdir().unwrap();
        let paths = Paths::under(scratch.path().join("data"));
        let file = scratch.path().join("settings.toml");
        let mut settings = SettingsStore::load(&file);
        settings
            .update(|content| {
                let agent = content.plugins.entry(BUNDLED_AGENT_ID.into()).or_default();
                agent.enabled = Some(false);
                agent.uninstalled = Some(true);
            })
            .unwrap();
        let settings = SettingsStore::load(&file);
        let catalog = cx.update(|cx| load_plugin_catalog_at(&settings, &paths, cx));
        assert!(!catalog.contains(BUNDLED_AGENT_ID));
        assert!(!paths.bundled.join(BUNDLED_AGENT_ID).exists());
        assert!(catalog.disabled.contains_key(BUNDLED_WAYFINDER_ID));
        assert!(catalog.prerequisite_error(BUNDLED_WAYFINDER_ID).is_some());
        for id in ["com.example.clock", "com.example.hello"] {
            assert!(!catalog.contains(id));
            assert!(!paths.bundled.join(id).exists());
        }
        assert!(catalog.get(BUNDLED_SKILLS_ID).is_some());
        assert!(catalog.get(BUNDLED_PROMPTS_ID).is_some());
        assert_eq!(catalog.loaded.len() + catalog.disabled.len(), 5);
    }

    #[gpui::test]
    fn wayfinder_materializes_as_a_restorable_web_package(cx: &mut gpui::TestAppContext) {
        let scratch = tempfile::tempdir().unwrap();
        let dir = scratch.path().join(BUNDLED_WAYFINDER_ID);
        std::fs::create_dir_all(&dir).unwrap();
        // Updating the earlier native bundle keeps the same stable package ID.
        let old = include_str!("../../../../plugins/wayfinder/chartr-plugin.toml")
            .replace("kind = \"web\"", "kind = \"native\"");
        std::fs::write(dir.join("chartr-plugin.toml"), old).unwrap();
        let manifest = materialize_bundled_wayfinder(&dir).unwrap();
        assert_eq!(manifest.kind, chartr_plugin::manifest::Kind::Web);
        assert!(manifest.capabilities.restorable && manifest.permissions.wayfinder);
        for name in [
            "index.html",
            "styles.css",
            "app.js",
            "starmap.js",
            "icons/ui.svg",
            "icons/HUGEICONS_NOTICE.md",
        ] {
            assert!(dir.join(name).is_file());
        }
        let paths = chartr_plugin_host::Paths {
            installed: scratch.path().join("installed"),
            bundled: scratch.path().to_owned(),
            data: scratch.path().join("data"),
        };
        cx.update(|cx| {
            let mut catalog = Catalog::default();
            catalog.add_directory(&dir, &paths, false, cx);
            assert!(catalog.rejected.is_empty());
            assert!(catalog.prerequisite_error(BUNDLED_WAYFINDER_ID).is_some());
            assert!(catalog.enable(&paths, BUNDLED_WAYFINDER_ID, cx).is_err());
            assert_eq!(
                catalog.disabled[BUNDLED_WAYFINDER_ID].manifest.entry.as_deref(),
                Some("index.html")
            );
        });
    }
}
