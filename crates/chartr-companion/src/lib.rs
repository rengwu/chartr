//! Bounded, open-access TLS RPC for the Chartr companion. No GUI dependencies.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub const VERSION: u32 = 1;
pub const MAX_REQUEST: u64 = 64 * 1024;
pub const MAX_RESPONSE: usize = 4 * 1024 * 1024;

mod history;
pub use history::History;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    List {},
    Screen {
        space: String,
        session: String,
    },
    Watch {
        space: String,
        session: String,
        columns: u16,
        rows: u16,
    },
    Release {
        space: String,
        session: String,
    },
    History {
        space: String,
        session: String,
        #[serde(default)]
        snapshot: Option<String>,
        #[serde(default)]
        offset: usize,
        #[serde(default)]
        known: Option<String>,
    },
    Input {
        space: String,
        session: String,
        data: String,
    },
    Paste {
        space: String,
        session: String,
        data: String,
    },
    Submit {
        space: String,
        session: String,
        data: String,
    },
    Focus {
        space: String,
        session: Option<String>,
    },
    Create {
        space: String,
    },
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    #[serde(default)]
    pub token: String,
    pub id: u64,
    pub operation: Operation,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<String>,
}
pub type Handler = dyn Fn(Operation, Arc<AtomicBool>) -> Result<Value, String> + Send + Sync;

pub struct Server {
    pub address: SocketAddr,
    /// Exposed for transport diagnostics; open-access clients do not need to pin it.
    pub fingerprint: String,
    alive: Arc<AtomicBool>,
    clients: Arc<Mutex<std::collections::HashMap<u64, TcpStream>>>,
    listener: Option<thread::JoinHandle<()>>,
}
impl Server {
    pub fn connection_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    pub fn start(address: SocketAddr, handler: Arc<Handler>) -> Result<Self, String> {
        let identity = rcgen::generate_simple_self_signed(vec!["chartr.local".into()])
            .map_err(|e| e.to_string())?;
        let cert = identity.cert.der().clone();
        let fingerprint = hex(&Sha256::digest(&cert));
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_no_client_auth()
            .with_single_cert(
                vec![cert],
                rustls::pki_types::PrivatePkcs8KeyDer::from(identity.signing_key.serialize_der())
                    .into(),
            )
            .map_err(|e| e.to_string())?;
        let listener = TcpListener::bind(address).map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let alive = Arc::new(AtomicBool::new(true));
        let clients = Arc::new(Mutex::new(std::collections::HashMap::<u64, TcpStream>::new()));
        let running = alive.clone();
        let sockets = clients.clone();
        let config = Arc::new(config);
        let listener = thread::spawn(move || {
            let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();
            let mut next_client = 0u64;
            while running.load(Ordering::Acquire) {
                workers.retain(|worker| !worker.is_finished());
                match listener.accept() {
                    Ok((socket, _)) => {
                        if workers.len() >= 4 {
                            continue;
                        }
                        if socket.set_nonblocking(false).is_err() {
                            continue;
                        }
                        let _ = socket.set_read_timeout(Some(Duration::from_secs(5)));
                        let _ = socket.set_write_timeout(Some(Duration::from_secs(5)));
                        let _ = socket.set_nodelay(true);
                        next_client += 1;
                        let client_id = next_client;
                        if let Ok(copy) = socket.try_clone() {
                            sockets.lock().unwrap().insert(client_id, copy);
                        }
                        let active_sockets = sockets.clone();
                        let (config, handler, running) =
                            (config.clone(), handler.clone(), running.clone());
                        workers.push(thread::spawn(move || {
                            let outcome = serve(socket, config, handler, running);
                            #[cfg(test)]
                            if let Err(error) = &outcome {
                                eprintln!("companion connection: {error}");
                            }
                            let _ = outcome;
                            active_sockets.lock().unwrap().remove(&client_id);
                        }));
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self { address, fingerprint, alive, clients, listener: Some(listener) })
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
        for (_, socket) in self.clients.lock().unwrap().drain() {
            let _ = socket.shutdown(Shutdown::Both);
        }
        if let Some(listener) = self.listener.take() {
            let _ = listener.join();
        }
    }
}
fn serve(
    socket: TcpStream,
    config: Arc<rustls::ServerConfig>,
    handler: Arc<Handler>,
    alive: Arc<AtomicBool>,
) -> io::Result<()> {
    struct Connected(Arc<AtomicBool>);
    impl Drop for Connected {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }
    let connected = Connected(Arc::new(AtomicBool::new(true)));
    let conn = rustls::ServerConnection::new(config).map_err(io::Error::other)?;
    let mut stream = BufReader::new(rustls::StreamOwned::new(conn, socket));
    while alive.load(Ordering::Acquire) {
        let mut line = Vec::new();
        let n = stream.by_ref().take(MAX_REQUEST + 1).read_until(b'\n', &mut line)?;
        if n == 0 {
            break;
        }
        if n as u64 > MAX_REQUEST || line.last() != Some(&b'\n') {
            return Err(io::Error::other("request too large"));
        }
        let request: Request = serde_json::from_slice(&line).map_err(io::Error::other)?;
        let result = if request.version != VERSION {
            Err("Unsupported protocol version".into())
        } else if !alive.load(Ordering::Acquire) {
            Err("Companion stopped".into())
        } else {
            handler(request.operation, connected.0.clone())
        };
        let response = match result {
            Ok(value) => Response { id: request.id, result: Some(value), error: None },
            Err(error) => Response { id: request.id, result: None, error: Some(error) },
        };
        let mut bytes = serde_json::to_vec(&response).map_err(io::Error::other)?;
        if bytes.len() > MAX_RESPONSE {
            return Err(io::Error::other("response too large"));
        }
        bytes.push(b'\n');
        stream.get_mut().write_all(&bytes)?;
        stream.get_mut().flush()?;
    }
    Ok(())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_rejects_unknown_operations_and_fields() {
        assert!(serde_json::from_str::<Operation>(r#"{"op":"exec","command":"sh"}"#).is_err());
        assert!(serde_json::from_str::<Operation>(r#"{"op":"list","unexpected":true}"#).is_err());
    }
    #[test]
    fn stop_releases_listener_and_rotates_identity() {
        let server =
            Server::start("127.0.0.1:0".parse().unwrap(), Arc::new(|_, _| Ok(Value::Null)))
                .unwrap();
        let address = server.address;
        let fingerprint = server.fingerprint.clone();
        drop(server);
        let next = Server::start(address, Arc::new(|_, _| Ok(Value::Null))).unwrap();
        assert_ne!(fingerprint, next.fingerprint);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod network_tests;
