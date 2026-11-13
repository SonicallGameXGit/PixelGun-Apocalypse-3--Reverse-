//! PUN Classic 1.60 conventions as the CrazyGames build (Unity 5.3.5) uses them on Photon, and
//! the game's own payloads (`PlayerNetworkController`, the room scripts' RPCs). Every layout here
//! is taken from the recorded session (`docs/protocol.md`, fixture
//! `docs/fixtures/photon-crazygames-frames.txt`).
//!
//! PUN events (all `RaiseEvent` to the other actors; an RPC "to All" also runs locally):
//! - 200 RPC: `{0: viewID, 2: timestamp, 5: RpcList index (byte) | 3: method name, 4: object[] args}`
//! - 201 / 206 serialization (unreliable / reliable): `{0: timestamp, 1: (short) level prefix,
//!   (short) 2+n: {0: viewID, 1: object[] stream}}`
//! - 202 instantiation, cached: `{0: prefab, 1: position, [2: rotation], 6: timestamp,
//!   7: instantiation id = first view id}`
//! - 203 close connection: the master client asks the target to leave (`PhotonNetwork.CloseConnection`)
//! - 204 destroy: `{0: viewID}`

use glam::{Quat, Vec3};

use super::client::AppProfile;
use super::protocol16::Value;
use crate::remote::NetState;
use crate::weapons::SyncFire;

/// The live build on CrazyGames (`files.crazygames.com/webgl/blocky-pixel-apocalypse-3/`): its
/// `PhotonServerSettings` AppID and `gameVersion "16"` + PUN `"1.60"`.
pub const CRAZYGAMES: AppProfile = AppProfile {
    name: "CrazyGames build (Unity 5.3.5, PUN 1.60)",
    app_id: "7f13f7d7-e387-42af-86b9-c353459dc738",
    app_version: "16_1.60",
    name_server: "wss://ns.exitgames.com:19093/",
};

pub const RPC: u8 = 200;
pub const SEND_SERIALIZE: u8 = 201;
pub const INSTANTIATION: u8 = 202;
pub const CLOSE_CONNECTION: u8 = 203;
pub const DESTROY: u8 = 204;
pub const SEND_SERIALIZE_RELIABLE: u8 = 206;
pub const DESTROY_PLAYER: u8 = 207;

/// `PhotonNetwork.MAX_VIEW_IDS`: view ids are `actor * 1000 + n`.
pub const MAX_VIEW_IDS: i32 = 1000;

/// The CrazyGames build's `PhotonServerSettings.RpcList`: an RPC is sent as its index here.
pub const RPC_LIST: [&str; 144] = [
    "Chat",
    "ColorRpc",
    "DestroyRpc",
    "DoDamage",
    "DoJump",
    "DoSwapTeams",
    "InstantiateRpc",
    "Marco",
    "networkAddMessage",
    "OnAwakeRPC",
    "PickupItemInit",
    "Polo",
    "PunPickup",
    "PunPickupSimple",
    "PunRespawn",
    "RequestForPickupTimes",
    "SendChatMessage",
    "SyncWeaponsRPC",
    "TaggedPlayer",
    "AddPlayer",
    "attackAnimation",
    "attackTo",
    "dayTimeUpdate",
    "doDamage",
    "doOverlapDamage",
    "drinkUpdate",
    "driving",
    "eatUpdate",
    "engineUpdate",
    "GetItem",
    "GetNumTag",
    "getRemovedIndex",
    "HPUpdate",
    "interactive",
    "MoveTo",
    "MoveToPosition",
    "OnDead",
    "onVehicle",
    "placingObject",
    "PlayerConnectedCallback",
    "PlayerRegisteration",
    "PositionUpdate",
    "ReceiveFromServer",
    "RequestLoadFromServer",
    "resetAllTrees",
    "SaveToServer",
    "SceneLoadLevel",
    "sendRemovedTreeIndex",
    "toggleFlash",
    "updateAnimation",
    "UpdateHeadRotation",
    "UpdateOtherInventory",
    "UpdatePlayer",
    "UpdatePlayerInfo",
    "UpdatePlayerScore",
    "UpdateRemovedTrees",
    "UpdateScoreBoards",
    "UpdateTransform",
    "UpdateVehicle",
    "activatebeartrap",
    "activelight",
    "addbleed",
    "addkill",
    "addmoney",
    "addmyspawner",
    "addzombiedeath",
    "ask_aparence",
    "askmyid",
    "asksuplly",
    "asktheanimation",
    "asktheplayeranim",
    "askthewave",
    "asktimer",
    "call_for_fort",
    "changecintoitem",
    "changeitem",
    "dead_ask_aparence",
    "dead_sync_aparence",
    "destroy_droploot",
    "destroymeitem",
    "dropitem",
    "endgame",
    "endmissile",
    "fadeanim",
    "fadearmanim",
    "fadeleganim",
    "forcar",
    "get_grab",
    "grabammo",
    "grabammocrate",
    "grabitem",
    "hearnoise",
    "hurt2",
    "hurtbyzombie",
    "hurthead",
    "hurtme",
    "killme",
    "killmebuffered",
    "nasceterra",
    "openbag",
    "playanim",
    "playarmanim",
    "playasound",
    "playitemsound",
    "playleganim",
    "playzombiesound",
    "pushme",
    "receiveammo",
    "receiveammocrate",
    "receivechat",
    "receiveitem",
    "revive",
    "sendchat",
    "senddeadname",
    "sendstuff",
    "servermessage",
    "set_realtime",
    "setbleeding",
    "setcanmove",
    "setcol",
    "setdied",
    "setid",
    "setkillerid",
    "setmyname",
    "setpermission",
    "setposstuff",
    "setreviving",
    "setrot",
    "setskin",
    "setzumbistate",
    "setzumbistuff",
    "spillblood",
    "stun_me",
    "sync_aparence",
    "sync_grab",
    "syncinfo",
    "syncmyid",
    "syncnames",
    "syncwave",
    "updatemypermission",
    "updatename",
    "updateposition",
    "SetOnVehicle",
    "TeleportPosition",
];

/// The scene's `PhotonView` 1 (`__Room`: `WhoKilledWho`), which carries `networkAddMessage`.
pub const ROOM_VIEW: i32 = 1;

/// A PUN event, decoded.
#[derive(Clone, Debug, PartialEq)]
pub enum PunEvent {
    Instantiate {
        view_id: i32,
        prefab: String,
        position: Vec3,
        rotation: Quat,
    },
    Serialize {
        view_id: i32,
        data: Vec<Value>,
    },
    Rpc {
        view_id: i32,
        method: String,
        args: Vec<Value>,
    },
    Destroy {
        view_id: i32,
    },
    /// `DestroyPlayerObjects` of an actor (event 207 `{0: actor}`).
    DestroyPlayer {
        actor: i32,
    },
    /// Event 203: `NetworkingPeer.OnEvent` leaves the room when the sender is the master client
    /// (decompiled from the CrazyGames build, `docs/gameplay/match-flow.md`).
    CloseConnection,
}

/// The actor that owns `view_id` (0 = a scene object).
pub fn owner_of(view_id: i32) -> i32 {
    view_id / MAX_VIEW_IDS
}

fn int_at(content: &Value, key: u8) -> Option<i32> {
    content.get_byte(key).and_then(Value::as_i32)
}

pub fn decode(code: u8, content: &Value) -> Vec<PunEvent> {
    let mut out = Vec::new();
    match code {
        RPC => {
            let Some(view_id) = int_at(content, 0) else {
                return out;
            };
            let method = match content.get_byte(5).and_then(Value::as_i32) {
                Some(index) => RPC_LIST.get(index as usize).map(|s| s.to_string()),
                None => content
                    .get_byte(3)
                    .and_then(Value::as_str)
                    .map(str::to_string),
            };
            let Some(method) = method else {
                return out;
            };
            let args = content
                .get_byte(4)
                .and_then(Value::as_list)
                .map(<[Value]>::to_vec)
                .unwrap_or_default();
            out.push(PunEvent::Rpc {
                view_id,
                method,
                args,
            });
        }
        SEND_SERIALIZE | SEND_SERIALIZE_RELIABLE => {
            // `OnEvent` reads the views from short key 2 on when the level prefix (byte key 1)
            // is present, else from short key 1.
            for (key, view) in content.entries().unwrap_or_default() {
                let Value::Short(index) = key else { continue };
                if *index < 1 {
                    continue;
                }
                let (Some(view_id), Some(data)) =
                    (int_at(view, 0), view.get_byte(1).and_then(Value::as_list))
                else {
                    continue;
                };
                out.push(PunEvent::Serialize {
                    view_id,
                    data: data.to_vec(),
                });
            }
        }
        INSTANTIATION => {
            let (Some(prefab), Some(view_id)) = (
                content.get_byte(0).and_then(Value::as_str),
                int_at(content, 7),
            ) else {
                return out;
            };
            out.push(PunEvent::Instantiate {
                view_id,
                prefab: prefab.to_string(),
                position: content
                    .get_byte(1)
                    .and_then(Value::as_vec3)
                    .unwrap_or(Vec3::ZERO),
                rotation: content
                    .get_byte(2)
                    .and_then(Value::as_quat)
                    .unwrap_or(Quat::IDENTITY),
            });
        }
        DESTROY => {
            if let Some(view_id) = int_at(content, 0) {
                out.push(PunEvent::Destroy { view_id });
            }
        }
        CLOSE_CONNECTION => out.push(PunEvent::CloseConnection),
        DESTROY_PLAYER => {
            if let Some(actor) = int_at(content, 0) {
                out.push(PunEvent::DestroyPlayer { actor });
            }
        }
        _ => {}
    }
    out
}

/// An RPC's content (`PhotonView.RPC`), by RpcList index when the method is listed.
pub fn rpc(view_id: i32, method: &str, args: Vec<Value>, timestamp: i32) -> Value {
    let mut entries = vec![
        (Value::Byte(0), Value::Int(view_id)),
        (Value::Byte(2), Value::Int(timestamp)),
    ];
    match RPC_LIST.iter().position(|m| *m == method) {
        Some(index) => entries.push((Value::Byte(5), Value::Byte(index as u8))),
        None => entries.push((Value::Byte(3), Value::str(method))),
    }
    if !args.is_empty() {
        entries.push((Value::Byte(4), Value::ObjectArray(args)));
    }
    Value::Hashtable(entries)
}

/// One view's `OnPhotonSerializeView` output, as event 201/206 content.
pub fn serialize(view_id: i32, data: Vec<Value>, timestamp: i32) -> Value {
    Value::Hashtable(vec![
        (Value::Byte(0), Value::Int(timestamp)),
        (Value::Byte(1), Value::Short(0)),
        (
            Value::Short(2),
            Value::Hashtable(vec![
                (Value::Byte(0), Value::Int(view_id)),
                (Value::Byte(1), Value::ObjectArray(data)),
            ]),
        ),
    ])
}

/// `PhotonNetwork.Instantiate(prefab, position, rotation, 0)` content (rotation left out when it is
/// the identity, as PUN does).
pub fn instantiate(
    prefab: &str,
    position: Vec3,
    rotation: Quat,
    view_id: i32,
    timestamp: i32,
) -> Value {
    let mut entries = vec![
        (Value::Byte(0), Value::str(prefab)),
        (Value::Byte(1), Value::vec3(position)),
    ];
    if rotation != Quat::IDENTITY {
        entries.push((Value::Byte(2), Value::quat(rotation)));
    }
    entries.push((Value::Byte(6), Value::Int(timestamp)));
    entries.push((Value::Byte(7), Value::Int(view_id)));
    Value::Hashtable(entries)
}

/// Event 204 / the cache-removal filters: `{0: viewID}` or `{7: instantiation id}`.
pub fn view_key(key: u8, view_id: i32) -> Value {
    Value::Hashtable(vec![(Value::Byte(key), Value::Int(view_id))])
}

/// `PlayerNetworkController.OnPhotonSerializeView` (writing).
pub fn state_data(state: &NetState) -> Vec<Value> {
    vec![
        Value::vec3(state.position),
        Value::quat(state.rotation),
        Value::str(&state.name),
        Value::vec3(state.look_target_position),
        Value::quat(state.look_target_rotation),
        Value::str(&state.team),
        Value::str(&state.animation),
        Value::str(&state.hands_animation),
        Value::str(&state.weapon),
        Value::str(&state.animation_type),
        Value::Bool(state.spawn_protected),
    ]
}

/// `PlayerNetworkController.OnPhotonSerializeView` (reading).
pub fn state_of(data: &[Value]) -> Option<NetState> {
    let text = |i: usize| data.get(i).and_then(Value::as_str).map(str::to_string);
    Some(NetState {
        position: data.first()?.as_vec3()?,
        rotation: data.get(1)?.as_quat()?,
        name: text(2)?,
        look_target_position: data.get(3)?.as_vec3()?,
        look_target_rotation: data.get(4)?.as_quat()?,
        team: text(5)?,
        animation: text(6)?,
        hands_animation: text(7)?,
        weapon: text(8)?,
        animation_type: text(9)?,
        spawn_protected: data.get(10)?.as_bool()?,
    })
}

/// `SyncWeaponsRPC(functionName, value)` arguments. This build always sends a float (the knife's
/// `syncKnife` with 0).
pub fn sync_fire_args(sync: SyncFire) -> Vec<Value> {
    let (name, value) = match sync {
        SyncFire::MachineGun(angle) => ("syncMachineGun", angle),
        SyncFire::ShotGun(fractions) => ("syncShotGun", fractions),
        SyncFire::GrenadeLauncher(speed) => ("syncGrenadeLauncher", speed),
        SyncFire::Knife => ("syncKnife", 0.0),
    };
    vec![Value::str(name), Value::Float(value)]
}

pub fn sync_fire_of(args: &[Value]) -> Option<SyncFire> {
    let value = args.get(1).and_then(Value::as_f32).unwrap_or(0.0);
    Some(match args.first()?.as_str()? {
        "syncMachineGun" => SyncFire::MachineGun(value),
        "syncShotGun" => SyncFire::ShotGun(value),
        "syncGrenadeLauncher" => SyncFire::GrenadeLauncher(value),
        "syncKnife" => SyncFire::Knife,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::photon::{Message, get, op, param};

    fn fixture() -> Vec<(String, Message)> {
        include_str!("../../../../docs/fixtures/photon-crazygames-frames.txt")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .map(|line| {
                let (dir, h) = line.split_once(' ').unwrap();
                let bytes: Vec<u8> = (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect();
                (dir.to_string(), Message::decode(&bytes).unwrap())
            })
            .collect()
    }

    /// Content of every recorded outgoing RaiseEvent with code `code`.
    fn sent(code: u8) -> Vec<Value> {
        fixture()
            .into_iter()
            .filter_map(|(dir, m)| match m {
                Message::Request {
                    op: op::RAISE_EVENT,
                    params,
                    ..
                } if dir == "out" && get(&params, param::CODE) == Some(&Value::Byte(code)) => {
                    get(&params, param::DATA).cloned()
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn recorded_streams_decode_to_player_states() {
        let mut states = 0;
        for (_, message) in fixture() {
            let Message::Event { code, params } = message else {
                continue;
            };
            let Some(content) = get(&params, param::DATA) else {
                continue;
            };
            for event in decode(code, content) {
                if let PunEvent::Serialize { view_id, data } = event
                    && view_id >= MAX_VIEW_IDS
                {
                    let state = state_of(&data).expect("an 11-value player stream");
                    assert!(!state.name.is_empty());
                    states += 1;
                }
            }
        }
        assert!(states > 0);
    }

    #[test]
    fn rpcs_encode_as_recorded() {
        for content in sent(RPC) {
            let PunEvent::Rpc {
                view_id,
                method,
                args,
            } = decode(RPC, &content).remove(0)
            else {
                panic!("not an RPC");
            };
            let timestamp = content.get_byte(2).and_then(Value::as_i32).unwrap();
            if content.get_byte(4).is_some() || content.get_byte(5).is_some() {
                assert_eq!(rpc(view_id, &method, args, timestamp), content);
            }
        }
    }

    #[test]
    fn instantiation_and_serialization_encode_as_recorded() {
        let instantiations = sent(INSTANTIATION);
        let spawn = instantiations
            .iter()
            .find(|c| c.get_byte(0).is_some())
            .expect("a recorded Instantiate");
        let PunEvent::Instantiate {
            view_id,
            prefab,
            position,
            rotation,
        } = decode(INSTANTIATION, spawn).remove(0)
        else {
            panic!("not an instantiation");
        };
        let timestamp = spawn.get_byte(6).and_then(Value::as_i32).unwrap();
        assert_eq!(
            instantiate(&prefab, position, rotation, view_id, timestamp),
            *spawn
        );

        for code in [SEND_SERIALIZE, SEND_SERIALIZE_RELIABLE] {
            for content in sent(code) {
                let PunEvent::Serialize { view_id, data } = decode(code, &content).remove(0) else {
                    panic!("not a serialization");
                };
                let state = state_of(&data).unwrap();
                let timestamp = content.get_byte(0).and_then(Value::as_i32).unwrap();
                assert_eq!(serialize(view_id, state_data(&state), timestamp), content);
            }
        }
    }

    #[test]
    fn weapon_sync_round_trips() {
        for sync in [
            SyncFire::MachineGun(3.0),
            SyncFire::ShotGun(4.0),
            SyncFire::GrenadeLauncher(30.0),
            SyncFire::Knife,
        ] {
            assert_eq!(sync_fire_of(&sync_fire_args(sync)), Some(sync));
        }
    }
}
