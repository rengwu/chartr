//! Declarative settings rendered by chartr's native UI, never a plugin webview.

use serde::Deserialize;
use std::{
    collections::HashSet,
    path::{Component, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSchema {
    /// JSON object in the plugin's private data directory; shared with its panes.
    pub file: PathBuf,
    pub fields: Vec<SettingsField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SettingsField {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub control: SettingsControl,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SettingsControl {
    Select { default: String, options: Vec<SettingsOption> },
    Toggle { default: bool },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsOption {
    pub value: String,
    pub label: String,
}

impl SettingsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.file.as_os_str().is_empty()
            || self.file.to_string_lossy().contains(['\\', ':'])
            || self.file.components().any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err("settings.file must be a relative path inside plugin data".into());
        }
        if self.fields.is_empty() || self.fields.len() > 64 {
            return Err("settings must declare between 1 and 64 fields".into());
        }
        let mut keys = HashSet::new();
        for field in &self.fields {
            if field.key.trim().is_empty() || !keys.insert(&field.key) {
                return Err("settings field keys must be nonempty and unique".into());
            }
            if field.label.trim().is_empty() {
                return Err(format!("settings field '{}' needs a label", field.key));
            }
            if let SettingsControl::Select { default, options } = &field.control {
                let mut values = HashSet::new();
                if options.is_empty()
                    || options.len() > 64
                    || options.iter().any(|option| {
                        option.label.trim().is_empty() || !values.insert(&option.value)
                    })
                    || !options.iter().any(|option| &option.value == default)
                {
                    return Err(format!(
                        "settings field '{}' needs unique, labeled options and a default from those options",
                        field.key
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> SettingsSchema {
        toml::from_str(
            r#"
file = "settings.json"
[[fields]]
key = "format"
label = "Clock format"
type = "select"
default = "24"
options = [{ value = "24", label = "24-hour" }, { value = "12", label = "12-hour" }]
[[fields]]
key = "seconds"
label = "Show seconds"
type = "toggle"
default = true
"#,
        )
        .unwrap()
    }

    #[test]
    fn portable_settings_require_contained_paths_and_unambiguous_fields() {
        let original = schema();
        original.validate().unwrap();
        for path in [
            "",
            "../settings.json",
            "/tmp/settings.json",
            "nested/../../settings.json",
            "C:\\settings.json",
        ] {
            let mut settings = original.clone();
            settings.file = path.into();
            assert!(settings.validate().is_err(), "{path}");
        }
        let mut duplicate = original.clone();
        duplicate.fields.push(duplicate.fields[0].clone());
        assert!(duplicate.validate().is_err());
        let mut invalid = original.clone();
        if let SettingsControl::Select { default, .. } = &mut invalid.fields[0].control {
            *default = "unknown".into();
        }
        assert!(invalid.validate().is_err());
        let mut invalid = original.clone();
        if let SettingsControl::Select { options, .. } = &mut invalid.fields[0].control {
            options.push(options[0].clone());
        }
        assert!(invalid.validate().is_err());
    }
}
