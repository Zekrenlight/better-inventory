//! The equipment HUD's layout and colours: armour and both hands in columns beside the hotbar,
//! each with what is left of it. Nothing blinks or warns; the colour only follows the wear.

use crate::config::{Sides, WearStyle};

/// One entry's place in the HUD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    MainHand,
    Head,
    Chest,
    OffHand,
    Legs,
    Feet,
}

impl Piece {
    /// Column and row (0 is the top row of three) with both sides used.
    fn split_place(self) -> (bool, u32) {
        match self {
            Self::MainHand => (true, 0),
            Self::Head => (true, 1),
            Self::Chest => (true, 2),
            Self::OffHand => (false, 0),
            Self::Legs => (false, 1),
            Self::Feet => (false, 2),
        }
    }
    fn single_row(self) -> u32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
            Self::Head => 2,
            Self::Chest => 3,
            Self::Legs => 4,
            Self::Feet => 5,
        }
    }
}

/// Row height in GUI pixels.
pub const ROW: i32 = 17;

/// Where an entry goes: its icon's top left, the side its text is on (true: right of the icon)
/// and the text's anchor x (the left edge for text on the right, the right edge otherwise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub icon: (i32, i32),
    pub text_right: bool,
    pub text_x: i32,
}

/// `width`, `height`: the GUI size; `label`: the widest label's width (the column keeps its
/// place whatever the numbers say).
pub fn place(piece: Piece, sides: Sides, width: i32, height: i32, label: i32) -> Place {
    let center = width / 2;
    let (left, row, rows) = match sides {
        Sides::Both => {
            let (l, r) = piece.split_place();
            (l, r, 3)
        }
        Sides::Left => (true, piece.single_row(), 6),
        Sides::Right => (false, piece.single_row(), 6),
    };
    // The rows stand on the hotbar's top edge, the last one lowest.
    let bottom = height - 22 - 2 - 16;
    let y = bottom - (rows - 1 - row as i32) * ROW;
    if left {
        // [icon] 100% |hotbar
        let text_end = center - 91 - 3;
        let icon_x = text_end - label - 2 - 16;
        Place {
            icon: (icon_x, y),
            text_right: true,
            text_x: icon_x + 16 + 2,
        }
    } else {
        // hotbar| 100% [icon]
        let text_start = center + 91 + 3;
        Place {
            icon: (text_start + label + 2, y),
            text_right: false,
            text_x: text_start + label,
        }
    }
}

/// What is left of an item, 0.0 to 1.0.
pub fn left_fraction(damage: i32, max: i32) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    ((max - damage.clamp(0, max)) as f32 / max as f32).clamp(0.0, 1.0)
}

/// The label for `left` of `max` uses; empty for the bar and the plain icon.
pub fn label(style: WearStyle, damage: i32, max: i32) -> String {
    let left = (max - damage.clamp(0, max)).max(0);
    match style {
        WearStyle::Percent => {
            // Rounded down, but never 0% while a use is left.
            let mut pct = (i64::from(left) * 100 / i64::from(max.max(1))) as i32;
            if pct == 0 && left > 0 {
                pct = 1;
            }
            format!("{pct}%")
        }
        WearStyle::Uses => left.to_string(),
        WearStyle::Bar | WearStyle::IconOnly => String::new(),
    }
}

/// Text colour as `[r, g, b]` for what is left: green when fresh, through yellow, to red when
/// nearly worn out. White when colouring is off.
pub fn wear_rgb(fraction: f32, colored: bool) -> [u8; 3] {
    if !colored {
        return [255, 255, 255];
    }
    let f = fraction.clamp(0.0, 1.0);
    let (green, yellow, red) = ([0x55, 0xFF, 0x55], [0xFF, 0xEE, 0x44], [0xFF, 0x4C, 0x4C]);
    let mix = |a: [u8; 3], b: [u8; 3], t: f32| {
        let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
        [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
    };
    if f >= 0.5 {
        mix(yellow, green, (f - 0.5) * 2.0)
    } else {
        mix(red, yellow, f * 2.0)
    }
}

/// The bar style's size.
pub const BAR_W: i32 = 20;
pub const BAR_H: i32 = 3;

#[cfg(test)]
mod check {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(label(WearStyle::Percent, 0, 1561), "100%");
        assert_eq!(label(WearStyle::Percent, 1560, 1561), "1%");
        assert_eq!(label(WearStyle::Uses, 61, 1561), "1500");
        assert_eq!(label(WearStyle::Bar, 0, 10), "");
    }

    #[test]
    fn places_mirror() {
        let l = place(Piece::Head, Sides::Both, 400, 300, 24);
        let r = place(Piece::Legs, Sides::Both, 400, 300, 24);
        assert_eq!(l.icon.1, r.icon.1);
        assert!(l.icon.0 + 16 < 200 - 91);
        assert!(r.icon.0 > 200 + 91);
        let top = place(Piece::MainHand, Sides::Both, 400, 300, 24);
        assert!(top.icon.1 < l.icon.1);
        assert_eq!(place(Piece::Chest, Sides::Both, 400, 300, 24).icon.1, 300 - 40);
    }

    #[test]
    fn colours() {
        assert_eq!(wear_rgb(1.0, true), [0x55, 0xFF, 0x55]);
        assert_eq!(wear_rgb(0.0, true), [0xFF, 0x4C, 0x4C]);
        assert_eq!(wear_rgb(0.3, false), [255, 255, 255]);
    }
}
