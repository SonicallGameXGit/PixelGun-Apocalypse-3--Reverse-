# Multiplayer protocol

## Summary

- **Observed**: the game uses **Photon Unity Networking (PUN) Classic v1.91** (`PhotonNetwork.versionPUN = "1.91"` in `Assembly-CSharp-firstpass`) on top of `Photon3Unity3D.dll` (Photon .NET client library) and hosted **Photon Cloud** (`ServerSettings.HostType = 1 = PhotonCloud`, name server `ns.exitgames.com`).
- The "original servers" are therefore Exit Games' Photon Cloud servers. Our client must speak the **Photon binary protocol** and follow **PUN 1.91 conventions** (event codes, RPC encoding, instantiation, serialization of `PhotonView`s).
- On WebGL the transport is forced to WebSocket(Secure): string literals `ws://{0}:{1}` / `wss://{0}:{1}`, `SocketWebTcpThread` / `SocketWebTcpCoroutine` (**observed**). Photon Cloud also serves UDP/TCP for the same application, and clients on different transports share rooms (**inferred** from Photon's documented behaviour, to verify).

## PhotonServerSettings (`resources.assets`, path_id 40957)

Decoded byte-exact (492/492 bytes) using the `ServerSettings` field layout from the IL2CPP dump.

| Field | Value |
|-------|-------|
| AppID | `3d66e745-64a9-438a-8e21-8012d7b8fbb4` |
| VoiceAppID | (empty) |
| ChatAppID | `4674a0f8-26de-48d8-9a36-7986f63e8658` |
| HostType | 1 (PhotonCloud) |
| PreferredRegion | 0 (eu) |
| EnabledRegions | -1 (all) |
| Protocol | 0 (Udp; overridden on WebGL) |
| ServerAddress / ServerPort | "" / 5055 |
| JoinLobby | true |
| EnableLobbyStatistics | false |
| RunInBackground | true |
| DisableAutoOpenWizard | true |

Note: the string literals `"Returns photon ID"` / `"PHOTON ID HERE"` in the GameArter SDK suggest the platform *may* override the AppID at runtime (**unknown**; verify with a capture).

### RpcList (RPC name shortcuts)

PUN sends an RPC as its **index** in this list when the name is present. Order matters:

| Index | RPC |
|-------|-----|
| 0 | Chat |
| 1 | DestroyRpc |
| 2 | DoDamage |
| 3 | DoSwapTeams |
| 4 | InstantiateRpc |
| 5 | networkAddMessage |
| 6 | PickupItemInit |
| 7 | PunPickup |
| 8 | PunPickupSimple |
| 9 | PunRespawn |
| 10 | RequestForPickupItems |
| 11 | RequestForPickupTimes |
| 12 | SendChatMessage |
| 13 | SetOnVehicle |
| 14 | SyncWeaponsRPC |
| 15 | TeleportPosition |

## Game version / matchmaking

- `ServerChose.GameVersion` in scene `Main` (level0) = **`"PGA3 v1"`**; in scene `Credits` (level4) = `"V15.2"` (likely stale).
- PUN Classic connects with `AppVersion = gameVersion + "_" + versionPUN`, so the expected value is **`"PGA3 v1_1.91"`** (**inferred** from standard PUN 1.91 behaviour; verify in wasm or capture). Photon only matches clients with identical AppId + AppVersion + region.
- Regions offered by `ServerChose`: `us`, `eu`, `au`, `asia`, `jp`, `ru`.

## Open

- Exact connect flow (`ServerChose` / `ConnectMenu` / `RoomMultiplayerMenu`): room properties, custom player properties, lobby usage.
- Per-object serialization (`OnPhotonSerializeView` in `PlayerNetworkController`, `WeaponSync`, …) and RPC parameter types.
- GameArter (`Garter`) backend calls: auth, progress, ranks. Possibly required to play.
- Confirm all of the above with a runtime capture of the original game.
