//! `FPSSoundController` (footsteps of the local player), decompiled `f28852`..`f28855`
//! (spec: `docs/gameplay/audio.md`).

use serde_json::Value;

use crate::assets::f32_of;
use crate::player::Player;
use crate::unity::Random;

pub struct FpsSoundController {
    pub walk_sounds: Vec<String>,
    pub walk_step_length: f32,
    pub run_step_length: f32,
    pub crouch_step_length: f32,
    /// `lastStep` (initially -10).
    pub last_step: f32,
    /// `StepLenght`: the step length of the previous step's movement mode.
    pub step_length: f32,
}

impl FpsSoundController {
    /// `Awake`: `StepLenght = walkStepLength`.
    pub fn new(fields: &Value) -> FpsSoundController {
        let walk_step_length = f32_of(&fields["walkStepLength"]);
        FpsSoundController {
            walk_sounds: fields["walkSounds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|c| c["asset"].as_str().map(str::to_string))
                .collect(),
            walk_step_length,
            run_step_length: f32_of(&fields["runStepLenght"]),
            crouch_step_length: f32_of(&fields["crouchStepLenght"]),
            last_step: -10.0,
            step_length: walk_step_length,
        }
    }

    /// `FixedUpdate` at fixed time `time`; returns the clip to `Play` on the player's
    /// AudioSource.
    pub fn fixed_update(&mut self, p: &Player, time: f32, random: &mut Random) -> Option<String> {
        let mut played = None;
        if !p.prone {
            if p.walking && p.grounded && !p.crouch {
                played = self.play_step_sounds(time, random).or(played);
                self.step_length = self.walk_step_length;
            }
            if p.running && p.grounded {
                played = self.play_step_sounds(time, random).or(played);
                self.step_length = self.run_step_length;
            }
            if p.walking && p.crouch && p.grounded {
                played = self.play_step_sounds(time, random).or(played);
                self.step_length = self.crouch_step_length;
            }
        }
        played
    }

    /// `PlayStepSounds`: a random walk sound once `StepLenght` has passed since the last one.
    fn play_step_sounds(&mut self, time: f32, random: &mut Random) -> Option<String> {
        if self.step_length == 0.0 || time <= self.step_length + self.last_step {
            return None;
        }
        let count = self.walk_sounds.len() as i32;
        let index = random.range_int(0, count);
        self.last_step = time;
        self.walk_sounds.get(index as usize).cloned()
    }
}
