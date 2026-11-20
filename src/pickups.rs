//! The pickup list: what you just picked up, in a corner of the screen. A repeat of something
//! still listed joins it (its amount grows and its time starts over); entries fade out at the
//! end of their time. Generic over what an entry is, so it needs no game types.

use crate::config::Corner;
use std::time::{Duration, Instant};

/// One listed pickup.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry<T> {
    pub what: T,
    pub amount: u64,
    /// When it was last added to.
    pub at: Instant,
}

/// How long an entry takes to fade out, at the end of its time.
pub const FADE: Duration = Duration::from_millis(700);
/// How long a new entry takes to fade in.
pub const APPEAR: Duration = Duration::from_millis(120);
/// The most entries kept at once, whatever is shown.
const KEEP: usize = 64;

#[derive(Clone, Debug)]
pub struct Log<T> {
    entries: Vec<Entry<T>>,
}

impl<T> Default for Log<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> Log<T> {
    /// Adds `amount` of `what` (joining a listed entry `same` says is the same thing, which then
    /// becomes the newest).
    pub fn add(&mut self, what: T, amount: u64, now: Instant, same: impl Fn(&T, &T) -> bool) {
        if amount == 0 {
            return;
        }
        if let Some(i) = self.entries.iter().position(|e| same(&e.what, &what)) {
            let mut e = self.entries.remove(i);
            e.amount = e.amount.saturating_add(amount);
            e.at = now;
            self.entries.push(e);
        } else {
            self.entries.push(Entry {
                what,
                amount,
                at: now,
            });
        }
        if self.entries.len() > KEEP {
            let extra = self.entries.len() - KEEP;
            self.entries.drain(..extra);
        }
    }

    /// Drops entries whose time (`life`) is over.
    pub fn expire(&mut self, now: Instant, life: Duration) {
        self.entries
            .retain(|e| now.saturating_duration_since(e.at) < life);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The newest `rows` entries, newest first, each with its opacity (0.0 to 1.0).
    pub fn shown(&self, now: Instant, life: Duration, rows: usize) -> Vec<(&Entry<T>, f32)> {
        self.entries
            .iter()
            .rev()
            .filter(|e| now.saturating_duration_since(e.at) < life)
            .take(rows)
            .map(|e| (e, opacity(now.saturating_duration_since(e.at), life)))
            .collect()
    }
}

/// An entry's opacity at `age` of `life`: fading in briefly, then out over the last `FADE`.
pub fn opacity(age: Duration, life: Duration) -> f32 {
    if age >= life {
        return 0.0;
    }
    let left = life - age;
    let out = if left < FADE {
        left.as_secs_f32() / FADE.as_secs_f32()
    } else {
        1.0
    };
    let fade_in = (age.as_secs_f32() / APPEAR.as_secs_f32()).min(1.0);
    (out * fade_in).clamp(0.0, 1.0)
}

/// Row height in GUI pixels, before the list's scale.
pub const ROW: f32 = 18.0;
/// Space kept from the screen's edge.
pub const MARGIN: f32 = 4.0;

/// Where the list starts: the corner point it grows from (the list is scaled about it), whether
/// it grows upward and whether entries are right-aligned. `bottom_clear` is how much of the
/// bottom edge to keep free on that side (the hotbar, the equipment HUD, the chat), `top_clear`
/// the same for the top (the effect icons).
pub fn anchor(corner: Corner, width: f32, height: f32, bottom_clear: f32, top_clear: f32) -> ((f32, f32), bool, bool) {
    match corner {
        Corner::TopLeft => ((MARGIN, MARGIN + top_clear), false, false),
        Corner::TopRight => ((width - MARGIN, MARGIN + top_clear), false, true),
        Corner::BottomLeft => ((MARGIN, height - MARGIN - bottom_clear), true, false),
        Corner::BottomRight => ((width - MARGIN, height - MARGIN - bottom_clear), true, true),
    }
}

/// The amount as shown after a name: `x3`; nothing for one.
pub fn amount_text(amount: u64) -> String {
    if amount > 1 {
        format!("x{amount}")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod check {
    use super::*;

    #[test]
    fn joins_repeats_and_expires() {
        let t0 = Instant::now();
        let mut log: Log<&str> = Log::default();
        log.add("dirt", 3, t0, |a, b| a == b);
        log.add("stone", 1, t0 + Duration::from_millis(10), |a, b| a == b);
        log.add("dirt", 2, t0 + Duration::from_millis(20), |a, b| a == b);
        let life = Duration::from_secs(5);
        let shown = log.shown(t0 + Duration::from_millis(500), life, 10);
        assert_eq!(shown.len(), 2);
        assert_eq!(shown[0].0.what, "dirt");
        assert_eq!(shown[0].0.amount, 5);
        assert_eq!(log.shown(t0 + Duration::from_millis(500), life, 1).len(), 1);
        log.expire(t0 + Duration::from_secs(6), life);
        assert!(log.is_empty());
    }

    #[test]
    fn fades() {
        let life = Duration::from_secs(5);
        assert_eq!(opacity(Duration::ZERO, life), 0.0);
        assert_eq!(opacity(Duration::from_secs(1), life), 1.0);
        let late = opacity(Duration::from_millis(4650), life);
        assert!(late > 0.0 && late < 1.0);
        assert_eq!(opacity(life, life), 0.0);
    }

    #[test]
    fn anchors() {
        let ((x, y), up, right) = anchor(Corner::BottomRight, 400.0, 300.0, 80.0, 0.0);
        assert_eq!((x, y, up, right), (396.0, 216.0, true, true));
        let ((x, y), up, right) = anchor(Corner::TopLeft, 400.0, 300.0, 80.0, 50.0);
        assert_eq!((x, y, up, right), (4.0, 54.0, false, false));
    }
}
