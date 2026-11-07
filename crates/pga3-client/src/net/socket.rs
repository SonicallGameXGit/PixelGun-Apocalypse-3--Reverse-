//! One secure WebSocket to a Photon server (`wss://host:port/path`, sub-protocol `GpBinaryV16`),
//! run on its own thread. The thread also sends Photon's internal ping once a second, as the
//! original client does on every connection, which keeps the connection alive while the game
//! loads a scene and gives the round-trip time and the server clock offset (PUN's
//! `ServerTimestamp`).

use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tungstenite::client::IntoClientRequest;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message as WsMessage, WebSocket};

use super::photon::{Message, get, op};
use super::protocol16::Value;

/// Seconds between pings (the original's `timePingInterval` is 1000 ms).
const PING_INTERVAL: Duration = Duration::from_millis(1000);
/// How long a read waits before the thread looks at its outgoing queue again.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Debug)]
pub enum SocketEvent {
    Message(Message),
    /// The connection ended; the text says why.
    Closed(String),
}

/// Clock state shared with the socket thread.
#[derive(Default)]
struct Clock {
    /// Last round-trip time in milliseconds (-1 before the first ping answer).
    rtt: AtomicI32,
    /// Server time minus local time, in milliseconds.
    offset: AtomicI32,
}

pub struct Socket {
    outgoing: Sender<Vec<u8>>,
    incoming: Receiver<SocketEvent>,
    clock: Arc<Clock>,
    stop: Arc<AtomicBool>,
    epoch: Instant,
}

impl Socket {
    /// Starts connecting; `epoch` is the client's clock origin (`Environment.TickCount`).
    pub fn open(url: &str, epoch: Instant) -> Socket {
        let (outgoing, outgoing_rx) = channel::<Vec<u8>>();
        let (incoming_tx, incoming) = channel();
        let clock = Arc::new(Clock::default());
        clock.rtt.store(-1, Ordering::Relaxed);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_url = url.to_string();
        let thread_clock = clock.clone();
        let thread_stop = stop.clone();
        thread::spawn(move || {
            let reason = match run(
                &thread_url,
                &outgoing_rx,
                &incoming_tx,
                &thread_clock,
                &thread_stop,
                epoch,
            ) {
                Ok(reason) => reason,
                Err(error) => format!("{error:#}"),
            };
            let _ = incoming_tx.send(SocketEvent::Closed(reason));
        });
        Socket {
            outgoing,
            incoming,
            clock,
            stop,
            epoch,
        }
    }

    pub fn send(&self, message: &Message) {
        let _ = self.outgoing.send(message.encode());
    }

    /// Everything received since the last call.
    pub fn poll(&self) -> Vec<SocketEvent> {
        let mut events = Vec::new();
        loop {
            match self.incoming.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    events.push(SocketEvent::Closed("socket thread ended".to_string()));
                    break;
                }
            }
        }
        events
    }

    /// `PhotonNetwork.GetPing()`: the last round-trip time in milliseconds.
    pub fn rtt(&self) -> i32 {
        self.clock.rtt.load(Ordering::Relaxed).max(0)
    }

    /// `PhotonNetwork.ServerTimestamp`: the server's millisecond clock (wrapping `int`).
    pub fn server_timestamp(&self) -> i32 {
        local_ticks(self.epoch).wrapping_add(self.clock.offset.load(Ordering::Relaxed))
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Milliseconds since `epoch`, as a wrapping `int` like `Environment.TickCount`.
fn local_ticks(epoch: Instant) -> i32 {
    epoch.elapsed().as_millis() as u64 as i32
}

fn tcp(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Option<&mut TcpStream> {
    match ws.get_mut() {
        MaybeTlsStream::Plain(stream) => Some(stream),
        MaybeTlsStream::Rustls(stream) => Some(stream.get_mut()),
        _ => None,
    }
}

fn run(
    url: &str,
    outgoing: &Receiver<Vec<u8>>,
    incoming: &Sender<SocketEvent>,
    clock: &Clock,
    stop: &AtomicBool,
    epoch: Instant,
) -> Result<String> {
    // Rustls needs a process-wide crypto provider; a second install fails harmlessly.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut request = url
        .into_client_request()
        .with_context(|| format!("bad Photon address {url}"))?;
    request
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", "GpBinaryV16".parse()?);
    let (mut ws, _) =
        tungstenite::connect(request).with_context(|| format!("connecting to {url}"))?;
    if let Some(stream) = tcp(&mut ws) {
        stream.set_read_timeout(Some(POLL_INTERVAL))?;
        stream.set_nodelay(true)?;
    }
    // Pings start once the server's init response arrived, as in the recorded session.
    let mut next_ping: Option<Instant> = None;
    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = ws.close(None);
            let _ = ws.flush();
            return Ok("closed by the client".to_string());
        }
        let mut wrote = false;
        loop {
            match outgoing.try_recv() {
                Ok(bytes) => {
                    ws.write(WsMessage::Binary(bytes.into()))?;
                    wrote = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = ws.close(None);
                    return Ok("closed by the client".to_string());
                }
            }
        }
        if let Some(at) = next_ping
            && Instant::now() >= at
        {
            next_ping = Some(at + PING_INTERVAL);
            let ping = Message::Request {
                internal: true,
                op: op::PING,
                params: vec![(1, Value::Int(local_ticks(epoch)))],
            };
            ws.write(WsMessage::Binary(ping.encode().into()))?;
            wrote = true;
        }
        if wrote {
            ws.flush()?;
        }
        match ws.read() {
            Ok(WsMessage::Binary(data)) => match Message::decode(&data) {
                Ok(Message::InitResponse) => {
                    next_ping.get_or_insert_with(Instant::now);
                    if incoming
                        .send(SocketEvent::Message(Message::InitResponse))
                        .is_err()
                    {
                        return Ok("receiver dropped".to_string());
                    }
                }
                Ok(Message::Response {
                    internal: true,
                    op: op::PING,
                    params,
                    ..
                }) => {
                    let sent = get(&params, 1).and_then(Value::as_i32);
                    let server = get(&params, 2).and_then(Value::as_i32);
                    if let (Some(sent), Some(server)) = (sent, server) {
                        let now = local_ticks(epoch);
                        let rtt = now.wrapping_sub(sent);
                        clock.rtt.store(rtt, Ordering::Relaxed);
                        clock.offset.store(
                            server.wrapping_add(rtt / 2).wrapping_sub(now),
                            Ordering::Relaxed,
                        );
                    }
                }
                Ok(message) => {
                    if incoming.send(SocketEvent::Message(message)).is_err() {
                        return Ok("receiver dropped".to_string());
                    }
                }
                Err(error) => eprintln!("photon: undecodable message from {url}: {error:#}"),
            },
            Ok(WsMessage::Close(frame)) => {
                return Ok(match frame {
                    Some(frame) => {
                        format!("closed by the server ({}: {})", frame.code, frame.reason)
                    }
                    None => "closed by the server".to_string(),
                });
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(tungstenite::Error::ConnectionClosed) => {
                return Ok("connection closed".to_string());
            }
            Err(error) => return Err(error.into()),
        }
    }
}
