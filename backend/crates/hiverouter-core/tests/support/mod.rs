//! Test support: in-process server plus a hand-rolled RFC6455 client.
//!
//! Test-only code. The client exists because the workspace deliberately does not
//! add a direct `tokio-tungstenite` dependency (FACT-REGISTER F2: use axum's `ws`
//! feature and let it resolve); it implements just enough of RFC 6455 for the
//! transport tests (masked client text frames, server text/close frames, ping).

#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use hiverouter_core::bus::EventBus;
use hiverouter_core::config::Config;
use hiverouter_core::db::Database;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::http::{self, AppState};
use hiverouter_core::run_checks;
use hiverouter_core::ws::SessionAuth;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

/// Origin allowlisted in the test server.
pub const TEST_ORIGIN: &str = "http://127.0.0.1:9999";
/// Token accepted by the test server.
pub const TEST_TOKEN: &str = "test-session-token";

/// An in-process server with known bus/graph/session handles.
pub struct TestServer {
    /// Bound address.
    pub addr: SocketAddr,
    /// Shared server state (tests may publish events / inspect the graph).
    pub state: Arc<AppState>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    /// Starts a server on an ephemeral loopback port.
    pub async fn start(temp: &tempfile::TempDir) -> Self {
        let config = Arc::new(
            Config::resolve("127.0.0.1:0", temp.path().to_str().expect("utf8"), None)
                .expect("test config"),
        );
        let database = Arc::new(Database::open(config.db_file()).expect("database"));
        database.migrate().expect("migrations");
        let state = Arc::new(AppState::with_services(
            database,
            run_checks(),
            config,
            Arc::new(EventBus::new()),
            Arc::new(GraphAuthority::new()),
            Arc::new(SessionAuth::new(
                TEST_TOKEN.to_owned(),
                vec![TEST_ORIGIN.to_owned()],
            )),
        ));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown, shutdown_rx) = oneshot::channel();
        let serve_state = state.clone();
        let handle = tokio::spawn(async move {
            let _ = http::serve(listener, serve_state, async move {
                let _ = shutdown_rx.await;
            })
            .await;
        });
        Self {
            addr,
            state,
            shutdown: Some(shutdown),
            handle,
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle.abort();
    }
}

async fn read_http_headers(stream: &mut TcpStream) -> Result<(u16, Vec<u8>), String> {
    let mut buffer = Vec::new();
    loop {
        let mut chunk = [0u8; 1024];
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("eof during HTTP handshake".to_owned());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&buffer[..position]).to_owned();
            let leftover = buffer[position + 4..].to_vec();
            let status = headers
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|code| code.parse::<u16>().ok())
                .ok_or_else(|| format!("bad status line: {headers}"))?;
            return Ok((status, leftover));
        }
    }
}

/// Performs the HTTP upgrade handshake and returns the status plus body text for
/// a rejected upgrade (used by the Origin/token tests).
pub async fn upgrade_status(
    addr: SocketAddr,
    path: &str,
    origin: Option<&str>,
    token: &str,
) -> (u16, String) {
    let query_separator = if path.contains('?') { '&' } else { '?' };
    let request_path = format!("{path}{query_separator}token={token}");
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let origin_header = origin.map_or_else(String::new, |value| format!("Origin: {value}\r\n"));
    let request = format!(
        "GET {request_path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n{origin_header}\r\n"
    );
    stream.write_all(request.as_bytes()).await.expect("write handshake");
    let (status, mut leftover) = read_http_headers(&mut stream).await.expect("headers");
    if status != 101 {
        let mut rest = Vec::new();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read_to_end(&mut rest)).await;
        leftover.extend_from_slice(&rest);
    }
    (status, String::from_utf8_lossy(&leftover).into_owned())
}

/// Minimal RFC6455 client.
pub struct WsClient {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl WsClient {
    /// Connects and completes the WebSocket upgrade.
    pub async fn connect(
        addr: SocketAddr,
        origin: &str,
        last_seq: Option<u64>,
    ) -> Result<Self, String> {
        let path = match last_seq {
            Some(seq) => format!("/ws?token={TEST_TOKEN}&last_seq={seq}"),
            None => format!("/ws?token={TEST_TOKEN}"),
        };
        let mut stream = TcpStream::connect(addr).await.map_err(|error| error.to_string())?;
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\nOrigin: {origin}\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|error| error.to_string())?;
        let (status, leftover) = read_http_headers(&mut stream).await?;
        if status != 101 {
            return Err(format!(
                "upgrade rejected with HTTP {status}: {}",
                String::from_utf8_lossy(&leftover)
            ));
        }
        Ok(Self {
            stream,
            buffer: leftover,
        })
    }

    /// Sends the transport hello frame.
    pub async fn hello(&mut self, last_seq: Option<u64>) -> Result<(), String> {
        self.send_text(&serde_json::json!({"type": "hello", "last_seq": last_seq}).to_string())
            .await
    }

    /// Sends a JSON value as a text frame.
    pub async fn send_json(&mut self, value: &Value) -> Result<(), String> {
        self.send_text(&value.to_string()).await
    }

    /// Sends raw text as a text frame.
    pub async fn send_text(&mut self, text: &str) -> Result<(), String> {
        self.write_frame(0x1, text.as_bytes()).await
    }

    /// Receives the next text frame as JSON (ignores ping/pong, errors on close).
    pub async fn recv_json(&mut self) -> Result<Value, String> {
        loop {
            let (opcode, payload) = self.read_frame().await?;
            match opcode {
                0x1 => {
                    return serde_json::from_slice(&payload)
                        .map_err(|error| format!("invalid JSON frame: {error}"))
                }
                0x8 => return Err("connection closed by server".to_owned()),
                0x9 => {
                    self.write_frame(0xA, &payload).await?;
                }
                0xA => {}
                other => return Err(format!("unexpected websocket opcode {other}")),
            }
        }
    }

    /// Sends a close frame (best effort).
    pub async fn close(mut self) {
        let _ = self.write_frame(0x8, &[]).await;
    }

    async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<(), String> {
        let mask = [0x12u8, 0x34, 0x56, 0x78];
        let mut frame = Vec::with_capacity(payload.len() + 14);
        frame.push(0x80 | opcode);
        let len = payload.len();
        if len < 126 {
            frame.push(0x80 | len as u8);
        } else if len <= 0xFFFF {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
        frame.extend_from_slice(&mask);
        for (index, byte) in payload.iter().enumerate() {
            frame.push(byte ^ mask[index % 4]);
        }
        self.stream
            .write_all(&frame)
            .await
            .map_err(|error| error.to_string())
    }

    async fn read_frame(&mut self) -> Result<(u8, Vec<u8>), String> {
        let first = self.read_byte().await?;
        let second = self.read_byte().await?;
        let opcode = first & 0x0F;
        let masked = second & 0x80 != 0;
        let mut len = usize::from(second & 0x7F);
        if len == 126 {
            let bytes = self.read_exact(2).await?;
            len = usize::from(u16::from_be_bytes([bytes[0], bytes[1]]));
        } else if len == 127 {
            let bytes = self.read_exact(8).await?;
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| "bad extended length".to_owned())?;
            len = usize::try_from(u64::from_be_bytes(array))
                .map_err(|_| "frame too large".to_owned())?;
        }
        let mask = if masked {
            Some(self.read_exact(4).await?)
        } else {
            None
        };
        let mut payload = if len == 0 {
            Vec::new()
        } else {
            self.read_exact(len).await?
        };
        if let Some(mask) = mask {
            for (index, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[index % 4];
            }
        }
        Ok((opcode, payload))
    }

    async fn read_byte(&mut self) -> Result<u8, String> {
        self.fill(1).await?;
        Ok(self.buffer.remove(0))
    }

    async fn read_exact(&mut self, count: usize) -> Result<Vec<u8>, String> {
        self.fill(count).await?;
        let rest = self.buffer.split_off(count);
        Ok(std::mem::replace(&mut self.buffer, rest))
    }

    async fn fill(&mut self, count: usize) -> Result<(), String> {
        while self.buffer.len() < count {
            let mut chunk = [0u8; 8192];
            let read = self
                .stream
                .read(&mut chunk)
                .await
                .map_err(|error| error.to_string())?;
            if read == 0 {
                return Err("eof".to_owned());
            }
            self.buffer.extend_from_slice(&chunk[..read]);
        }
        Ok(())
    }
}

/// Receives the next JSON frame with a timeout (5 s).
pub async fn recv_with_timeout(client: &mut WsClient) -> Value {
    tokio::time::timeout(std::time::Duration::from_secs(5), client.recv_json())
        .await
        .expect("timed out waiting for a websocket frame")
        .expect("websocket frame error")
}

/// Receives frames until one satisfies `predicate`.
pub async fn recv_until(client: &mut WsClient, predicate: impl Fn(&Value) -> bool) -> Value {
    for _ in 0..64 {
        let value = recv_with_timeout(client).await;
        if predicate(&value) {
            return value;
        }
    }
    panic!("did not receive the expected frame within 64 frames");
}

/// Name helper for assertion messages.
pub fn frame_name(value: &Value) -> &str {
    value.get("name").and_then(Value::as_str).unwrap_or("<no name>")
}

/// Code helper for ERROR frames.
pub fn error_code(value: &Value) -> &str {
    value
        .get("payload")
        .and_then(|payload| payload.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("<no code>")
}
