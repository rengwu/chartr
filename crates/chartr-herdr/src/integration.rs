//! The pinned Herdr installer writes OpenCode's plugin to ~/.config/opencode,
//! whereas OpenCode 1.2.27 loads plugins from its XDG config directory. Copy
//! that same managed server plugin into this backend's actual terminal config.
//! Its events report the native session without parsing terminal output.
use std::path::{Path, PathBuf};

use crate::{Namespace, Result};

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
        .ok_or_else(|| std::io::Error::other("OpenCode config has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let temporary = target.with_extension("js.installing");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(temporary, target)
}

#[cfg(test)]
mod tests {
    use super::*;

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
