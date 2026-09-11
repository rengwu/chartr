use super::*;
use chartr_conversations::Delivery;
use chartr_herdr::control::{AgentInputTarget, Client};

type Failure = (bool, String);

/// Runs off the UI thread. Keep the receipt even if a transport result arrives
/// after an observation: matching the native transcript can resolve either order.
pub(super) fn send(
    store: &Arc<Mutex<Store>>,
    id: &str,
    terminal_client: Option<Client>,
    message_id: String,
    text: String,
) -> (Result<(), Failure>, Option<Delivery>) {
    let definitive = |error: anyhow::Error| (false, error.to_string());
    let locked = || store.lock().map_err(|_| (true, "Conversation store unavailable".to_owned()));
    let mut receipt = None;
    let result = (|| {
        let provider = locked()?
            .get(id)
            .map(|row| row.provider)
            .ok_or_else(|| (false, "Conversation no longer exists".to_owned()))?;
        if provider.transport() == chartr_agent::MessageTransport::TerminalPrompt {
            let client = terminal_client
                .ok_or_else(|| (false, "Terminal service is unavailable".to_owned()))?;
            let observation = locked()?.terminal_target(id).map_err(definitive)?;
            let binding = AgentInputTarget {
                pane: chartr_herdr::PaneId(observation.runtime),
                terminal: chartr_herdr::TerminalId(observation.terminal),
                provider: observation.provider.slug().into(),
                native_id: observation.native.unwrap().id,
                pid: observation
                    .pid
                    .ok_or_else(|| (false, "Waiting for the agent process".to_owned()))?,
            };
            client.check_agent_input(&binding).map_err(|e| (false, e.to_string()))?;
            {
                let mut store = locked()?;
                store.begin_terminal_delivery(id, message_id, text.clone()).map_err(definitive)?;
                receipt = store.get(id).and_then(|row| row.delivery.clone());
            }
            if let Err(error) = client.send_agent_input(&binding, &text) {
                if !error.uncertain {
                    locked()?.confirm_delivery(id).map_err(|e| (true, e.to_string()))?;
                    receipt = None;
                }
                return Err((error.uncertain, error.message));
            }
            // A PTY write only acknowledges bytes. Reconciliation confirms the
            // new user item in the original CLI's structured transcript.
            return Ok(());
        }

        let (client, native) = locked()?.live_client(id).map_err(definitive)?;
        client.ready(&native).map_err(definitive)?;
        {
            let mut store = locked()?;
            store.begin_delivery(id, message_id.clone(), text.clone()).map_err(definitive)?;
            receipt = store.get(id).and_then(|row| row.delivery.clone());
        }
        client.send(&native, &message_id, &text).map_err(|e| (true, e.to_string()))?;
        locked()?.confirm_delivery(id).map_err(|e| (true, e.to_string()))?;
        receipt = None;
        Ok(())
    })();
    (result, receipt)
}
