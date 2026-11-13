//! The round of a room (spec: `docs/gameplay/match-flow.md`), from the CrazyGames build's
//! `RoomMultiplayerMenu`: the round timer and the leading player of `FixedUpdate`, the
//! `WaitBeforeCheck` delay, and the `RoundEnded` / `Restart` coroutines (winner text, 5 s,
//! scores reset, the master client's new `RefTime`, the player destroyed and spawned again).
//! The game applies what `MatchEvent`s ask for (network, player objects, the menu).

use crate::remote::ActorId;
use crate::room_menu::ROUND_DELAY;
/// `RoundEnded`: `WaitForSeconds(5)` between the winner text and `Restart`.
pub const END_DELAY: f32 = 5.0;
/// `Restart`: `WaitForSeconds(0.2)` between destroying the player and spawning it again.
pub const RESPAWN_DELAY: f32 = 0.2;
/// `finalText` until the first round ends (constructor).
const NO_TEXT: &str = "......";

/// What the round asks of the game.
#[derive(Clone, Debug, PartialEq)]
pub enum MatchEvent {
    /// `FixedUpdate` started `RoundEnded`: `isPaused = false` (the Tab menu closes).
    RoundEnded,
    /// `Restart`: the player properties `Kills = 0`, then `Deaths = 0`.
    ResetScores,
    /// `Restart` on the master client: `RefTime = reference_time`, `Team1Score = 0`,
    /// `Team2Score = 0`.
    NewRound { reference_time: f32 },
    /// `Restart`: `PhotonNetwork.Destroy(Player)` when the player object exists (no death).
    DestroyPlayer,
    /// `Restart`, 0.2 s later: `SpawnPlayer(TeamName)` unless the team is `Spectators`.
    Respawn,
}

/// The room as `RoundEnded` reads it.
pub struct Teams<'a> {
    /// Room property `GameMode`.
    pub game_mode: &'a str,
    pub team_1: &'a str,
    pub team_2: &'a str,
    pub team1_score: i32,
    pub team2_score: i32,
}

/// One entry of `PhotonNetwork.playerList` for `SortPlayers`.
#[derive(Clone, Debug)]
pub struct Ranked<'a> {
    pub actor: ActorId,
    pub name: &'a str,
    /// The `Kills` property (`None`: not set).
    pub kills: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Playing,
    /// `RoundEnded` waits its 5 s.
    Ending {
        resume_at: f32,
    },
    /// `Restart` on a client that is not the master: one check of `RefTime` per frame.
    WaitingForReference,
    /// `Restart` waits its 0.2 s.
    Respawning {
        at: f32,
    },
}

pub struct MatchFlow {
    /// `roundDuration`: room property `RoundDuration` (seconds).
    pub round_duration: i32,
    /// `referenceTime`: the `PhotonNetwork.time` the round started at (room property `RefTime`).
    pub reference_time: f32,
    /// `currentRoundTime`: the seconds left, as the box at the top shows them.
    pub current_round_time: f32,
    /// `roundEnded`: the winner box is shown.
    pub round_ended: bool,
    /// `delayAction`: the timer does not run yet.
    pub delay_action: bool,
    delay_until: f32,
    /// `finalText`.
    pub final_text: String,
    /// `leadingPlayer`: the name at the top of the kills ranking (DM's winner).
    pub leading_player: String,
    phase: Phase,
}

impl MatchFlow {
    /// The constructor and `Awake` at `time` (`Time.time`): `delayAction` until `WaitBeforeCheck`
    /// clears it, the room's duration and reference time.
    pub fn new(round_duration: i32, reference_time: f32, time: f32) -> MatchFlow {
        MatchFlow {
            round_duration,
            reference_time,
            current_round_time: 0.0,
            round_ended: false,
            delay_action: true,
            delay_until: time + ROUND_DELAY,
            final_text: NO_TEXT.to_string(),
            leading_player: String::new(),
            phase: Phase::Playing,
        }
    }

    /// `FixedUpdate` at `time` with the server clock `photon_time` (`PhotonNetwork.time`):
    /// the ranking (while the menu is open, always in DM), then the timer. Returns
    /// `RoundEnded` when the round just ended (the coroutine's first step already ran).
    pub fn fixed_update(
        &mut self,
        time: f32,
        photon_time: f64,
        paused: bool,
        players: &[Ranked],
        teams: &Teams,
    ) -> Option<MatchEvent> {
        if paused || teams.game_mode == "DM" {
            self.leading_player = leading_player(players);
        }
        if self.delay_action {
            return None;
        }
        let left = self.round_duration as f32 - (photon_time as f32 - self.reference_time);
        if left > 0.0 {
            self.current_round_time = left;
            return None;
        }
        self.current_round_time = 0.0;
        if self.round_ended {
            return None;
        }
        self.round_ended = true;
        self.final_text = final_text(teams, &self.leading_player);
        self.phase = Phase::Ending {
            resume_at: time + END_DELAY,
        };
        Some(MatchEvent::RoundEnded)
    }

    /// The coroutines' step of this frame (`WaitBeforeCheck`, `RoundEnded`, `Restart`).
    /// `room_reference` is the room property `RefTime` as this client knows it.
    pub fn update(
        &mut self,
        time: f32,
        photon_time: f64,
        is_master: bool,
        room_reference: Option<f32>,
    ) -> Vec<MatchEvent> {
        let mut events = Vec::new();
        if self.delay_action && time >= self.delay_until {
            self.delay_action = false;
        }
        match self.phase {
            Phase::Playing => {}
            Phase::Ending { resume_at } => {
                if time >= resume_at {
                    // `StartCoroutine(Restart())` runs its first step at once.
                    events.push(MatchEvent::ResetScores);
                    if is_master {
                        self.reference_time = photon_time as f32;
                        events.push(MatchEvent::NewRound {
                            reference_time: self.reference_time,
                        });
                        self.restart_player(time, &mut events);
                    } else {
                        self.check_reference(time, room_reference, &mut events);
                    }
                }
            }
            Phase::WaitingForReference => self.check_reference(time, room_reference, &mut events),
            Phase::Respawning { at } => {
                if time >= at {
                    events.push(MatchEvent::Respawn);
                    self.round_ended = false;
                    self.phase = Phase::Playing;
                }
            }
        }
        events
    }

    /// `Restart` on another client than the master: wait (a frame at a time) until `RefTime`
    /// differs from this round's, then take it. A missing `RefTime` throws in the original
    /// (unboxing null) and ends the coroutine; the client keeps waiting instead.
    fn check_reference(&mut self, time: f32, room: Option<f32>, events: &mut Vec<MatchEvent>) {
        match room {
            Some(reference) if reference != self.reference_time => {
                self.reference_time = reference;
                self.restart_player(time, events);
            }
            _ => self.phase = Phase::WaitingForReference,
        }
    }

    fn restart_player(&mut self, time: f32, events: &mut Vec<MatchEvent>) {
        events.push(MatchEvent::DestroyPlayer);
        self.phase = Phase::Respawning {
            at: time + RESPAWN_DELAY,
        };
    }
}

/// `allPlayers` sorted by `SortPlayers` (`B.Kills - A.Kills`: most kills first), then the first
/// name. `List.Sort` is not stable; ties keep the actor order here, and a missing `Kills` counts
/// as 0 (`SortPlayers` returns 0 for it, which no total order reproduces): both inferred.
pub fn leading_player(players: &[Ranked]) -> String {
    let mut sorted: Vec<&Ranked> = players.iter().collect();
    sorted.sort_by_key(|p| p.actor);
    sorted.sort_by_key(|p| std::cmp::Reverse(p.kills.unwrap_or(0)));
    sorted.first().map_or(String::new(), |p| p.name.to_string())
}

/// `RoundEnded`'s first step: in TDM the team with the higher score won ("TIMEOUT.." on a tie),
/// otherwise the leading player.
pub fn final_text(teams: &Teams, leading_player: &str) -> String {
    if teams.game_mode == "TDM" {
        let (a, b) = (teams.team1_score, teams.team2_score);
        if a > b {
            format!("{} Won... Restarting", teams.team_1)
        } else if a < b {
            format!("{} Won... Restarting", teams.team_2)
        } else {
            "TIMEOUT.. Restarting".to_string()
        }
    } else {
        format!("{leading_player} is Winner... Restarting")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn teams(mode: &'static str, a: i32, b: i32) -> Teams<'static> {
        Teams {
            game_mode: mode,
            team_1: "German-ZM",
            team_2: "Mercenary",
            team1_score: a,
            team2_score: b,
        }
    }

    fn ranked(actor: ActorId, name: &str, kills: Option<i32>) -> Ranked<'_> {
        Ranked { actor, name, kills }
    }

    #[test]
    fn winner_texts() {
        assert_eq!(
            final_text(&teams("TDM", 3, 1), "x"),
            "German-ZM Won... Restarting"
        );
        assert_eq!(
            final_text(&teams("TDM", 0, 2), "x"),
            "Mercenary Won... Restarting"
        );
        assert_eq!(final_text(&teams("TDM", 4, 4), "x"), "TIMEOUT.. Restarting");
        assert_eq!(
            final_text(&teams("DM", 9, 0), "Bob"),
            "Bob is Winner... Restarting"
        );
    }

    #[test]
    fn the_leader_has_the_most_kills() {
        let players = [
            ranked(3, "C", Some(2)),
            ranked(1, "A", Some(5)),
            ranked(2, "B", Some(5)),
            ranked(4, "D", None),
        ];
        assert_eq!(leading_player(&players), "A", "ties keep the actor order");
        assert_eq!(leading_player(&[]), "");
    }

    #[test]
    fn the_timer_waits_for_the_delay_and_counts_down() {
        let mut round = MatchFlow::new(600, 1000.0, 0.0);
        let t = teams("TDM", 0, 0);
        assert_eq!(round.fixed_update(0.1, 1001.0, false, &[], &t), None);
        assert_eq!(round.current_round_time, 0.0, "delayAction: not yet");
        round.update(0.29, 1001.0, true, Some(1000.0));
        assert!(round.delay_action);
        round.update(0.3, 1001.0, true, Some(1000.0));
        assert!(!round.delay_action);
        round.fixed_update(0.32, 1010.5, false, &[], &t);
        assert_eq!(round.current_round_time, 589.5);
    }

    #[test]
    fn the_master_restarts_the_round() {
        let mut round = MatchFlow::new(60, 0.0, 0.0);
        round.update(1.0, 1.0, true, Some(0.0));
        let players = [ranked(1, "A", Some(1)), ranked(2, "B", Some(3))];
        let ended = round.fixed_update(60.0, 60.0, true, &players, &teams("DM", 0, 0));
        assert_eq!(ended, Some(MatchEvent::RoundEnded));
        assert!(round.round_ended);
        assert_eq!(round.final_text, "B is Winner... Restarting");
        assert_eq!(
            round.fixed_update(60.02, 60.02, false, &players, &teams("DM", 0, 0)),
            None,
            "the round ends once"
        );
        assert!(round.update(64.99, 64.99, true, Some(0.0)).is_empty());
        assert_eq!(
            round.update(65.0, 65.0, true, Some(0.0)),
            vec![
                MatchEvent::ResetScores,
                MatchEvent::NewRound {
                    reference_time: 65.0
                },
                MatchEvent::DestroyPlayer,
            ]
        );
        assert!(round.round_ended, "the box stays until the respawn");
        assert!(round.update(65.1, 65.1, true, Some(65.0)).is_empty());
        assert_eq!(
            round.update(65.2, 65.2, true, Some(65.0)),
            vec![MatchEvent::Respawn]
        );
        assert!(!round.round_ended);
        round.fixed_update(65.22, 70.0, false, &players, &teams("DM", 0, 0));
        assert_eq!(round.current_round_time, 55.0);
    }

    #[test]
    fn other_clients_wait_for_the_masters_reference_time() {
        let mut round = MatchFlow::new(60, 0.0, 0.0);
        round.update(1.0, 1.0, false, Some(0.0));
        round.fixed_update(61.0, 61.0, false, &[], &teams("TDM", 2, 1));
        assert_eq!(round.final_text, "German-ZM Won... Restarting");
        assert_eq!(
            round.update(66.0, 66.0, false, Some(0.0)),
            vec![MatchEvent::ResetScores]
        );
        assert!(round.update(66.5, 66.5, false, Some(0.0)).is_empty());
        assert_eq!(
            round.update(67.0, 67.0, false, Some(66.4)),
            vec![MatchEvent::DestroyPlayer]
        );
        assert_eq!(round.reference_time, 66.4);
        assert_eq!(
            round.update(67.2, 67.2, false, Some(66.4)),
            vec![MatchEvent::Respawn]
        );
    }
}
