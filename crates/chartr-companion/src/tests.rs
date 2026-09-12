use super::*;
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, SignatureScheme, StreamOwned,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
struct Pin(String);
impl ServerCertVerifier for Pin {
    fn verify_server_cert(
        &self,
        cert: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if hex(&Sha256::digest(cert)) != self.0 {
            return Err(rustls::Error::General("certificate pin mismatch".into()));
        }
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
fn client(
    server: &Server,
    pin: Option<&str>,
) -> BufReader<StreamOwned<ClientConnection, TcpStream>> {
    let pin = pin.unwrap_or(&server.fingerprint).to_string();
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Pin(pin)))
            .with_no_client_auth();
    let connection =
        ClientConnection::new(Arc::new(config), ServerName::try_from("chartr.local").unwrap())
            .unwrap();
    let socket = TcpStream::connect(server.address).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    BufReader::new(StreamOwned::new(connection, socket))
}
fn request(
    stream: &mut BufReader<StreamOwned<ClientConnection, TcpStream>>,
    token: &str,
    id: u64,
    version: u32,
) -> io::Result<Response> {
    let request = Request { version, token: token.into(), id, operation: Operation::List {} };
    let mut bytes = serde_json::to_vec(&request).unwrap();
    bytes.push(b'\n');
    stream.get_mut().write_all(&bytes)?;
    stream.get_mut().flush()?;
    let mut line = String::new();
    stream.read_line(&mut line)?;
    serde_json::from_str(&line).map_err(io::Error::other)
}
#[test]
fn open_access_keeps_tls_version_and_request_order() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let server = Server::start(
        "127.0.0.1:0".parse().unwrap(),
        Arc::new(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({"spaces":[]}))
        }),
    )
    .unwrap();
    let token = String::new();
    let mut stream = client(&server, None);
    for id in 1..4 {
        let response = request(&mut stream, &token, id, VERSION).unwrap();
        assert_eq!(response.id, id);
        assert!(response.error.is_none());
    }
    assert_eq!(
        request(&mut stream, &token, 4, 99).unwrap().error.as_deref(),
        Some("Unsupported protocol version")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    let mut anonymous = client(&server, None);
    assert!(request(&mut anonymous, "", 5, VERSION).unwrap().error.is_none());
    assert!(request(&mut anonymous, "any legacy value", 6, VERSION).unwrap().error.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 5);
    drop(server);
    assert!(request(&mut stream, &token, 8, VERSION).is_err());
}
#[test]
fn oversized_requests_never_reach_the_host() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let server = Server::start(
        "127.0.0.1:0".parse().unwrap(),
        Arc::new(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(Value::Null)
        }),
    )
    .unwrap();
    let mut stream = client(&server, None);
    let _ = stream.get_mut().write_all(&vec![b'x'; MAX_REQUEST as usize + 2]);
    let _ = stream.get_mut().flush();
    let mut line = String::new();
    assert!(stream.read_line(&mut line).is_err() || line.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn connection_lifetimes_are_independent_and_revoked_on_disconnect() {
    let identities = Arc::new(Mutex::new(Vec::new()));
    let seen = identities.clone();
    let server = Server::start(
        "127.0.0.1:0".parse().unwrap(),
        Arc::new(move |_, alive| {
            seen.lock().unwrap().push(alive);
            Ok(Value::Null)
        }),
    )
    .unwrap();
    let mut first = client(&server, None);
    let mut second = client(&server, None);
    request(&mut first, "", 1, VERSION).unwrap();
    request(&mut second, "", 1, VERSION).unwrap();
    assert_eq!(server.connection_count(), 2);
    let ids = identities.lock().unwrap().clone();
    assert!(!Arc::ptr_eq(&ids[0], &ids[1]));
    drop(first);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while (ids[0].load(Ordering::Acquire) || server.connection_count() != 1)
        && std::time::Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!ids[0].load(Ordering::Acquire));
    assert_eq!(server.connection_count(), 1);
    assert!(ids[1].load(Ordering::Acquire));
    drop(server);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while ids[1].load(Ordering::Acquire) && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!ids[1].load(Ordering::Acquire));
}
