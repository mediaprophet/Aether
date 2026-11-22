// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Real-worker qualification harness. Pass an absolute Chora worker executable.
//! Optional second argument: an absolute JS fixture path, served verbatim.
//! Supplemental coverage is a thin page-window adapter, not WPT machinery or
//! dedicated-worker qualification.
//! This isolated loopback fixture denies every host request; it is not an
//! authorized QualiaDB application or a replacement for signed-host qualification.
use aether_chora_client::{
    ChoraClient,
    protocol::{Control, ExecutionProfile, ProtocolError, ReplyError},
};
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const HTML: &str = r#"<!doctype html><html style="height:100%;background:rgb(170,0,0)"><body style="margin:0">
<script>
navigator.chora.request('qualia.graph.snapshot', '{}').then(
  () => { document.documentElement.style.backgroundColor = 'rgb(255,0,255)'; },
  error => { if (String(error) === 'host denied the message')
    document.documentElement.style.backgroundColor = 'rgb(68,85,102)'; }
);
</script></body></html>"#;
const MAX_FIXTURE_BYTES: usize = 64 * 1024;
const CRYPTO_HTML: &str = r#"<!doctype html><html style="height:100%;background:rgb(170,0,0)"><body style="margin:0">
<script>
const fixtureCases = [];
let fixtureCollecting = true;
let fixtureCompleted = 0;
let fixtureHostDenied = false;
let fixtureCryptoPassed = false;
let fixtureFailed = false;
function fixtureFail(error) {
  fixtureFailed = true;
  document.documentElement.style.backgroundColor = 'rgb(255,0,255)';
  console.error('Supplemental page-window crypto fixture failed: ' + String(error));
}
function fixturePaint() {
  if (!fixtureFailed && fixtureHostDenied && fixtureCryptoPassed &&
      fixtureCases.length === 3 && fixtureCompleted === 3)
    document.documentElement.style.backgroundColor = 'rgb(68,85,102)';
}
function promise_test(callback, name) {
  if (!fixtureCollecting || typeof callback !== 'function') {
    fixtureFail('unexpected test registration');
    return;
  }
  fixtureCases.push({callback, name});
}
function assert_equals(actual, expected, message) {
  if (!Object.is(actual, expected)) throw new Error(message || 'assert_equals failed');
}
function assert_true(actual, message) {
  if (actual !== true) throw new Error(message || 'assert_true failed');
}
navigator.chora.request('qualia.graph.snapshot', '{}').then(
  () => fixtureFail('unexpected host authorization'),
  error => {
    if (String(error) !== 'host denied the message') {
      fixtureFail(error);
      return;
    }
    fixtureHostDenied = true;
    fixturePaint();
  }
);
</script>
<script src="/crypto.js" onerror="fixtureFail('crypto.js load failed')"></script>
<script>
fixtureCollecting = false;
(async () => {
  try {
    if (globalThis.isSecureContext !== true || !globalThis.crypto || !globalThis.crypto.subtle)
      throw new Error('secure-context crypto.subtle is required');
    if (fixtureCases.length !== 3) throw new Error('exactly three cases are required');
    for (const test of fixtureCases) {
      await test.callback();
      fixtureCompleted += 1;
    }
    if (fixtureCompleted !== 3 || fixtureCases.length !== 3)
      throw new Error('exactly three completed cases are required');
    fixtureCryptoPassed = true;
    fixturePaint();
  } catch (error) {
    fixtureFail(error);
  }
})();
</script></body></html>"#;

fn load_fixture(path: PathBuf) -> io::Result<Vec<u8>> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "JS fixture path must be absolute",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(MAX_FIXTURE_BYTES + 1)
        .map_err(|_| io::Error::other("fixture allocation failed"))?;
    std::fs::File::open(path)?
        .take((MAX_FIXTURE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FIXTURE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "JS fixture exceeds 64 KiB",
        ));
    }
    Ok(bytes)
}

// Read at most 2 KiB of HTTP headers and spend at most 500ms reading headers.
// The total deadline also bounds slow clients, not just each individual read.
fn request_path(connection: &mut TcpStream) -> Result<String, &'static str> {
    let deadline = Instant::now() + Duration::from_millis(500);
    let mut request = [0; 2048];
    let mut used = 0;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("408 Request Timeout");
        }
        connection
            .set_read_timeout(Some(remaining.min(Duration::from_millis(100))))
            .map_err(|_| "400 Bad Request")?;
        match connection.read(&mut request[used..]) {
            Ok(0) => return Err("400 Bad Request"),
            Ok(count) => used += count,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::Interrupted
                ) =>
            {
                continue;
            }
            Err(_) => return Err("400 Bad Request"),
        }
        if request[..used].windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            break;
        }
        if used == request.len() {
            return Err("431 Request Header Fields Too Large");
        }
    }
    let header = std::str::from_utf8(&request[..used]).map_err(|_| "400 Bad Request")?;
    let mut first_line = header
        .split("\r\n")
        .next()
        .ok_or("400 Bad Request")?
        .split_whitespace();
    let method = first_line.next().ok_or("400 Bad Request")?;
    let path = first_line.next().ok_or("400 Bad Request")?;
    let version = first_line.next().ok_or("400 Bad Request")?;
    if !matches!(version, "HTTP/1.0" | "HTTP/1.1") || first_line.next().is_some() {
        return Err("400 Bad Request");
    }
    if method != "GET" {
        return Err("405 Method Not Allowed");
    }
    Ok(path.to_owned())
}
#[derive(Default)]
struct ServerStats {
    accepted: AtomicUsize,
    root: AtomicUsize,
    malformed: AtomicUsize,
    write_failures: AtomicUsize,
}
struct Server {
    url: String,
    stats: Arc<ServerStats>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(crypto_fixture: Option<Vec<u8>>) -> io::Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let url = format!("http://{}/", listener.local_addr()?);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let stats = Arc::new(ServerStats::default());
        let server_stats = stats.clone();
        let thread = thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut connection, _)) => {
                        server_stats.accepted.fetch_add(1, Ordering::Relaxed);
                        let _ = connection.set_write_timeout(Some(Duration::from_millis(100)));
                        let request = request_path(&mut connection);
                        if request.as_deref() == Ok("/") {
                            server_stats.root.fetch_add(1, Ordering::Relaxed);
                        }
                        if request.is_err() {
                            server_stats.malformed.fetch_add(1, Ordering::Relaxed);
                        }
                        let (status, mime, body): (&str, &str, &[u8]) = match request {
                            Ok(path) if path == "/" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                if crypto_fixture.is_some() {
                                    CRYPTO_HTML.as_bytes()
                                } else {
                                    HTML.as_bytes()
                                },
                            ),
                            Ok(path) if path == "/crypto.js" && crypto_fixture.is_some() => (
                                "200 OK",
                                "application/javascript; charset=utf-8",
                                crypto_fixture.as_deref().unwrap(),
                            ),
                            Ok(_) => ("404 Not Found", "text/plain; charset=utf-8", b"Not found"),
                            Err(status) => {
                                (status, "text/plain; charset=utf-8", b"Invalid request")
                            }
                        };
                        let header = format!(
                            "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nX-Content-Type-Options: nosniff\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        if connection
                            .write_all(header.as_bytes())
                            .and_then(|_| connection.write_all(body))
                            .is_err()
                        {
                            server_stats.write_failures.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            url,
            stats,
            stop,
            thread: Some(thread),
        })
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn probe_error(client: &ChoraClient, reason: &str) -> Box<dyn std::error::Error> {
    let diagnostics = match client.diagnostics() {
        Ok(tail) => format!(
            "stderr_tail(truncated={}, complete={}, read_error={:?}, poisoned={}): {:?}",
            tail.truncated,
            tail.complete,
            tail.read_error,
            tail.lock_poisoned,
            String::from_utf8_lossy(tail.bytes())
        ),
        Err(error) => format!("stderr snapshot unavailable: {error:?}"),
    };
    format!("{reason}; exited={}; {diagnostics}", client.has_exited()).into()
}
fn run(
    client: &ChoraClient,
    url: &str,
    supplemental_crypto: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let deadline = started + Duration::from_secs(30);
    while !client.is_ready() {
        while let Some(event) = client.take_event() {
            match event.message {
                Control::Failed { reason } => {
                    return Err(probe_error(
                        client,
                        &format!("worker startup failure: {reason:?}"),
                    ));
                }
                Control::Stopped => {
                    return Err(probe_error(client, "worker stopped during startup"));
                }
                _ => {}
            }
        }
        if client.has_exited() || Instant::now() >= deadline {
            return Err(probe_error(client, "worker failed to become ready"));
        }
        thread::sleep(Duration::from_millis(10));
    }
    println!(
        "milestone=ready elapsed_ms={}",
        started.elapsed().as_millis()
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    // UI-facing client methods deliberately return Busy instead of waiting.
    loop {
        match client.resize(64, 64) {
            Ok(_) => break,
            Err(ProtocolError::Busy) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(error.into()),
        }
    }
    loop {
        match client.navigate(url.into()) {
            Ok(_) => break,
            Err(ProtocolError::Busy) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(error.into()),
        }
    }
    println!(
        "milestone=resize-and-navigate-enqueued elapsed_ms={}",
        started.elapsed().as_millis()
    );
    let mut first_frame = true;
    let mut capture = 0u64;
    let mut denied = 0;
    let mut pending = None;
    loop {
        if Instant::now() >= deadline {
            return Err(format!("worker bridge/pixel fixture deadline; captures={capture}; denied={denied}; reply_pending={}", pending.is_some()).into());
        }
        while let Some(event) = client.take_event() {
            match event.message {
                Control::PageRequest {
                    id,
                    caller,
                    channel,
                    payload,
                } => {
                    if pending.is_some()
                        || caller.origin != url.trim_end_matches('/')
                        || caller.private_browsing
                        || channel != "qualia.graph.snapshot"
                        || payload != "{}"
                    {
                        return Err("unexpected engine-derived fixture caller/request".into());
                    }
                    println!(
                        "milestone=page-request elapsed_ms={}",
                        started.elapsed().as_millis()
                    );
                    pending = Some((event.stamp, id));
                }
                Control::Failed { reason } => {
                    return Err(format!("worker failure: {reason:?}").into());
                }
                Control::Stopped => return Err("worker stopped during fixture".into()),
                _ => {}
            }
        }
        if client.has_exited() {
            return Err("worker exited during fixture".into());
        }
        if let Some((stamp, id)) = pending {
            match client.reply(stamp, id, Err(ReplyError::Denied)) {
                Ok(()) => {
                    denied += 1;
                    println!(
                        "milestone=host-denial-enqueued elapsed_ms={}",
                        started.elapsed().as_millis()
                    );
                    pending = None;
                }
                Err(ProtocolError::Busy) => {}
                Err(ProtocolError::StaleFrame) => {
                    pending = None;
                }
                Err(error) => return Err(error.into()),
            }
        }
        if let Some(frame) = client.take_frame() {
            if first_frame {
                println!(
                    "milestone=first-frame elapsed_ms={} width={} height={} first_pixel={:?}",
                    started.elapsed().as_millis(),
                    frame.width,
                    frame.height,
                    frame.rgba.get(..4)
                );
                first_frame = false;
            }
            if denied > 0
                && frame.width == 64
                && frame.height == 64
                && frame
                    .rgba
                    .chunks_exact(4)
                    .all(|pixel| pixel == [68, 85, 102, 255])
            {
                if supplemental_crypto {
                    println!(
                        "Actual Chora host denial and exactly three secure-context WebCrypto cases painted expected opaque RGBA pixels (64x64); thin page-window adapter only."
                    );
                } else {
                    println!(
                        "Actual Chora page request denied; JS promise rejection painted expected opaque RGBA pixels (64x64)."
                    );
                }
                return Ok(());
            }
        }
        match client.capture(capture + 1) {
            Ok(()) => capture += 1,
            Err(ProtocolError::Busy) => {}
            Err(error) => return Err(error.into()),
        }
        thread::sleep(Duration::from_millis(15));
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let executable = PathBuf::from(
        arguments
            .next()
            .ok_or("pass an absolute worker executable path")?,
    );
    let crypto_fixture = arguments
        .next()
        .map(PathBuf::from)
        .map(load_fixture)
        .transpose()?;
    if arguments.next().is_some() {
        return Err("usage: worker_readback ABSOLUTE_WORKER [ABSOLUTE_JS_FIXTURE]".into());
    }
    let supplemental_crypto = crypto_fixture.is_some();
    let server = Server::new(crypto_fixture)?;
    let client = ChoraClient::launch(&executable, ExecutionProfile::TrustedEmbedding, false)?;
    let result = run(&client, &server.url, supplemental_crypto)
        .map_err(|error| probe_error(&client, &error.to_string()));
    println!(
        "fixture_http accepted={} root={} malformed={} write_failures={}",
        server.stats.accepted.load(Ordering::Relaxed),
        server.stats.root.load(Ordering::Relaxed),
        server.stats.malformed.load(Ordering::Relaxed),
        server.stats.write_failures.load(Ordering::Relaxed)
    );
    client.shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !client.has_exited() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if !client.has_exited() {
        return Err(probe_error(
            &client,
            "worker not reaped before fixture deadline",
        ));
    }
    result
}
