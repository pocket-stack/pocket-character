//! Portable character-widget simulation.
//!
//! The schedulers and state that make an idle character feel alive — blink
//! envelope, eye saccades, look-at smoothing — independent of any renderer
//! or host. Deterministic: seeded RNG, fixed-step friendly, no allocation
//! after construction.
//!
//! The behavior constants replicate airi's VRM stage defaults
//! (`useBlink` / `useIdleEyeSaccades` in moeru-ai/airi `stage-ui-three`),
//! which is the parity reference for pocket-character.

use glam::Vec3;

/// airi: BLINK_DURATION seconds.
const BLINK_DURATION: f32 = 0.2;
/// airi: MIN/MAX_BLINK_INTERVAL seconds, uniform.
const BLINK_INTERVAL: (f32, f32) = (1.0, 6.0);
/// airi: fixation jitter ±0.25 m on the look-at plane.
const SACCADE_JITTER: f32 = 0.25;
/// airi: EYE_SACCADE_INT_STEP ms.
const SACCADE_STEP_MS: f32 = 400.0;
/// airi: cumulative probability rows of `randomSaccadeInterval`; row i maps
/// to base interval 800 + 400·i ms.
const SACCADE_CDF: [f32; 10] = [
    0.075, 0.185, 0.310, 0.450, 0.575, 0.625, 0.665, 0.695, 0.715, 1.0,
];
const SACCADE_BASE_MS: f32 = 800.0;

/// Where the character's eyes take their cue from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackingMode {
    /// Idle saccades around the base look target (airi's VRM default).
    None,
    /// Follow an externally supplied target (cursor tracking).
    Mouse,
}

/// Small deterministic PCG32 so ticks replay identically for a given seed.
#[derive(Clone)]
struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    fn new(seed: u64) -> Self {
        let mut s = Self {
            state: seed.wrapping_add(0x853c_49e6_748f_ea9b),
        };
        s.next_u32();
        s
    }

    fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in [0, 1).
    fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

/// Per-tick outputs the host applies to the rendered character.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimOutputs {
    /// Blink expression weight, 0..1.
    pub blink: f32,
    /// Whether `blink` differs from the previous tick (morph uploads are
    /// worth skipping on the ~95 % of frames where it doesn't).
    pub blink_changed: bool,
    /// World-space point the eyes aim at this tick.
    pub look_target: Vec3,
}

/// The idle-behavior state machine.
pub struct CharacterSim {
    rng: Pcg32,
    pub tracking: TrackingMode,
    /// Base look-at point (parity default: the camera position).
    pub look_base: Vec3,
    /// External look target used when `tracking == Mouse`.
    pub mouse_target: Vec3,

    // Blink (airi useBlink).
    blinking: bool,
    blink_progress: f32,
    time_since_blink: f32,
    next_blink: f32,
    last_blink_weight: f32,

    // Saccades (airi useIdleEyeSaccades).
    time_since_saccade: f32,
    next_saccade: f32,
    fixation: Vec3,
}

impl CharacterSim {
    pub fn new(seed: u64, look_base: Vec3) -> Self {
        let mut rng = Pcg32::new(seed);
        let next_blink = rng.range(BLINK_INTERVAL.0, BLINK_INTERVAL.1);
        Self {
            rng,
            tracking: TrackingMode::None,
            look_base,
            mouse_target: look_base,
            blinking: false,
            blink_progress: 0.0,
            time_since_blink: 0.0,
            next_blink,
            last_blink_weight: 0.0,
            // airi starts with nextSaccadeAfter = -1: first tick fixates.
            time_since_saccade: 0.0,
            next_saccade: -1.0,
            fixation: look_base,
        }
    }

    /// airi randomSaccadeInterval(), in seconds.
    fn saccade_interval(&mut self) -> f32 {
        let r = self.rng.next_f32();
        let row = SACCADE_CDF.iter().position(|&p| r <= p).unwrap_or(9);
        let base = SACCADE_BASE_MS + SACCADE_STEP_MS * row as f32;
        (base + self.rng.next_f32() * SACCADE_STEP_MS) / 1000.0
    }

    pub fn tick(&mut self, dt: f32) -> SimOutputs {
        // --- blink ------------------------------------------------------
        self.time_since_blink += dt;
        if !self.blinking && self.time_since_blink >= self.next_blink {
            self.blinking = true;
            self.blink_progress = 0.0;
        }
        let mut blink = self.last_blink_weight;
        if self.blinking {
            self.blink_progress += dt / BLINK_DURATION;
            blink = (core::f32::consts::PI * self.blink_progress.min(1.0)).sin();
            if self.blink_progress >= 1.0 {
                self.blinking = false;
                self.time_since_blink = 0.0;
                blink = 0.0;
                self.next_blink = self.rng.range(BLINK_INTERVAL.0, BLINK_INTERVAL.1);
            }
        }
        let blink_changed = blink != self.last_blink_weight;
        self.last_blink_weight = blink;

        // --- eyes -------------------------------------------------------
        let look_target = match self.tracking {
            TrackingMode::Mouse => self.mouse_target,
            TrackingMode::None => {
                if self.time_since_saccade >= self.next_saccade {
                    self.fixation = self.look_base
                        + Vec3::new(
                            self.rng.range(-SACCADE_JITTER, SACCADE_JITTER),
                            self.rng.range(-SACCADE_JITTER, SACCADE_JITTER),
                            0.0,
                        );
                    self.time_since_saccade = 0.0;
                    self.next_saccade = self.saccade_interval();
                }
                self.time_since_saccade += dt;
                self.fixation
            }
        };

        SimOutputs {
            blink,
            blink_changed,
            look_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = CharacterSim::new(7, Vec3::new(0.0, 0.0, -1.0));
        let mut b = CharacterSim::new(7, Vec3::new(0.0, 0.0, -1.0));
        for _ in 0..600 {
            let (oa, ob) = (a.tick(1.0 / 60.0), b.tick(1.0 / 60.0));
            assert_eq!(oa.blink.to_bits(), ob.blink.to_bits());
            assert_eq!(oa.look_target, ob.look_target);
        }
    }

    #[test]
    fn blinks_arrive_in_airi_window() {
        let mut sim = CharacterSim::new(42, Vec3::ZERO);
        let mut blinks = 0;
        let mut last_peak = 0.0f32;
        for _ in 0..(60 * 120) {
            let o = sim.tick(1.0 / 60.0);
            if o.blink > last_peak && o.blink > 0.9 {
                blinks += 1;
                last_peak = o.blink;
            }
            if o.blink == 0.0 {
                last_peak = 0.0;
            }
        }
        // 120 s with 1–6 s intervals + 0.2 s blinks → roughly 18–60 blinks.
        assert!((10..=80).contains(&blinks), "blinks = {blinks}");
    }

    #[test]
    fn saccade_intervals_match_table_bounds() {
        let mut sim = CharacterSim::new(1, Vec3::ZERO);
        for _ in 0..200 {
            let s = sim.saccade_interval();
            assert!((0.8..=4.8).contains(&s), "interval {s}");
        }
    }

    #[test]
    fn fixation_stays_in_jitter_box() {
        let base = Vec3::new(0.0, 1.2, -1.0);
        let mut sim = CharacterSim::new(3, base);
        for _ in 0..(60 * 60) {
            let o = sim.tick(1.0 / 60.0);
            assert!((o.look_target - base).abs().max_element() <= SACCADE_JITTER + 1e-6);
        }
    }
}
