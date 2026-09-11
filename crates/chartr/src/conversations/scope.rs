use super::*;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceChoice {
    pub key: String,
    pub name: String,
    pub path: Option<PathBuf>,
}

impl Conversations {
    pub fn set_spaces(
        &mut self,
        spaces: Vec<SpaceChoice>,
        all: bool,
        active: Option<String>,
    ) -> bool {
        let scope = if all { None } else { Some(active.clone().unwrap_or_default()) };
        let changed = self.scope != scope;
        let notify = changed || self.spaces != spaces || self.active_space != active;
        self.spaces = spaces;
        self.active_space = active;
        self.scope = scope;
        if changed {
            self.clear_terminal();
            self.pending_runtime = None;
            self.new_agent = None;
            self.new_space = None;
            self.launch_runtime = None;
            if self.store.is_some() {
                self.problem = None;
            }
            if !self
                .selected
                .as_ref()
                .and_then(|id| self.rows.iter().find(|r| &r.id == id))
                .is_some_and(|row| self.in_scope(row))
            {
                self.selected = None;
            }
        }
        notify
    }

    pub(super) fn in_scope(&self, row: &Conversation) -> bool {
        self.scope.as_ref().is_none_or(|scope| owner_key(row, &self.spaces) == *scope)
    }

    pub(super) fn space_label(&self, row: &Conversation) -> String {
        let key = owner_key(row, &self.spaces);
        if let Some(space) = self.spaces.iter().find(|space| space.key == key) {
            return choice_label(space, &self.spaces);
        }
        row.space.as_ref().map(|space| space.name.clone()).unwrap_or_else(|| {
            row.cwd
                .as_deref()
                .and_then(Path::file_name)
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Free sessions".into())
        })
    }
}

pub(super) fn choice_label(space: &SpaceChoice, spaces: &[SpaceChoice]) -> String {
    if spaces.iter().filter(|other| other.name == space.name).count() > 1 {
        if let Some(path) = &space.path {
            return format!("{} · {}", space.name, path.display());
        }
    }
    space.name.clone()
}

/// Old history lacks an owner. Use component-aware, most-specific folder
/// matching only for those legacy rows; explicit ownership always wins.
fn owner_key(row: &Conversation, spaces: &[SpaceChoice]) -> String {
    if let Some(space) = &row.space {
        return space.key.clone();
    }
    row.cwd
        .as_ref()
        .and_then(|cwd| {
            spaces
                .iter()
                .filter(|space| space.path.as_ref().is_some_and(|path| cwd.starts_with(path)))
                .max_by_key(|space| space.path.as_ref().map(|path| path.components().count()))
        })
        .map(|space| space.key.clone())
        .unwrap_or_else(|| {
            row.cwd
                .as_ref()
                .map(|cwd| format!("folder:{}", cwd.display()))
                .unwrap_or_else(|| "ad-hoc".into())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn space(path: &str) -> SpaceChoice {
        SpaceChoice {
            key: format!("folder:{path}"),
            name: "Project".into(),
            path: Some(path.into()),
        }
    }
    fn legacy(cwd: &str) -> Conversation {
        serde_json::from_value(serde_json::json!({
            "id":"old", "provider":"codex", "native":null, "title":"A task", "custom_title":null,
            "cwd":cwd, "updated":1, "draft":"keep this", "archived":false, "messages":[]
        }))
        .unwrap()
    }
    #[test]
    fn legacy_history_uses_folder_boundaries_and_the_most_specific_space() {
        let spaces = vec![space("/work/app"), space("/work/app/packages/api")];
        assert_eq!(owner_key(&legacy("/work/app/src"), &spaces), "folder:/work/app");
        assert_eq!(
            owner_key(&legacy("/work/app/packages/api/src"), &spaces),
            "folder:/work/app/packages/api"
        );
        assert_eq!(owner_key(&legacy("/work/application"), &spaces), "folder:/work/application");
        assert_eq!(legacy("/work/app").draft, "keep this");
    }
    #[test]
    fn explicit_ownership_wins_over_cwd_and_survives_removing_the_space() {
        let mut row = legacy("/work/app/src");
        row.space = Some(chartr_conversations::SpaceIdentity {
            key: "ad-hoc".into(),
            name: "Free sessions".into(),
        });
        assert_eq!(owner_key(&row, &[space("/work/app")]), "ad-hoc");
        assert_eq!(owner_key(&row, &[]), "ad-hoc");
    }
    #[test]
    fn duplicate_space_names_are_disambiguated() {
        let spaces = vec![space("/work/one"), space("/work/two")];
        assert_ne!(choice_label(&spaces[0], &spaces), choice_label(&spaces[1], &spaces));
        assert_eq!(choice_label(&spaces[0], &spaces[..1]), "Project");
    }
}
