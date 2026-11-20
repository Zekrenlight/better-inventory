//! The hotbar's motion: the selection frame gliding between slots, the held item standing a
//! little larger than the rest, and a small squeeze when the held stack is used. All of it is
//! worked out here per frame from the time that passed; the game half only draws it.

use crate::config::Config;
use std::sync::Mutex;
use std::time::Instant;

/// What one hotbar slot (0-8, then 9 for the off hand) holds, enough to see it being used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotLook {
    pub item: u64,
    pub count: u32,
    pub damage: u32,
}

/// Where to draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// The selection frame's place, in slots: 0.0 over the first, 8.0 over the last. With
    /// wrapping it can sit past either end (down to -0.5, up to 8.5): it is then drawn twice,
    /// half off one edge and half on at the other.
    pub selector: f32,
    /// Each slot's item size, 1.0 as the game draws it.
    pub scale: [f32; 10],
}

struct State {
    last: Option<Instant>,
    sel: f32,
    sel_vel: f32,
    scale: [f32; 10],
    scale_vel: [f32; 10],
    pulse: [f32; 10],
    seen: [Option<SlotLook>; 10],
}

static STATE: Mutex<State> = Mutex::new(State {
    last: None,
    sel: 0.0,
    sel_vel: 0.0,
    scale: [1.0; 10],
    scale_vel: [0.0; 10],
    pulse: [0.0; 10],
    seen: [None; 10],
});

/// How long a use squeeze takes, in seconds.
const PULSE_TIME: f32 = 0.2;

fn wrap_into_bar(x: f32) -> f32 {
    let mut x = x;
    while x >= 8.5 {
        x -= 9.0;
    }
    while x < -0.5 {
        x += 9.0;
    }
    x
}

/// A damped spring towards `to`, `omega` its stiffness (radians a second) and `zeta` its damping
/// (1.0 settles without overshoot, lower bounces).
fn spring(x: &mut f32, v: &mut f32, to: f32, omega: f32, zeta: f32, dt: f32) {
    let steps = (dt / (1.0 / 240.0)).ceil().max(1.0) as u32;
    let h = dt / steps as f32;
    for _ in 0..steps {
        let a = omega * omega * (to - *x) - 2.0 * zeta * omega * *v;
        *v += a * h;
        *x += *v * h;
    }
}

/// One frame: `selected` is the selected hotbar slot, `slots` what each slot holds.
pub fn step(now: Instant, selected: usize, slots: [Option<SlotLook>; 10], cfg: &Config) -> Pose {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let selected = selected.min(8);
    let dt = s.last.map_or(1.0, |t| now.saturating_duration_since(t).as_secs_f32());
    s.last = Some(now);
    let lift = 1.0 + cfg.lift_size.clamp(0.0, 50.0) / 100.0;
    let target_scale = |i: usize| if cfg.lift && i == selected { lift } else { 1.0 };
    if dt > 0.5 {
        // First frame, or back after a while: everything in place.
        s.sel = selected as f32;
        s.sel_vel = 0.0;
        for i in 0..10 {
            s.scale[i] = target_scale(i);
            s.scale_vel[i] = 0.0;
            s.pulse[i] = 0.0;
        }
        s.seen = slots;
        return Pose {
            selector: s.sel,
            scale: s.scale,
        };
    }
    let dt = dt.max(0.0);
    // The selection frame.
    if cfg.glide {
        let mut diff = selected as f32 - s.sel;
        if cfg.wrap && diff.abs() > 4.5 {
            diff -= 9.0 * diff.signum();
        }
        let to = s.sel + diff;
        let speed = cfg.glide_speed.clamp(1.0, 10.0);
        if cfg.spring {
            let (mut x, mut v) = (s.sel, s.sel_vel);
            spring(&mut x, &mut v, to, 10.0 + 4.0 * speed, 0.45, dt);
            s.sel = x;
            s.sel_vel = v;
        } else {
            let rate = 5.0 + 3.0 * speed;
            s.sel += diff * (1.0 - (-rate * dt).exp());
            s.sel_vel = 0.0;
        }
        if (to - s.sel).abs() < 0.002 && s.sel_vel.abs() < 0.01 {
            s.sel = to;
            s.sel_vel = 0.0;
        }
        if cfg.wrap {
            s.sel = wrap_into_bar(s.sel);
        }
    } else {
        s.sel = selected as f32;
        s.sel_vel = 0.0;
    }
    // Item sizes.
    for i in 0..10 {
        let to = target_scale(i);
        if cfg.wobble {
            let (mut x, mut v) = (s.scale[i], s.scale_vel[i]);
            spring(&mut x, &mut v, to, 28.0, 0.32, dt);
            s.scale[i] = x;
            s.scale_vel[i] = v;
        } else {
            s.scale[i] += (to - s.scale[i]) * (1.0 - (-16.0 * dt).exp());
            s.scale_vel[i] = 0.0;
        }
    }
    // A used stack (fewer items, or more wear, of the same item) gives a squeeze.
    for i in 0..10 {
        if let (Some(was), Some(now)) = (s.seen[i], slots[i])
            && was.item == now.item
            && (now.count < was.count || now.damage > was.damage)
            && cfg.use_pulse
        {
            s.pulse[i] = 1.0;
        }
        s.pulse[i] = (s.pulse[i] - dt / PULSE_TIME).max(0.0);
    }
    s.seen = slots;
    let mut scale = s.scale;
    for (i, v) in scale.iter_mut().enumerate() {
        let squeeze = (std::f32::consts::PI * s.pulse[i]).sin();
        *v *= 1.0 - 0.16 * squeeze;
        if !v.is_finite() || *v <= 0.0 {
            *v = 1.0;
        }
    }
    if !s.sel.is_finite() {
        s.sel = selected as f32;
        s.sel_vel = 0.0;
    }
    Pose {
        selector: s.sel,
        scale,
    }
}

#[cfg(test)]
mod check {
    use super::*;
    use std::time::Duration;

    #[test]
    fn glides_and_settles() {
        let cfg = Config::default();
        let t0 = Instant::now();
        let p = step(t0, 0, [None; 10], &cfg);
        assert_eq!(p.selector, 0.0);
        let mut t = t0;
        let mut last = 0.0;
        for _ in 0..120 {
            t += Duration::from_millis(16);
            last = step(t, 3, [None; 10], &cfg).selector;
        }
        assert!((last - 3.0).abs() < 0.01, "{last}");
        // 8 -> 0 wraps: the frame goes right, off the edge.
        for _ in 0..60 {
            t += Duration::from_millis(16);
            step(t, 8, [None; 10], &cfg);
        }
        t += Duration::from_millis(16);
        let p = step(t, 0, [None; 10], &cfg);
        assert!(p.selector > 8.0 || p.selector < 0.5, "{}", p.selector);
    }
}
