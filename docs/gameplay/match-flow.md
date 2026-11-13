# Match flow

The lifecycle of a room, from joining to leaving: modes, the round timer, the round end and restart, the kill feed, respawns, late joins, the master client, leaving, kicks and disconnects.

- **Source:** the CrazyGames build (the online target). All function names are from its IL2CPP dump (`research/crazygames-il2cpp-dump/dump.cs`). The asm.js names (`xug`, `Vtg`, ...) are what `tools/re/asmjs_func.py` prints for them.
- **Decompiled bodies:** the excerpts used here are kept in `research/notes/crazygames/` (gitignored):
  - `rmm_roundended.js`, `rmm_restart.js`, `rmm_loadmap.js`, `rmm_waitbeforecheck.js`;
  - `roommultiplayermenu.js` (the whole class);
  - `whokilledwho.js`;
  - `networkingpeer_onevent.js`, `pun_master.js` (PUN 1.60).
- **Labels:** **observed** means read in the decompiled code or seen in a live session. **Inferred** means it follows from the code or from Unity/PUN behaviour, but nobody has compared it with the original yet.

**Client code:**
- `match_flow.rs`: the round (timer, winner, restart).
- `main.rs` (`room_update`, `match_event`, `load_main`): applies the round's events, the leave fade and the return to the menu.
- `online.rs`: the PUN reactions (join, leave, master client, kick).
- `room.rs`: player prefabs and spawn points.
- `room_menu.rs`: the Tab menu, the winner box and the fade.

## Summary

| item | original | client | status |
|---|---|---|---|
| Game modes | `GameMode` room property: `"DM"` or `"TDM"` (from `ConnectMenu`) | both | ported |
| Zombie maps | the 9 `ZM_*` scenes spawn the `Zombie` prefab for team 1 | same | ported |
| Round timer | `RoomMultiplayerMenu.FixedUpdate` (`Vtg`), `WaitBeforeCheck` (`Cug`) | `MatchFlow::fixed_update` / `update` | ported |
| Scoreboard (Tab) | `RoomMultiplayerMenu.OnGUI`, `SortPlayers` (`Ptg`) | `room_menu.rs` | ported earlier (`docs/gameplay/settings.md`) |
| Round end and winner | `RoundEnded` (`xug`) | `final_text`, the box in `room_menu.rs` | ported |
| Restart | `Restart` (`sug`) | `MatchEvent`s handled in `main.rs` | ported |
| Map rotation | none: the room keeps its map forever | none | matches |
| Kill feed | `WhoKilledWho` (`networkAddMessage`, `PlayerLeftRoom` `oBg`, `OnPhotonPlayerDisconnected` `vBg`) | `remote.rs`, `main.rs` | ported, "left" message added |
| Respawn | `PlayerDamage` → `RagdollController` (2 s) → `SpawnPlayer` (`dug`); `Restart` | `damage.rs`, `main.rs` | ported |
| Late join | PUN: cached events, `ResetPhotonViewsOnSerialize` (`uig`) on Join | `online.rs` | **fixed** (the bug below) |
| Master client switch | `HandleEventLeave` (`vig`), `CheckMasterClient` (`xig`) | `online.rs` | ported |
| Leaving | `LeaveRoom` (`fug`) → `OnLeftRoom` (`kug`) → `LoadMap("Main")` (`nug`) with `FadeScreen` (`$tg`) | `main.rs`, `menu.rs` | ported |
| Kick | PUN event 203 `CloseConnection` from the master → `LeaveRoom` | `online.rs` | ported (no game code sends it) |
| Disconnect | `OnDisconnectedFromPhoton` (`jug`) → `LoadMap("Main")` | `main.rs`, `menu.rs` | ported |
| Reconnect | nothing automatic: the player picks a server again | same | matches |

## Game modes

**Observed (`ConnectMenu`, `RoomMultiplayerMenu.Awake`):**
- The room creator picks `GameMode` `"DM"` or `"TDM"`. It is a room property, read in `Awake` into `gameMode`.
- The mode only changes the following:
  - `FixedUpdate`: the ranking runs every fixed step in DM, but only while the menu is open in TDM;
  - `RoundEnded`: the winner text (below);
  - `OnGUI`: the team scores box is shown in TDM only (`docs/gameplay/settings.md`);
  - `SpawnPlayer`: in DM, team 1 spawns at any spawn point of the map; in TDM, at a team-1 point (`docs/gameplay/remote-players.md`);
  - `PlayerNetworkController`: in TDM a teammate's copy gets `disableDamage` and a name tag, so teammates cannot hurt each other; in DM everyone can be hit;
  - the kill message: it carries the killer's team in TDM, which adds to the team score (`docs/gameplay/remote-players.md`).
- Teams exist in both modes: a DM player still picks one of the two teams in the Tab menu.

**Zombie maps (observed):**
- The CrazyGames build has the 9 `ZM_*` scenes. Their `RoomMultiplayerMenu` sets:
  - `playerPrefab` (team 1, `German-ZM`) = `Zombie` (`resources.assets:9329`): 1300 HP, a melee weapon only;
  - `playerPrefabTwo` (team 2, `Mercenary`) = `Mercenarys`.
- On the other maps, team 1 spawns `Mercenarys` and team 2 spawns `German`.
- `SpawnPlayer` instantiates `playerPrefab.name` or `playerPrefabTwo.name` by team, so the network name of a zombie is `"Zombie"`.
- There is no zombie-specific rule in the scripts. "Zombie mode" is TDM or DM on a `ZM_*` map with this prefab.

**Client:**
- `scene.rs` reads `playerPrefab` / `playerPrefabTwo` from the scene's `RoomMultiplayerMenu` (`Scene::room_prefabs`).
- `room.rs` maps them to `PLAYER_PREFABS` (`Mercenarys`, `German`, `Zombie`).
- `online.rs` accepts all three names in instantiations (`prefab_kind`).
- Before this change, the client dropped every `"Zombie"` instantiation, so zombies were invisible and could not be hit.
- Test: `room::tests::zombie_maps_spawn_zombies_and_mercenaries`.

## Round timer

**Observed:**
- `Awake`:
  - `delayAction = true`, then `StartCoroutine(WaitBeforeCheck())`. That coroutine (`Cug`) waits `WaitForSeconds(0.3)` and sets `delayAction = false`.
  - The master client sets `RefTime = (float)PhotonNetwork.time`, `Team1Score = 0` and `Team2Score = 0`. Every client reads `RefTime` into `referenceTime` and `RoundDuration` into `roundDuration` (`docs/protocol.md`, "The master client").
- `FixedUpdate` (`Vtg`): unless `delayAction` is set,
  - `currentRoundTime = roundDuration - (PhotonNetwork.time - referenceTime)`;
  - at or below 0: `currentRoundTime = 0`. If `roundEnded` is not set yet: `StartCoroutine(RoundEnded())` (its first step runs at once), then `isPaused = false` and `roundEnded = true`.
- `OnGUI` draws the time as `mm:ss` at the top (`docs/gameplay/settings.md`).

**Client (`MatchFlow`):**
- `fixed_update` runs **once per frame** when at least one physics step ran in that frame (`main.rs`, `room_update`). The original runs it per physics step, but it only reads properties and the clock, so the result is the same (**inferred**).
- `PhotonNetwork.time` is the server clock (`online.rs`). Offline it is the game clock.
- `--round-time SECONDS` sets `RoundDuration` for games started without the menu (default 1200).
- Test: `match_flow::tests::the_timer_waits_for_the_delay_and_counts_down`.

## Scoreboard (Tab)

Ported earlier and unchanged:
- The Tab menu's player list ("All Players", teams, `Kills`, `Deaths`, `Ping`) is `room_menu.rs`; spec in `docs/gameplay/settings.md`.
- `leadingPlayer` comes from the same `SortPlayers` ranking (`Ptg`): `B.Kills - A.Kills`, so most kills first.
  - `List.Sort` is not stable. The client sorts stably, by actor number first (**inferred**).
  - `SortPlayers` returns 0 when either player has no `Kills`, which no total order reproduces. The client counts a missing `Kills` as 0 (**inferred**).
  - Test: `match_flow::tests::the_leader_has_the_most_kills`.

## Round end and winner

**Observed (`RoundEnded`, `xug`):**
1. `gameMode` is re-read from the room's `GameMode`.
2. `finalText` is set by mode:
   - TDM, `team1Score > team2Score`: `"<team 1 name> Won... Restarting"` (`German-ZM Won... Restarting`);
   - TDM, `team1Score < team2Score`: `"<team 2 name> Won... Restarting"`;
   - TDM, a tie: `"TIMEOUT.. Restarting"`;
   - DM: `"<leadingPlayer> is Winner... Restarting"`.
3. `WaitForSeconds(5)`, then `StartCoroutine(Restart())`.

**`OnGUI` while `roundEnded` (observed):**
- `GUI.color = white`.
- `GUI.Box(Rect(W/2 - 200, H/2 - 100, 400, 30), finalText)`.
- The box is drawn after the TDM scores and before the menu window.

**Client:**
- `match_flow::final_text`, and `RoomView::round_ended` / `final_text` in `room_menu.rs`.
- `MatchEvent::RoundEnded` closes the Tab menu (`isPaused = false`).
- The game keeps running during the 5 s: players can still move, shoot and kill (observed: nothing in `RoundEnded` stops them).
- Test: `match_flow::tests::winner_texts`.

## Restart

**Observed (`Restart`, `sug`), on every client:**
1. Own player properties `Kills = 0`, then `Deaths = 0` (two `SetCustomProperties` calls).
2. The new round's reference time:
   - **master client:**
     - `referenceTime = (float)PhotonNetwork.time`;
     - room properties `RefTime`, then `Team1Score = 0`, then `Team2Score = 0` (three calls);
   - **other clients:** every frame (`yield return null`), compare `referenceTime` with the room's `RefTime`. Once it differs, take it.
3. If the `Player` object exists: `PhotonNetwork.Destroy(Player)`. This is not a death: no ragdoll, no `Deaths`, no kill message.
4. `WaitForSeconds(0.2)`.
5. `SpawnPlayer(TeamName)` unless the player's `TeamName` is `"Spectators"`.
6. `roundEnded = false`, which hides the box.

**Client:**
- `MatchFlow::update` returns `ResetScores`, `NewRound` (master only), `DestroyPlayer` and `Respawn`, and `main.rs` (`match_event`) applies them:
  - `NewRound` uses `OnlineGame::start_round`, the same three properties as at the start of the room.
  - Offline, the dummy players' kills and deaths are reset too. Their bodies are destroyed and spawned again, as their owners would do.
- If `RefTime` is missing, a non-master original throws (unboxing null) and the coroutine ends, so that client never respawns. The client keeps waiting instead (**inferred**, and only possible in rooms without a master's `RefTime`).
- Tests:
  - `match_flow::tests::the_master_restarts_the_round`;
  - `match_flow::tests::other_clients_wait_for_the_masters_reference_time`.

## Map rotation

**Observed:**
- No code changes the map of a room. `LoadMap` is only called with `"Main"`, by `OnLeftRoom` and `OnDisconnectedFromPhoton`.
- `Restart` keeps the scene. The room plays the same map, round after round, until it closes.

**Client:** the same (nothing to port).

## Kill feed

`WhoKilledWho` (view 1) shows the messages sent with the `networkAddMessage` RPC. Its layout is in `docs/gameplay/hud.md`, and the RPC in `docs/protocol.md`.

**Messages (observed):**
- `connected`: sent on joining.
- `joined <team>`: sent on the first spawn.
- `killed`: sent by the victim's owner (`docs/gameplay/damage.md`).
- `fell down`: sent on a death by fall damage.
- `left`:
  - `OnPhotonPlayerDisconnected` (`vBg`) runs on every client, but only the **master client** calls `SendMessage("PlayerLeftRoom", player.name)`;
  - `PlayerLeftRoom` (`oBg`) sends `networkAddMessage(name, "", "left", "")` to `PhotonTargets.All`. The master therefore shows it at once, and the other clients show it when the RPC arrives.

**Leaving the room (observed):**
- `WhoKilledWho.OnLeftRoom` clears the list and disables the component.
- `OnJoinedRoom` / `OnCreatedRoom` enable it again.

**Client:**
- `NetIn::PlayerLeft` (`online.rs`). On the master, `main.rs` sends the RPC and adds the message locally.
- `load_main` clears the feed on leaving. Each room gets a new feed anyway.

## Respawn rules

**Observed (details in `docs/gameplay/damage.md`):**
- **Death:**
  - `PlayerDamage.DoDamage` drops `hp` below 0;
  - the player object is replaced by the death prefab;
  - `RagdollController.RespawnAfter` counts 2 s ("Respawn in: 2", "1");
  - then `RoomMultiplayerMenu.SpawnPlayer(team)`.
- **`SpawnPlayer`** (`dug`):
  - destroys the living player first (`PhotonNetwork.Destroy`), so choosing a team while alive moves the player;
  - sets `TeamName`;
  - instantiates the team's prefab at a spawn point of the team (`docs/gameplay/remote-players.md`);
  - sends `joined <team>` only on the first spawn.
- **Spawn protection** comes from `PlayerNetworkController` (`docs/gameplay/damage.md`).
- **`Restart`** respawns everyone except spectators. The player keeps the team.
- **Spectators** (the team after joining) never spawn until they pick a team in the Tab menu.

**Client:** `respawn()` in `main.rs` now destroys the living player before spawning, as `SpawnPlayer` does.

## Late join

**What PUN does (observed in `NetworkingPeer.OnEvent`, `Vig`):**
- The joining client gets the cached events (the players' `202` instantiations) from the server.
- Event 255 Join, on the clients already in the room:
  - `AddNewPlayer`;
  - `ResetPhotonViewsOnSerialize` (`uig`): it clears every view's last sent stream, so each owner sends its **full** state again on the next serialize tick, even if it has not changed.
- With `UnreliableOnChange`, a player who stands still sends nothing. Without this reset, the newcomer would never get a stream from them.

**The newcomer (observed, `RoomMultiplayerMenu.Awake`):**
- It reads `RefTime`, `RoundDuration` and `GameMode`, so its timer shows the round's remaining time, not a new round.
- It starts as `Spectators` with the menu open.
- It sends `connected`, and `joined <team>` on its first spawn.

**The bug ("someone joins: I can't see or hurt them, they see and kill me"), fixed:**
- **Cause 1:** the server sends the cached instantiations in the same WebSocket batch as the join answer. `OnlineGame::wait_for` returned at the join answer and dropped the rest of the batch. A client that joined late therefore never instantiated the players already in the room.
  - The user saw the opposite side: a player who joins after our client sees everyone, because they get our cached `202`.
- **Cause 2:** our client never reset its serialize state on a Join. When our player stood still, the newcomer got no stream from us.
- **Cause 3:** on the 9 `ZM_*` maps, the `"Zombie"` instantiation was ignored (above).
- **Fix (`online.rs`):**
  - the join batch is kept: events that came with the answer go into a backlog, which `update` processes first;
  - `ClientEvent::ActorJoined` clears `last_state` and `reliable_sent`, as `ResetPhotonViewsOnSerialize` does.
- **Tests:**
  - `online::tests::cached_events_that_came_with_the_join_answer_are_kept`;
  - `online::tests::a_joining_player_gets_the_whole_state_again`.
- **Live check (2026-11-13), two instances of this client on the EU servers:**
  - the second instance received the first one's cached instantiation and stream while the first stood still;
  - the first (master) sent `left` when the second quit.

## Master client switch

**Observed (PUN 1.60, `pun_master.js`):**
- The master client is `mMasterClientId`. `PhotonNetwork.isMasterClient` is true when it equals the local actor.
- **On joining:** the room property byte `248` names the master, when the server sends it ("server-side master client"). Otherwise PUN uses the lowest actor number.
- **Event 254 Leave (`HandleEventLeave`, `vig`):**
  - If the event has param `203` with a nonzero actor, that actor becomes the master (`UpdateMasterClient`).
  - Otherwise, if the room has no server-side master, `CheckMasterClient` (`xig`) runs. When the leaving actor was the master, the lowest remaining actor other than the leaving one becomes the master (the local actor if it is alone), and `OnMasterClientSwitched` is sent.
  - With auto clean-up, the leaver's objects are destroyed (`DestroyPlayerObjects`) and the player is removed. `OnPhotonPlayerDisconnected` then follows.
- **Room property change with `248`:** the master is taken from it.
- **What the game does with the master:** no game script implements `OnMasterClientSwitched`. The master only matters in three places:
  - who sets `RefTime` in `Awake` and `Restart`;
  - who sends `left`;
  - whose `203` is obeyed.

**Client:**
- `OnlineGame::master_id`, `is_master()` and `room_master()`.
- `ClientEvent::ActorLeft { master }` reads param 203.
- Test: `online::tests::the_master_client_follows_the_server_or_the_lowest_actor`.

## Leaving and kicks

**Leaving (observed):**
1. Tab menu, "Leave Room": `LeaveRoom` (`fug`) calls `PhotonNetwork.LeaveRoom()` when connected.
2. The server answers. `OnLeftRoom` (`kug`):
   - `isPaused = false`;
   - `roomCamera.SetActive(true)`;
   - `StartCoroutine(LoadMap("Main"))`.
3. `LoadMap` (`nug`):
   - `isMessageQueueRunning = false`;
   - `fadeDir = 1`;
   - `WaitForSeconds(1)`;
   - `LoadLevelAsync("Main")`.
4. `FadeScreen` (`$tg`, called from `OnGUI` while `fadeDir == 1`):
   - `fadeValue = Clamp01(fadeValue + deltaTime · 15)`;
   - `blackScreen` is drawn full-screen with alpha `fadeValue`;
   - a white `"Loading..."` label is drawn at `(W/2 - 75, H/2 - 15, 150, 30)` (integer division).
5. `Main` loads while still connected, so `ConnectMenu` goes straight to the lobby (`docs/gameplay/menu.md`).

**Other players (observed):** they get event 254 Leave. Their PUN destroys the leaver's objects (auto clean-up, `CleanupCacheOnLeave = true`), and the master sends `left`.

**Kicks (observed):**
- PUN event 203 `CloseConnection` from the master client makes the receiver call `PhotonNetwork.LeaveRoom()`.
- From anyone else it only logs `"Error: Someone else(...) then the masterserver requests a disconnect!"`.
- No game script calls `PhotonNetwork.CloseConnection`, so the official client never kicks anyone. Only a modified client could.

**Client:**
- OpLeave 254 is `Client::leave_room`. Its answer is `ClientEvent::LeftRoom`, which becomes `NetIn::LeftRoom`.
- `main.rs` handles leaving in `load_main`:
  - the room camera is turned on and the fade runs;
  - 1 s later the client returns to the lobby.
- `OnGUI` runs twice per frame (Layout and Repaint, `docs/unknowns.md` #16), so the fade advances by `dt · 30` per frame.
- The kick is `PunEvent::CloseConnection`.
- Offline, "Leave Room" fades back to the menu, or quits when the game was started without the menu.
- Test: `online::tests::a_leaving_player_takes_its_object_with_it`.

## Disconnect and reconnect

**Observed:**
- `OnDisconnectedFromPhoton` (`jug`):
  - prints `"Disconnected from Photon"`;
  - `isPaused = false`;
  - `StartCoroutine(LoadMap("Main"))`, with the same fade as leaving, but the room camera is not re-enabled.
- `Main` then starts unconnected, which is the server window (`ServerChose`).
- **Reconnect:**
  - no script calls `PhotonNetwork.Reconnect` or `ReconnectAndRejoin`;
  - the player reconnects by picking a server again, and rejoins from the lobby;
  - a rejoining player is a new actor with zero scores.

**Client:**
- A socket error or server disconnect becomes `NetIn::Disconnected`. `main.rs` prints the same line and fades out.
- `Menu::return_disconnected` then opens the server window.
- A game started directly with `--online` (without the menu) exits instead, since it has no menu to return to.

## Not ported

- `MultiplayerChat` (T, Enter): the chat RPCs are logged once and ignored (`docs/unknowns.md` #32).
- `SpawnPlayerTwo` exists, but nothing calls it.
- `OnMasterClientSwitched` is sent by PUN, but no game script receives it, so there is nothing to port.

## Open

- A full round with the official client in the room (winner box, restart timing, the respawn after `RefTime` changes): `docs/unknowns.md` #32.
