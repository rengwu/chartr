//! Compatibility fixes for the pinned Herdr integrations.
//! The installer writes OpenCode's plugin to ~/.config/opencode,
//! whereas OpenCode 1.2.27 loads plugins from its XDG config directory. Copy
//! that same managed server plugin into this backend's actual terminal config.
//! Its events report the native session without parsing terminal output.
use std::path::{Path, PathBuf};

use crate::{Namespace, Result};

fn pi_paths() -> (PathBuf, PathBuf) {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let installed = home.join(".pi/agent/extensions/herdr-agent-state.ts");
    let config = std::env::var_os("PI_CODING_AGENT_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| path.strip_prefix("~").map(|rest| home.join(rest)).unwrap_or(path.clone()))
        .unwrap_or_else(|| home.join(".pi/agent"));
    (installed, config.join("extensions/herdr-agent-state.ts"))
}

pub(crate) fn pi_current() -> bool {
    let (source, target) = pi_paths();
    match (std::fs::read_to_string(source), std::fs::read_to_string(target)) {
        (Ok(source), Ok(target)) => {
            source == target && compatible_pi_plugin(&source).is_ok_and(|patched| patched == source)
        }
        _ => false,
    }
}

pub(crate) fn install_pi() -> Result<()> {
    let (source, target) = pi_paths();
    let patched = compatible_pi_plugin(&std::fs::read_to_string(&source)?)?;
    write_managed_plugin(&source, patched.as_bytes())?;
    write_managed_plugin(&target, patched.as_bytes())?;
    Ok(())
}

const PI_TUI_GATE: &str = "if (ctx?.mode !== \"tui\")";
const PI_COMPAT_GATE: &str =
    "if (ctx?.mode !== undefined ? ctx.mode !== \"tui\" : ctx?.hasUI !== true)";
const PI_SETTLED: &str = "  pi.on(\"agent_settled\", (_event, ctx) => {";
const PI_LEGACY_EVENTS: &str = r#"  // Chartr: older Pi releases expose hasUI and agent_end instead of mode/agent_settled.
  pi.on("agent_end", (_event, ctx) => {
    if (!rootSession || ctx?.mode !== undefined) return;
    agentActive = false;
    publishState();
  });

  for (const event of ["session_switch", "session_fork"]) {
    pi.on(event, (_event, ctx) => {
      if (!rootSession || ctx?.mode !== undefined) return;
      updateSessionRef(ctx);
      void reportSession();
      agentActive = ctx?.isIdle?.() === false;
      publishState(true);
    });
  }

"#;

fn compatible_pi_plugin(source: &str) -> std::io::Result<String> {
    if !source.starts_with("// installed by herdr\n")
        || !source.contains("// HERDR_INTEGRATION_ID=pi\n")
        || !(source.contains(PI_TUI_GATE) || source.contains(PI_COMPAT_GATE))
        || !source.contains(PI_SETTLED)
    {
        return Err(std::io::Error::other("Unrecognized managed Pi integration"));
    }
    let mut patched = source.replace(PI_TUI_GATE, PI_COMPAT_GATE);
    if !patched.contains(PI_LEGACY_EVENTS) {
        patched = patched.replace(PI_SETTLED, &format!("{PI_LEGACY_EVENTS}{PI_SETTLED}"));
    }
    Ok(patched)
}

pub(crate) fn opencode_paths(namespace: &Namespace) -> (PathBuf, PathBuf) {
    let installed = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config/opencode/plugins/herdr-agent-state.js");
    let config = std::env::var_os("OPENCODE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| namespace.session_config_home().join("opencode"));
    (installed, config.join("plugins/herdr-agent-state.js"))
}

pub(crate) fn opencode_current(namespace: &Namespace) -> bool {
    let (source, target) = opencode_paths(namespace);
    match (std::fs::read(source), std::fs::read(target)) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

pub(crate) fn install_opencode(namespace: &Namespace) -> Result<()> {
    let (source, target) = opencode_paths(namespace);
    copy_managed_plugin(&source, &target)?;
    Ok(())
}

fn copy_managed_plugin(source: &Path, target: &Path) -> std::io::Result<()> {
    let bytes = std::fs::read(source)?;
    write_managed_plugin(target, &bytes)
}

fn write_managed_plugin(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Ok(existing) = std::fs::read(target) {
        if existing == bytes {
            return Ok(());
        }
        if !existing.starts_with(b"// installed by herdr\n") {
            return Err(std::io::Error::other(format!(
                "Refusing to overwrite an unmanaged plugin at {}",
                target.display()
            )));
        }
    }
    let parent = target
        .parent()
        .ok_or_else(|| std::io::Error::other("Plugin config has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let temporary = target.with_extension("installing");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(temporary, target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_compatibility_is_idempotent_and_rejects_unknown_or_unmanaged_hooks() {
        let source = "// installed by herdr\n// HERDR_INTEGRATION_ID=pi\nexport default function(pi) {\n  pi.on(\"session_start\", (event, ctx) => {\n    if (ctx?.mode !== \"tui\") return;\n  });\n  pi.on(\"agent_settled\", (_event, ctx) => {\n  });\n}\n";
        let patched = compatible_pi_plugin(source).unwrap();
        assert_eq!(compatible_pi_plugin(&patched).unwrap(), patched);
        assert!(patched.contains("ctx?.hasUI !== true"));
        assert_eq!(patched.matches("pi.on(\"agent_end\"").count(), 1);
        assert!(patched.contains(PI_SETTLED));
        assert!(compatible_pi_plugin(&source.replace("// installed by herdr\n", "")).is_err());
        assert!(compatible_pi_plugin(&source.replace("ctx?.mode", "ctx.mode")).is_err());
    }

    #[test]
    fn installs_in_the_loaded_directory_without_overwriting_other_plugins() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("managed.js");
        let target = dir.path().join("custom-config/plugins/herdr-agent-state.js");
        std::fs::write(&source, "// installed by herdr\nexport const hook = 1;\n").unwrap();
        copy_managed_plugin(&source, &target).unwrap();
        copy_managed_plugin(&source, &target).unwrap();
        assert_eq!(std::fs::read(&source).unwrap(), std::fs::read(&target).unwrap());
        std::fs::write(&target, "// custom plugin\n").unwrap();
        assert!(copy_managed_plugin(&source, &target).is_err());
        assert_eq!(std::fs::read_to_string(target).unwrap(), "// custom plugin\n");
    }
}
