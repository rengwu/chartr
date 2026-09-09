//! Opt-in remote service. The plugin owns the listener; disabling it revokes connections.
use crate::text_input::TextInput;
use chartr_companion::{Operation, Server};
use chartr_plugin::{Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar};
use gpui::{App, AppContext, Context, Entity, Global, Render, Window};
use std::sync::{Arc, atomic::AtomicBool, mpsc};
use ui::prelude::*;

pub struct Call {
    pub operation: Operation,
    pub alive: Arc<AtomicBool>,
    pub deadline: std::time::Instant,
    pub reply: mpsc::SyncSender<Result<serde_json::Value, String>>,
}
#[derive(Clone)]
pub struct Bridge(pub futures::channel::mpsc::Sender<Call>);
impl Global for Bridge {}

pub struct CompanionPlugin {
    state: Entity<State>,
    server: Arc<std::sync::Mutex<Option<Server>>>,
}
pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(CompanionPlugin::new(host, cx))
}
impl Plugin for CompanionPlugin {
    const ID: &'static str = "com.chartr.companion";
    fn new(host: Host, cx: &mut App) -> Self {
        let test_mode = false;
        #[cfg(feature = "companion-test-host")]
        let test_mode = test_mode || std::env::var_os("CHARTR_COMPANION_TEST_READY").is_some();
        let config_path = (!test_mode && !host.data_dir.as_os_str().is_empty())
            .then(|| host.data_dir.join("sharing.json"));
        let saved = config_path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<Sharing>(&bytes).ok())
            .unwrap_or_default();
        let server = Arc::new(std::sync::Mutex::new(None));
        let plugin = Self {
            server: server.clone(),
            state: cx.new(|cx| State {
                server,
                problem: None,
                config_path,
                bridge: cx.try_global::<Bridge>().cloned(),
                address: cx.new(|cx| {
                    let mut field = TextInput::new("Bind IP:port", cx);
                    field.set_text(saved.address, false, cx);
                    field
                }),
            }),
        };
        if saved.enabled {
            plugin.state.update(cx, |state, cx| state.toggle(cx));
        }
        #[cfg(feature = "companion-test-host")]
        plugin.start_test_host(cx);
        plugin
    }
    fn background_status(&self, cx: &App) -> Option<chartr_plugin::BackgroundStatus> {
        use chartr_plugin::{BackgroundState, BackgroundStatus};
        let state = self.state.read(cx);
        let server = self.server.lock().unwrap();
        let (label, detail, status) = if let Some(problem) = &state.problem {
            ("Companion: needs attention".into(), problem.clone(), BackgroundState::Error)
        } else if let Some(server) = server.as_ref() {
            let count = server.connection_count();
            let label = if count == 0 {
                "Companion: sharing".into()
            } else {
                format!("Companion: {count} connected")
            };
            (
                label,
                format!("Sharing on {}. Open Companion controls.", server.address),
                BackgroundState::Running,
            )
        } else {
            (
                "Companion: off".into(),
                "Open Companion controls to start sharing.".into(),
                BackgroundState::Idle,
            )
        };
        Some(BackgroundStatus { label, detail, state: status })
    }

    fn activate(&mut self, registrar: &mut Registrar, _: &mut App) {
        registrar.add_pane("main", "Companion").add_settings();
    }
    fn view(
        &mut self,
        _: &PaneKey,
        _: &InstanceContext,
        _: &mut Window,
        _: &mut App,
    ) -> gpui::AnyView {
        self.state.clone().into()
    }
    fn settings(&mut self, _: &mut Window, _: &mut App) -> Option<gpui::AnyView> {
        Some(self.state.clone().into())
    }
}
impl Drop for CompanionPlugin {
    fn drop(&mut self) {
        // Stop synchronously even if a GPUI view still retains the state entity.
        self.server.lock().unwrap().take();
    }
}

struct State {
    server: Arc<std::sync::Mutex<Option<Server>>>,
    address: Entity<TextInput>,
    problem: Option<String>,
    bridge: Option<Bridge>,
    config_path: Option<std::path::PathBuf>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Sharing {
    enabled: bool,
    address: String,
}
impl Default for Sharing {
    fn default() -> Self {
        Self { enabled: false, address: "0.0.0.0:9847".into() }
    }
}
impl State {
    fn save(&mut self, enabled: bool, cx: &App) {
        let Some(path) = &self.config_path else {
            return;
        };
        let value = Sharing { enabled, address: self.address.read(cx).text().to_string() };
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let temporary = path.with_extension("json.tmp");
            std::fs::write(&temporary, serde_json::to_vec(&value)?)?;
            std::fs::rename(temporary, path)
        })();
        if let Err(error) = result {
            self.problem = Some(format!("Could not remember sharing settings: {error}"));
        }
    }
    fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.server.lock().unwrap().take().is_some() {
            self.save(false, cx);
            cx.notify();
            return;
        }
        self.problem = None;
        let Some(bridge) = self.bridge.clone() else {
            self.problem = Some("Workspace is unavailable.".into());
            cx.notify();
            return;
        };
        let result = self
            .address
            .read(cx)
            .text()
            .parse()
            .map_err(|_| "Use an IP address and port, such as 0.0.0.0:9847.".to_string())
            .and_then(|address| {
                let sender = std::sync::Mutex::new(bridge.0);
                Server::start(
                    address,
                    Arc::new(move |operation, alive| {
                        let (reply, response) = mpsc::sync_channel(1);
                        sender
                            .lock()
                            .unwrap()
                            .try_send(Call {
                                operation,
                                alive,
                                reply,
                                deadline: std::time::Instant::now()
                                    + std::time::Duration::from_secs(4),
                            })
                            .map_err(|_| "Host is busy or closed.".to_string())?;
                        response.recv_timeout(std::time::Duration::from_secs(5)).map_err(|_| {
                            "Host did not respond. Input was not retried.".to_string()
                        })?
                    }),
                )
            });
        match result {
            Ok(server) => {
                *self.server.lock().unwrap() = Some(server);
                self.save(true, cx);
            }
            Err(error) => self.problem = Some(error),
        }
        cx.notify();
    }
}
impl Render for State {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let info = self.server.lock().unwrap().as_ref().map(|s| s.address.to_string());
        v_flex().size_full().p_6().gap_4()
            .child(Label::new("Chartr, within reach").size(LabelSize::Large))
            .child(Label::new("Open Chartr Mobile and enter this computer’s LAN or Tailscale IP and port.").color(Color::Muted))
            .child(self.address.clone())
            .child(Button::new("companion-toggle", if info.is_some() { "Stop sharing" } else { "Start sharing" }).on_click(cx.listener(|this, _, _, cx| this.toggle(cx))))
            .when_some(info, |view, address| view.child(Label::new(format!("Sharing on {address}"))))
            .child(Label::new("Open access: connections need no pairing code. A mobile viewer controls terminal sizing until it leaves or disconnects.").color(Color::Muted))
            .when_some(self.problem.clone(), |view, error| view.child(Label::new(error).color(Color::Error)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn sharing_restarts_without_pairing_and_remembers_an_explicit_stop(
        cx: &mut gpui::TestAppContext,
    ) {
        let temporary = tempfile::tempdir().unwrap();
        let (sender, _receiver) = futures::channel::mpsc::channel(16);
        cx.update(|cx| cx.set_global(Bridge(sender)));
        let create = |cx: &mut App| {
            CompanionPlugin::new(
                Host {
                    data_dir: temporary.path().to_owned(),
                    plugin_dir: std::path::PathBuf::new(),
                },
                cx,
            )
        };
        let first = cx.update(create);
        assert_eq!(
            cx.update(|cx| Plugin::background_status(&first, cx).unwrap().state),
            chartr_plugin::BackgroundState::Idle
        );
        first.state.update(cx, |state, cx| {
            state.address.update(cx, |address, cx| address.set_text("127.0.0.1:0", false, cx));
            state.toggle(cx);
            assert!(state.problem.is_none());
        });
        drop(first);
        let second = cx.update(create);
        assert!(second.server.lock().unwrap().is_some());
        // Status is available after restart without constructing a pane.
        assert_eq!(
            cx.update(|cx| Plugin::background_status(&second, cx).unwrap().state),
            chartr_plugin::BackgroundState::Running
        );
        second.state.update(cx, |state, cx| state.toggle(cx));
        drop(second);
        let third = cx.update(create);
        assert!(third.server.lock().unwrap().is_none());
        assert_eq!(
            cx.update(|cx| Plugin::background_status(&third, cx).unwrap().state),
            chartr_plugin::BackgroundState::Idle
        );
    }

    #[gpui::test]
    fn disabling_plugin_stops_listener_even_when_a_view_survives(cx: &mut gpui::TestAppContext) {
        let (sender, _receiver) = futures::channel::mpsc::channel(16);
        let plugin = cx.update(|cx| {
            cx.set_global(Bridge(sender));
            CompanionPlugin::new(
                Host { data_dir: std::path::PathBuf::new(), plugin_dir: std::path::PathBuf::new() },
                cx,
            )
        });
        let view = plugin.state.clone();
        view.update(cx, |state, cx| {
            state.address.update(cx, |address, cx| address.set_text("127.0.0.1:0", false, cx));
            state.toggle(cx);
            assert!(state.problem.is_none());
            assert!(state.server.lock().unwrap().is_some());
        });
        let server = plugin.server.clone();
        let address = server.lock().unwrap().as_ref().unwrap().address;
        drop(plugin);
        assert!(server.lock().unwrap().is_none());
        // A retained settings/pane view must not keep the network service running.
        assert!(std::net::TcpListener::bind(address).is_ok());
        drop(view);
    }
}

#[cfg(feature = "companion-test-host")]
impl CompanionPlugin {
    /// Explicit integration fixture. This code is excluded from normal builds.
    fn start_test_host(&self, cx: &mut App) {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let Some(path) =
            std::env::var_os("CHARTR_COMPANION_TEST_READY").map(std::path::PathBuf::from)
        else {
            return;
        };
        let root = path.parent().expect("test readiness path needs a parent");
        assert!(
            root.is_absolute()
                && root.file_name().is_some_and(|name| name == "chartr-companion-test"),
            "use an isolated chartr-companion-test directory"
        );
        for name in ["XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME"] {
            assert!(
                std::env::var_os(name)
                    .map(std::path::PathBuf::from)
                    .is_some_and(|path| path.starts_with(root)),
                "test config, data, and state must all be isolated"
            );
        }
        self.state.update(cx, |state, cx| {
            state.address.update(cx, |address, cx| address.set_text("127.0.0.1:19847", false, cx));
            state.toggle(cx);
            assert!(state.problem.is_none(), "{:?}", state.problem);
        });
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .expect("test readiness file must not already exist");
        file.write_all(
            self.server.lock().unwrap().as_ref().unwrap().address.to_string().as_bytes(),
        )
        .unwrap();
    }
}
