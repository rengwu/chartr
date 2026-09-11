use chartr_conversations::Provider;
use chartr_herdr::protocol::{IntegrationInfo, IntegrationState};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(super) enum Setup {
    #[default]
    Checking,
    Available,
    Outdated,
    Installing,
    Enabled,
    Failed(String),
}

#[derive(Default)]
pub(super) struct Integrations {
    checked: bool,
    states: HashMap<Provider, Setup>,
}

impl Integrations {
    pub fn begin_check(&mut self) -> bool {
        if self.checked {
            return false;
        }
        self.checked = true;
        true
    }

    pub fn get(&self, provider: Provider) -> Setup {
        self.states.get(&provider).cloned().unwrap_or_default()
    }

    pub fn checked(&mut self, result: Result<Vec<IntegrationInfo>, String>) {
        for provider in Provider::all() {
            // A slow status query must not replace a newer installation result.
            if self.states.contains_key(&provider) {
                continue;
            }
            let state = match &result {
                Ok(integrations) => {
                    match integrations.iter().find(|i| i.target == provider.slug()).map(|i| i.state)
                    {
                        Some(IntegrationState::Current) => Setup::Enabled,
                        Some(IntegrationState::Outdated) => Setup::Outdated,
                        Some(IntegrationState::NotInstalled) => Setup::Available,
                        _ => Setup::Failed(
                            "Could not determine the installed integration state.".into(),
                        ),
                    }
                }
                Err(error) => Setup::Failed(error.clone()),
            };
            self.states.insert(provider, state);
        }
    }

    pub fn installing(&mut self, provider: Provider) {
        self.states.insert(provider, Setup::Installing);
    }

    pub fn installed(&mut self, provider: Provider, result: Result<(), String>) {
        self.states.insert(
            provider,
            match result {
                Ok(()) => Setup::Enabled,
                Err(error) => Setup::Failed(error),
            },
        );
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_feedback_survives_delayed_status_and_other_provider_actions() {
        let mut state = Integrations::default();
        assert!(state.begin_check());
        state.installing(Provider::Codex);
        state.checked(Ok(vec![IntegrationInfo {
            target: "codex".into(),
            state: IntegrationState::NotInstalled,
        }]));
        assert_eq!(state.get(Provider::Codex), Setup::Installing);
        state.installed(Provider::Codex, Ok(()));
        state.installing(Provider::Claude);
        state.installed(Provider::Claude, Err("Permission denied".into()));
        assert_eq!(state.get(Provider::Codex), Setup::Enabled);
        assert_eq!(state.get(Provider::Claude), Setup::Failed("Permission denied".into()));
        state.reset();
        assert!(state.begin_check());
        state.checked(Ok(vec![IntegrationInfo {
            target: "codex".into(),
            state: IntegrationState::Current,
        }]));
        assert_eq!(state.get(Provider::Codex), Setup::Enabled);
    }
}
