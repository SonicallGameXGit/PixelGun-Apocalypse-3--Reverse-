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

## Connect flow (decompiled)

Decompiled with `tools/re/` (`wasm_func.py --meta --strings`); function numbers are wasm2c indices. **Observed** from code unless marked.

**Region choice (`ServerChose`, scene `Main`).**
- The buttons `us` / `eu` / `au` / `asia` / `jp` / `ru` call `PhotonNetwork.ConnectToRegion(code, GameVersion)` with `CloudRegionCode` 1 / 0 / 5 / 2 / 3 / 11 (`f29770`..`f29775`).
- `GameVersion` is `"2.0"` in the constructor, overridden to `"PGA3 v1"` by the scene.
- `Update` (`f29769`): once `PhotonNetwork.connected`, it logs `"conected"`, hides `serverdialog` and shows `lob` (the `ConnectMenu` lobby).
- `OnDisconnectedFromPhoton` clears `ConnectMenu.chooseserve` and shows the server dialog again.

**WebSocket URL (Photon3Unity3D, `PeerBase.PepareWebSocketUrl` `f52227`).**
- `wss://<host>:<port>` (`"wss://{0}:{1}"`), followed by the query `app={0}&clientver={1}&sid={2}&{3}&initobj={4}`:
  - `app`: the AppID;
  - `clientver`: the Photon library version `"{0}.{1}.{2}.{3}"`, from the 4 static bytes of `ExitGames.Client.Photon.Version` (value to be read from a capture);
  - `sid`: `ClientSdkId << 1`. `PhotonPeer..ctor` (`f52264`) sets `ClientSdkId = 15`, so `sid=30`;
  - `{3}`: `"IPv6"` when the socket resolved an IPv6 address, else empty;
  - `initobj`: always empty, because no custom init data is passed.
- WebSocket sub-protocol: `"GpBinaryV16"` (string literal; Protocol16).
- Name server host: `ns.exitgames.com`.

**Lobby (`ConnectMenu`, `f65565`..`f65585`).**
- `Start`:
  - default names `"Room " + Random.Range(111, 9999)` and `"Player " + Random.Range(111, 9999)`;
  - `PlayerPrefs` `PlayerName`, `Kills`, `Deaths`, `GamePlays`, `Clan`;
  - max-players options 2/4/10/16/20/22 (default index 4 = 20);
  - round-time options 5/10/15/30 minutes (default index 3 = 30; 600 s if unset);
  - game mode `"TDM"`.
- The room list is `PhotonNetwork.GetRoomList()`, sorted (`ConnectMenu.Sort`).
- Room names that contain `PixelWarfare001v` are this game's rooms. Rooms with the other markers (`PixelWarfare002v`, `--redirecttopaco…`) redirect to PacoGames pages or hide maps.

**Create room (`ShowConnectMenu`, `f65571`).**
- Player name: `playerName + " [" + MyLevel + "]"`, set through `PhotonPlayer.name`.
- Room name: `newRoomName + ": " + gameMode + WEAPONSALLOW + "PixelWarfare001v"`. `WEAPONSALLOW` is the concatenation of the toggled `[WEP02]`..`[WEP06]` tags.
- Room custom properties, all listed as lobby properties in this order:

| key | value |
|---|---|
| `MapName` | the selected map name (string) |
| `RoundDuration` | minutes × 60 (int) |
| `GameMode` | `"TDM"` or `"DM"` (string) |
| `Password` | `ConnectMenu.Password` (static string) |
| `Version` | `VersionChecker` instance field at 0xC (int) |
| `Premium` | `"Premium"` if the room name contains `"1978"`, else `"Normal"` |

- `RoomOptions`: `MaxPlayers` = the chosen option, `IsVisible = true`, `IsOpen = true`, `CleanupCacheOnLeave = true`. Then `PhotonNetwork.CreateRoom(name, options, null)`.
- Join: a room-list entry's `Join` button calls `PhotonNetwork.JoinRoom(name)`. It checks the room's `Version` and `Password` properties first. It also sets `PhotonNetwork.playerName` and saves `PlayerName` / `Clan`.
- `OnJoinedRoom` loads the scene named by the room property `MapName` (`LoadMap` coroutine).
- `OnConnectedToPhoton` leaves any room still joined.

## Runtime observation: the CrazyGames build (2026-11-07)

On 2026-11-07 the PacoGames Build-7 page loaded, but its region buttons did nothing for the user, and no WebSocket was opened. CrazyGames hosts an **older build** of the same game (`files.crazygames.com/webgl/blocky-pixel-apocalypse-3/Release/pixelgunapocalypse3.2.*`, Unity **5.3.5f1**, asm.js, released 2018-03-06). It was started standalone, and a WebSocket hook in the page recorded its traffic (**observed**). Secrets in the frames (auth token, user id) are not reproduced here.

**Not the same Photon application as Build-7:**
- AppID `7f13f7d7-e387-42af-86b9-c353459dc738`, AppVersion `"16_1.60"` (GameVersion `16`, PUN 1.60);
- Build-7: `3d66e745-…`, `"PGA3 v1_1.91"`.

Photon matches only identical AppID + AppVersion, so players of the two builds never meet.

**Connect sequence observed.** The build connected on its own, without input:
1. Name server `wss://ns.exitgames.com:19093/`, sub-protocol `GpBinaryV16`, no URL query (the older library).
2. Master `wss://gcams1128.exitgames.com:19090/`.
3. Game server `wss://gcams2020.exitgames.com:19091/game`.

Each socket was closed with code 1005 once the next one was up.

**Framing:**
- Every WebSocket binary message is one Photon message: `0xF3`, a message type, then the Protocol16 body. No length prefix, no TCP header.
- The server sends the init response `F3 01 00` right after the WebSocket opens. The client sends no init request.
- Ping (internal op request, type 6): `F3 06 01 | 00 01 | 01 69 <int32 client time>`. Response (type 7): `F3 07 01 | 00 00 (return code) | 2A (null debug message) | 00 02 | 01 69 <client time> | 02 69 <server time>`.
- Authenticate (op request, type 2, op 230) to the name server, 3 parameters:

| key | type | value |
|---|---|---|
| 220 | `s` | AppVersion |
| 224 | `s` | AppId |
| 210 | `s` | region (`"eu"`) |

- Its response (type 3, op 230, return 0) has 4 parameters:

| key | value |
|---|---|
| 196 | cluster `"default"` |
| 230 | master address `"wss://GCAMS1128.exitgames.com:19090"` |
| 221 | auth secret |
| 225 | user id (a GUID) |

These match the standard Photon LoadBalancing protocol. Build-7's newer library adds the URL query from `PepareWebSocketUrl`. Whether it also skips the init request is to be confirmed.

## Player object (decompiled)

Details and the receiving side: `docs/gameplay/remote-players.md`.

- **`PlayerNetworkController.OnPhotonSerializeView`** (`f37894`), every 100 ms (`sendIntervalOnSerialize`). The order is:
  1. position `Vector3`
  2. rotation `Quaternion`
  3. name `string`
  4. lookTarget position `Vector3`
  5. lookTarget rotation `Quaternion`
  6. `TeamName` `string`
  7. movement clip `string`
  8. hands clip `string`
  9. active weapon `string`
  10. `animationType` `string`
  11. `IsSpawnProtected` `bool`
- **RPCs** (sent at most every 50 ms by PUN's `sendInterval`):

| RPC | target | parameters |
|---|---|---|
| `SyncWeaponsRPC` | Others | `string functionName`, one value: float error angle, or int `fractions` / `initialSpeed` (boxed) |
| `DoDamage` | All | `float damage`, damager `PhotonPlayer` |
| `networkAddMessage` (`WhoKilledWho`) | All | `string killer, killed, middleText, teamName` |

- Room property `GameMode` (`DM` / `TDM`); player custom properties `TeamName`, `Kills`, `Deaths`; room properties `Team1Score` / `Team2Score`.

## Playing online

The client plays on the CrazyGames build's Photon application (`net/`, `online.rs`), because Build-7's application no longer accepts logins (its custom authentication answers 32755, "(308) Permanent Redirect", observed with `ns_probe` on 2026-11-07).

Everything below was **observed** in the recorded session (`research/captures/crazygames/`, gitignored), unless it says otherwise. The redacted frames are in `docs/fixtures/photon-crazygames-frames.txt`. The client reproduces them byte for byte: `net/photon.rs` and `net/pun.rs` have tests for this.

### LoadBalancing operations

| step | server | request | response / events |
|---|---|---|---|
| Authenticate 230 | name server | `{220 AppVersion, 224 AppId, 210 region}` | `{196 cluster, 230 master address, 221 secret, 225 userId}` |
| Authenticate 230 | master | `{221 secret}` | |
| JoinLobby 229 | master | `{}` | event 230 GameList, then 229 GameListUpdate (param 222: room name → props, `null` props = removed) |
| JoinGame 226 / CreateGame 227 | master | `{255 room name}` (create: `248` game props) | `{230 game server address (…/game), 221 secret}` |
| Authenticate 230 | game server | `{221 secret}` | |
| JoinGame 226 / CreateGame 227 | game server | `{255 name, 249 {Byte255: player name}, 250 true}` (create: also `248`, `241`) | `{254 actor, 249 {actor: props}, 248 room props, 252 actor list}` |
| SetProperties 252 | game server | player: `{251 props, 254 actor, 250 true}`; room: `{251 props, 250 true}` | event 253 `{253 target actor (players only), 251 props}` |
| RaiseEvent 253 | game server | `{244 code, 245 content, [247 cache], [252 target actors]}` | the event, with `254 sender actor` |

The game server's events 255 Join (`{254 actor, 249 props, 252 actors}`) and 254 Leave (`{254 actor}`) maintain the player list.

### PUN 1.60 events (`RaiseEvent` codes)

| code | name | content (Hashtable, byte keys) |
|---|---|---|
| 200 | RPC | `{0: Int viewID, 2: Int timestamp, 5: Byte RpcList index (3: method name when not in the list), 4: ObjectArray args}` |
| 201 / 206 | serialize (unreliable / reliable) | `{0: Int timestamp, 1: Short 0, Short(2+n): {0: Int viewID, 1: ObjectArray stream}}` |
| 202 | instantiate, cached (cache 4) | `{0: prefab, 1: V3 position, [2: Q rotation if not identity], 6: Int timestamp, 7: Int viewID}` |
| 204 | destroy | `{0: Int viewID}` |
| 207 | destroy player | `{0: Int actor}` (sent by the master client for leaving players) |

- **Views and types.** View ids are `actor * 1000 + n`, with n counting up from 1 per actor. The room's scene view (`WhoKilledWho.networkAddMessage`) is view 1. Custom types: `V` (Vector3, 3 big-endian floats), `Q` (Quaternion w, x, y, z), `P` (PhotonPlayer, int actor).
- **Serialization.** `PlayerNetworkController` uses `UnreliableOnChange`. The stream (the 11 values above) goes out as 201 every 100 ms. When it stops changing, it is sent once more as 206, then nothing until it changes again.
- **RpcList.** The CrazyGames build's list has 144 names (in `net/pun.rs`, read from its `PhotonServerSettings`). It differs from Build-7's list, so the indices are only valid for this build.

### What the original client sends

1. After joining:
   - player properties `TeamName = "Spectators"`, `Kills = 0`, `Deaths = 0`;
   - then `networkAddMessage(name, "", "connected", "")` on view 1.
2. On each spawn:
   - `TeamName` (the `RoomMultiplayerMenu` team name: `"German-ZM"` for team 1, `"Mercenary"` for team 2);
   - then the cached instantiation of the team's prefab.
   - The `Resources` names in this build are `"Mercenarys"` (team 1) and `"German"` (team 2).
   - `networkAddMessage(name, "", "joined <team>", "")` is sent only on the first spawn.
3. While alive:
   - the stream;
   - `SyncWeaponsRPC(functionName, value)` to Others. In this build **every value is a float**: the shotgun's fractions and the rocket's speed too, and the knife sends `0.0`.
   - `DoDamage(Float damage, Player damager)` to the target's view, once per `HitBox` the `SendMessageUpwards` chain reaches.
4. On a kill (by the victim's owner):
   - `networkAddMessage(killer, killed, "killed", team)`;
   - the killer's `Kills`;
   - the team score room property `Team1Score` / `Team2Score` (TDM).
5. On death, in order:
   - RaiseEvent 202 `{7: viewID}` with cache 6 (remove) and targets `[own actor]`, which removes the cached instantiation;
   - event 204 `{0: viewID}`;
   - RaiseEvent 200 `{0: viewID}` with cache 6, which removes the view's cached RPCs;
   - then `Deaths`.
6. Every 5 s: the player property `Ping` (`PhotonNetwork.GetPing()`).

### The master client

When the scene loads, `RoomMultiplayerMenu.Awake` on the master client (the lowest actor number) sets the room properties `RefTime = (float)PhotonNetwork.time`, `Team1Score = 0` and `Team2Score = 0` (decompiled from the CrazyGames build, `docs/crazygames-build.md`).
- Every other client reads `RefTime` in its own `Awake`. If it is missing, the unboxing throws and `Awake` stops. **Observed** by the user: the original client showed no menu in a room created by this client before it sent `RefTime`. That this was the cause is inferred from the code, and the user still has to confirm it.
- This client now does the same when it is the master at join time. A second instance of this client then received all three properties.

### Rooms

- Our client lists the lobby's rooms that contain `PixelWarfare001v`, are open and not full, have no password and are on a map we have.
- `--online` joins the most populated of those (`--room TEXT` picks by name). If there is none, it creates one. `--create` always creates.
- A created room uses the `ConnectMenu` format: `"Room N: MODE[WEP02]…[WEP06]PixelWarfare001v"`, max 16 players, props `MapName`, `RoundDuration` 1200, `GameMode`, `Password ""`.
- This matches the CrazyGames build's own `ConnectMenu` (decompiled): the same name format and the same four properties, also used as lobby properties. The original offers 5/10/15/20 minutes; this client uses 20. Unlike Build-7, there is no `Version` property.

## Open

- How the room scripts react to the remaining RPCs (`DoSwapTeams`, chat), and the round timer and round end (`RefTime` is only sent, not used yet).
- GameArter (`Garter`) backend calls of Build-7 (auth, progress, ranks). Not needed on the CrazyGames build, which plays without them.
