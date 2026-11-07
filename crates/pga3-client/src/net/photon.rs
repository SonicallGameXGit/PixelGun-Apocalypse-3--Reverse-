//! Photon messages over WebSocket: every binary WebSocket message is one Photon message, `0xF3`, a
//! message type, then its Protocol16 body (no length prefix, no TCP header). Observed in the
//! recorded CrazyGames session (`docs/protocol.md`).

use anyhow::{Result, bail};

use super::protocol16::{self, Reader, Value};

pub const MAGIC: u8 = 0xF3;

const INIT_RESPONSE: u8 = 1;
const REQUEST: u8 = 2;
const RESPONSE: u8 = 3;
const EVENT: u8 = 4;
const INTERNAL_REQUEST: u8 = 6;
const INTERNAL_RESPONSE: u8 = 7;
/// Bit 7 of the message type marks an encrypted message (never used by the game).
const ENCRYPTED: u8 = 0x80;

/// Photon LoadBalancing operation codes.
pub mod op {
    /// Internal: the ping (`PhotonPeer` keep-alive and time sync).
    pub const PING: u8 = 1;
    pub const JOIN_GAME: u8 = 226;
    pub const CREATE_GAME: u8 = 227;
    pub const JOIN_LOBBY: u8 = 229;
    pub const AUTHENTICATE: u8 = 230;
    pub const SET_PROPERTIES: u8 = 252;
    pub const RAISE_EVENT: u8 = 253;
}

/// Photon LoadBalancing parameter codes.
pub mod param {
    pub const REGION: u8 = 210;
    pub const APP_VERSION: u8 = 220;
    pub const SECRET: u8 = 221;
    pub const GAME_LIST: u8 = 222;
    pub const APP_ID: u8 = 224;
    pub const ADDRESS: u8 = 230;
    pub const CLEANUP_CACHE_ON_LEAVE: u8 = 241;
    pub const CODE: u8 = 244;
    pub const DATA: u8 = 245;
    pub const CACHE: u8 = 247;
    pub const GAME_PROPERTIES: u8 = 248;
    pub const PLAYER_PROPERTIES: u8 = 249;
    pub const BROADCAST: u8 = 250;
    pub const PROPERTIES: u8 = 251;
    pub const ACTOR_LIST: u8 = 252;
    pub const TARGET_ACTOR: u8 = 253;
    pub const ACTOR_NR: u8 = 254;
    pub const ROOM_NAME: u8 = 255;
}

/// Photon LoadBalancing event codes (PUN's own are in `pun.rs`).
pub mod event {
    pub const GAME_LIST_UPDATE: u8 = 229;
    pub const GAME_LIST: u8 = 230;
    pub const PROPERTIES_CHANGED: u8 = 253;
    pub const LEAVE: u8 = 254;
    pub const JOIN: u8 = 255;
}

/// Byte keys of game (room) properties.
pub mod game_key {
    pub const MAX_PLAYERS: u8 = 255;
    pub const IS_OPEN: u8 = 254;
    pub const IS_VISIBLE: u8 = 253;
    pub const PLAYER_COUNT: u8 = 252;
    pub const REMOVED: u8 = 251;
    pub const PROPS_LISTED_IN_LOBBY: u8 = 250;
}

/// Byte key of the player name in player properties.
pub const PLAYER_NAME_KEY: u8 = 255;

/// RaiseEvent cache options (`EventCaching`).
pub mod cache {
    pub const ADD_TO_ROOM_CACHE: u8 = 4;
    pub const REMOVE_FROM_ROOM_CACHE: u8 = 6;
}

pub type Params = Vec<(u8, Value)>;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// The server's answer to the connection (sent right after the WebSocket opens).
    InitResponse,
    Request {
        internal: bool,
        op: u8,
        params: Params,
    },
    Response {
        internal: bool,
        op: u8,
        code: i16,
        debug: Option<String>,
        params: Params,
    },
    Event {
        code: u8,
        params: Params,
    },
}

pub fn get(params: &Params, key: u8) -> Option<&Value> {
    params.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}

impl Message {
    pub fn request(op: u8, params: Params) -> Message {
        Message::Request {
            internal: false,
            op,
            params,
        }
    }

    pub fn decode(data: &[u8]) -> Result<Message> {
        if data.len() < 2 || data[0] != MAGIC {
            bail!("not a Photon message ({} bytes)", data.len());
        }
        if data[1] & ENCRYPTED != 0 {
            bail!("encrypted Photon message (type {})", data[1] & !ENCRYPTED);
        }
        let mut r = Reader::new(&data[2..]);
        let message = match data[1] {
            INIT_RESPONSE => return Ok(Message::InitResponse),
            kind @ (REQUEST | INTERNAL_REQUEST) => Message::Request {
                internal: kind == INTERNAL_REQUEST,
                op: r.byte()?,
                params: r.params()?,
            },
            kind @ (RESPONSE | INTERNAL_RESPONSE) => {
                let op = r.byte()?;
                let code = r.short()?;
                let debug = match r.typed()? {
                    Value::String(s) => Some(s),
                    _ => None,
                };
                Message::Response {
                    internal: kind == INTERNAL_RESPONSE,
                    op,
                    code,
                    debug,
                    params: r.params()?,
                }
            }
            EVENT => Message::Event {
                code: r.byte()?,
                params: r.params()?,
            },
            other => bail!("unsupported Photon message type {other}"),
        };
        if !r.at_end() {
            bail!(
                "{} trailing bytes after a Photon message",
                data.len() - 2 - r.pos
            );
        }
        Ok(message)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![MAGIC];
        match self {
            Message::InitResponse => out.extend([INIT_RESPONSE, 0]),
            Message::Request {
                internal,
                op,
                params,
            } => {
                out.push(if *internal { INTERNAL_REQUEST } else { REQUEST });
                out.push(*op);
                protocol16::write_params(&mut out, params);
            }
            Message::Response {
                internal,
                op,
                code,
                debug,
                params,
            } => {
                out.push(if *internal {
                    INTERNAL_RESPONSE
                } else {
                    RESPONSE
                });
                out.push(*op);
                out.extend(code.to_be_bytes());
                match debug {
                    Some(text) => protocol16::write_typed(&mut out, &Value::str(text)),
                    None => out.push(protocol16::NULL),
                }
                protocol16::write_params(&mut out, params);
            }
            Message::Event { code, params } => {
                out.push(EVENT);
                out.push(*code);
                protocol16::write_params(&mut out, params);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Recorded frames: decode, then encode back to the same bytes (`docs/fixtures/`).
    #[test]
    fn recorded_frames_round_trip() {
        let fixture = include_str!("../../../../docs/fixtures/photon-crazygames-frames.txt");
        let mut count = 0;
        for line in fixture.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let bytes = hex(line.split_whitespace().last().unwrap());
            let message = Message::decode(&bytes).unwrap_or_else(|e| panic!("{line}: {e}"));
            assert_eq!(message.encode(), bytes, "{message:?}");
            count += 1;
        }
        assert!(count > 20);
    }

    #[test]
    fn ping_request_layout() {
        // `F3 06 01 | 00 01 | 01 69 <int>`, as recorded.
        let ping = Message::Request {
            internal: true,
            op: op::PING,
            params: vec![(1, Value::Int(0xFFFDF322_u32 as i32))],
        };
        assert_eq!(ping.encode(), hex("f3060100010169fffdf322"));
    }
}
