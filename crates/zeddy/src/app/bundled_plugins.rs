//! Bundled plugin catalog and packaged assets.

use super::*;

const BUNDLED_HELLO_ID: &str = "com.example.hello";
const BUNDLED_CLOCK_ID: &str = "com.example.clock";
const BUNDLED_AGENT_ID: &str = "com.chartr.agent";
const BUNDLED_SKILLS_ID: &str = "com.chartr.skills";
const BUNDLED_WAYFINDER_ID: &str = "com.chartr.wayfinder";

pub(super) fn load_plugin_catalog(settings: &SettingsStore, cx: &mut App) -> Catalog {
    let paths = plugin_paths();
    let mut catalog =
        zeddy_plugin_host::load_all_where(&paths, |id| settings.resolved().plugin(id).enabled, cx);
    for rejected in &catalog.rejected {
        eprintln!("Chartr rejected plugin {}: {}", rejected.dir.display(), rejected.why);
    }

    if !catalog.contains(BUNDLED_HELLO_ID) {
        let dir = paths.bundled.join(BUNDLED_HELLO_ID);
        match materialize_bundled_hello(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                &paths,
                settings.resolved().plugin(BUNDLED_HELLO_ID).enabled,
                crate::hello_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(zeddy_plugin_host::Rejected { dir, why }),
        }
    }

    if !catalog.contains(BUNDLED_CLOCK_ID) {
        let dir = paths.bundled.join(BUNDLED_CLOCK_ID);
        match materialize_bundled_clock(&dir) {
            Ok(()) => catalog.add_directory(
                &dir,
                &paths,
                settings.resolved().plugin(BUNDLED_CLOCK_ID).enabled,
                cx,
            ),
            Err(why) => catalog.rejected.push(zeddy_plugin_host::Rejected {
                dir,
                why: format!("cannot prepare the bundled Clock plugin: {why}"),
            }),
        }
    }

    if !catalog.contains(BUNDLED_AGENT_ID) {
        let dir = paths.bundled.join(BUNDLED_AGENT_ID);
        match materialize_bundled_agent(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                &paths,
                settings.resolved().plugin(BUNDLED_AGENT_ID).enabled,
                crate::agent_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(zeddy_plugin_host::Rejected { dir, why }),
        }
    }

    if !catalog.contains(BUNDLED_SKILLS_ID) {
        let dir = paths.bundled.join(BUNDLED_SKILLS_ID);
        match materialize_bundled_skills(&dir) {
            Ok(manifest) => catalog.add_bundled_native(
                manifest,
                dir,
                &paths,
                settings.resolved().plugin(BUNDLED_SKILLS_ID).enabled,
                crate::skills_plugin::bundled,
                cx,
            ),
            Err(why) => catalog.rejected.push(zeddy_plugin_host::Rejected { dir, why }),
        }
    }
    if !catalog.contains(BUNDLED_WAYFINDER_ID) {
        let dir = paths.bundled.join(BUNDLED_WAYFINDER_ID);
        match materialize_bundled_wayfinder(&dir) {
            Ok(_) => catalog.add_directory(
                &dir,
                &paths,
                settings.resolved().plugin(BUNDLED_WAYFINDER_ID).enabled,
                cx,
            ),
            Err(why) => catalog.rejected.push(zeddy_plugin_host::Rejected { dir, why }),
        }
    }
    catalog
}

pub(super) fn materialize_bundled_wayfinder(
    dir: &std::path::Path,
) -> Result<zeddy_plugin::Manifest, String> {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("icons"))?;
        write_bundled_file(
            &dir.join("zeddy-plugin.toml"),
            include_bytes!("../../../../plugins/wayfinder/zeddy-plugin.toml"),
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
        ] {
            write_bundled_file(&dir.join(name), bytes)?;
        }
        Ok(())
    };
    write().map_err(|error| format!("Cannot prepare Wayfinder: {error}"))?;
    zeddy_plugin::Manifest::read(dir).map_err(|error| error.to_string())
}

fn materialize_bundled_skills(dir: &std::path::Path) -> Result<zeddy_plugin::Manifest, String> {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("icons"))?;
        write_bundled_file(
            &dir.join("zeddy-plugin.toml"),
            include_bytes!("../../../../plugins/skills/zeddy-plugin.toml"),
        )?;
        write_bundled_file(
            &dir.join("icons/BookOpen01Icon.svg"),
            include_bytes!("../../../../plugins/skills/icons/BookOpen01Icon.svg"),
        )
    };
    write().map_err(|why| format!("cannot prepare the bundled Skills plugin: {why}"))?;
    zeddy_plugin::Manifest::read(dir).map_err(|why| why.to_string())
}

pub(super) fn materialize_bundled_hello(
    dir: &std::path::Path,
) -> Result<zeddy_plugin::Manifest, String> {
    std::fs::create_dir_all(dir)
        .map_err(|why| format!("cannot prepare the bundled Hello plugin: {why}"))?;
    write_bundled_file(
        &dir.join("zeddy-plugin.toml"),
        include_bytes!("../../../../plugins/hello/zeddy-plugin.toml"),
    )
    .map_err(|why| format!("cannot prepare the bundled Hello plugin: {why}"))?;
    std::fs::create_dir_all(dir.join("icons"))
        .map_err(|why| format!("cannot prepare the bundled Hello plugin: {why}"))?;
    write_bundled_file(
        &dir.join("icons/WavingHand01Icon.svg"),
        include_bytes!("../../../../plugins/hello/icons/WavingHand01Icon.svg"),
    )
    .map_err(|why| format!("cannot prepare the bundled Hello plugin: {why}"))?;
    zeddy_plugin::Manifest::read(dir).map_err(|why| why.to_string())
}

pub(super) fn materialize_bundled_clock(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_bundled_file(
        &dir.join("zeddy-plugin.toml"),
        include_bytes!("../../../../plugins/clock/zeddy-plugin.toml"),
    )?;
    std::fs::create_dir_all(dir.join("icons"))?;
    write_bundled_file(
        &dir.join("icons/Clock01Icon.svg"),
        include_bytes!("../../../../plugins/clock/icons/Clock01Icon.svg"),
    )?;
    write_bundled_file(
        &dir.join("index.html"),
        include_bytes!("../../../../plugins/clock/index.html"),
    )?;
    write_bundled_file(
        &dir.join("settings.html"),
        include_bytes!("../../../../plugins/clock/settings.html"),
    )
}

pub(super) fn materialize_bundled_agent(
    dir: &std::path::Path,
) -> Result<zeddy_plugin::Manifest, String> {
    std::fs::create_dir_all(dir)
        .map_err(|why| format!("cannot prepare the bundled Agent plugin: {why}"))?;
    write_bundled_file(
        &dir.join("zeddy-plugin.toml"),
        include_bytes!("../../../../plugins/agent/zeddy-plugin.toml"),
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
    zeddy_plugin::Manifest::read(dir).map_err(|why| why.to_string())
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
    fn wayfinder_materializes_as_a_restorable_web_package(cx: &mut gpui::TestAppContext) {
        let scratch = tempfile::tempdir().unwrap();
        let dir = scratch.path().join(BUNDLED_WAYFINDER_ID);
        std::fs::create_dir_all(&dir).unwrap();
        // Updating the earlier native bundle keeps the same stable package ID.
        let old = include_str!("../../../../plugins/wayfinder/zeddy-plugin.toml")
            .replace("kind = \"web\"", "kind = \"native\"");
        std::fs::write(dir.join("zeddy-plugin.toml"), old).unwrap();
        let manifest = materialize_bundled_wayfinder(&dir).unwrap();
        assert_eq!(manifest.kind, zeddy_plugin::manifest::Kind::Web);
        assert!(manifest.capabilities.restorable && manifest.permissions.wayfinder);
        for name in ["index.html", "styles.css", "app.js", "starmap.js"] {
            assert!(dir.join(name).is_file());
        }
        let paths = zeddy_plugin_host::Paths {
            installed: scratch.path().join("installed"),
            bundled: scratch.path().to_owned(),
            data: scratch.path().join("data"),
        };
        cx.update(|cx| {
            let mut catalog = Catalog::default();
            catalog.add_directory(&dir, &paths, true, cx);
            assert!(catalog.rejected.is_empty());
            assert!(matches!(
                catalog
                    .get_mut(BUNDLED_WAYFINDER_ID)
                    .unwrap()
                    .pane(&zeddy_plugin::PaneKey::new(BUNDLED_WAYFINDER_ID, "main")),
                Some(PaneSource::Web(_))
            ));
        });
    }
}
