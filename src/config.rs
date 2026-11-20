//! The settings: their keys, defaults and the copy the game half reads.

use std::sync::Mutex;

pub const ID: &str = "better_inventory";

/// The sort key's id (Options > Controls).
pub const SORT_KEY: &str = "sort";
/// Its default key.
pub const SORT_KEY_DEFAULT: &str = "key.keyboard.r";

// Sorting
pub const KEY_SORT_BUTTONS: &str = "sort_buttons";
pub const KEY_SORT_ORDER: &str = "sort_order";
// Refill
pub const KEY_REFILL: &str = "refill";
pub const KEY_REFILL_OFFHAND: &str = "refill_offhand";
pub const KEY_REFILL_SIMILAR: &str = "refill_similar_tools";
// Equipment HUD
pub const KEY_HUD: &str = "hud";
pub const KEY_HUD_ARMOR: &str = "hud_armor";
pub const KEY_HUD_MAIN_HAND: &str = "hud_main_hand";
pub const KEY_HUD_OFF_HAND: &str = "hud_off_hand";
pub const KEY_HUD_PLAIN_ITEMS: &str = "hud_plain_items";
pub const KEY_HUD_STYLE: &str = "hud_style";
pub const KEY_HUD_COLOR: &str = "hud_color";
pub const KEY_HUD_SIDES: &str = "hud_sides";
// Hotbar motion
pub const KEY_GLIDE: &str = "selector_glide";
pub const KEY_GLIDE_SPEED: &str = "selector_glide_speed";
pub const KEY_SPRING: &str = "selector_spring";
pub const KEY_WRAP: &str = "selector_wrap";
pub const KEY_LIFT: &str = "selected_lift";
pub const KEY_LIFT_SIZE: &str = "selected_lift_size";
pub const KEY_WOBBLE: &str = "item_wobble";
pub const KEY_USE_PULSE: &str = "use_pulse";
// Pickup list
pub const KEY_PICKUPS: &str = "pickups";
pub const KEY_PICKUP_CORNER: &str = "pickup_corner";
pub const KEY_PICKUP_XP: &str = "pickup_xp";
pub const KEY_PICKUP_NAMES: &str = "pickup_names";
pub const KEY_PICKUP_TIME: &str = "pickup_time";
pub const KEY_PICKUP_ROWS: &str = "pickup_rows";
pub const KEY_PICKUP_SCALE: &str = "pickup_scale";

/// How sorted stacks are ordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    /// The creative inventory's order: building blocks, then colours, nature, tools and so on.
    Groups,
    /// By name, A to Z.
    Name,
    /// Most items first, then by group.
    Amount,
}

impl SortOrder {
    pub const OPTIONS: [&'static str; 3] = [
        "better_inventory.sort_order.groups",
        "better_inventory.sort_order.name",
        "better_inventory.sort_order.amount",
    ];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Name,
            2 => Self::Amount,
            _ => Self::Groups,
        }
    }
}

/// How the equipment HUD shows what is left of an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WearStyle {
    /// `87%`.
    Percent,
    /// A small bar.
    Bar,
    /// The uses left: `1452`.
    Uses,
    /// The item alone.
    IconOnly,
}

impl WearStyle {
    pub const OPTIONS: [&'static str; 4] = [
        "better_inventory.hud_style.percent",
        "better_inventory.hud_style.bar",
        "better_inventory.hud_style.uses",
        "better_inventory.hud_style.icon_only",
    ];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Bar,
            2 => Self::Uses,
            3 => Self::IconOnly,
            _ => Self::Percent,
        }
    }
}

/// Which side of the hotbar the equipment HUD uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sides {
    /// Main hand, helmet and chestplate on the left; off hand, leggings and boots on the right.
    Both,
    /// Everything in one column on the left.
    Left,
    /// Everything in one column on the right.
    Right,
}

impl Sides {
    pub const OPTIONS: [&'static str; 3] = [
        "better_inventory.hud_sides.both",
        "better_inventory.hud_sides.left",
        "better_inventory.hud_sides.right",
    ];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Left,
            2 => Self::Right,
            _ => Self::Both,
        }
    }
}

/// The screen corner the pickup list sits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const OPTIONS: [&'static str; 4] = [
        "better_inventory.corner.top_left",
        "better_inventory.corner.top_right",
        "better_inventory.corner.bottom_left",
        "better_inventory.corner.bottom_right",
    ];
    /// The default's index in `OPTIONS`.
    pub const DEFAULT: usize = 3;
    pub fn from_index(i: u32) -> Self {
        match i {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            _ => Self::BottomRight,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub sort_buttons: bool,
    pub sort_order: SortOrder,
    pub refill: bool,
    pub refill_offhand: bool,
    pub refill_similar: bool,
    pub hud: bool,
    pub hud_armor: bool,
    pub hud_main_hand: bool,
    pub hud_off_hand: bool,
    pub hud_plain_items: bool,
    pub hud_style: WearStyle,
    pub hud_color: bool,
    pub hud_sides: Sides,
    pub glide: bool,
    /// 1 (slow) to 10 (snappy).
    pub glide_speed: f32,
    pub spring: bool,
    pub wrap: bool,
    pub lift: bool,
    /// Percent larger than the other slots' items.
    pub lift_size: f32,
    pub wobble: bool,
    pub use_pulse: bool,
    pub pickups: bool,
    pub pickup_corner: Corner,
    pub pickup_xp: bool,
    pub pickup_names: bool,
    /// Seconds an entry stays.
    pub pickup_time: f32,
    pub pickup_rows: u32,
    /// Percent of the normal size.
    pub pickup_scale: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sort_buttons: true,
            sort_order: SortOrder::Groups,
            refill: true,
            refill_offhand: true,
            refill_similar: true,
            hud: true,
            hud_armor: true,
            hud_main_hand: true,
            hud_off_hand: true,
            hud_plain_items: true,
            hud_style: WearStyle::Percent,
            hud_color: true,
            hud_sides: Sides::Both,
            glide: true,
            glide_speed: 5.0,
            spring: false,
            wrap: true,
            lift: true,
            lift_size: 15.0,
            wobble: true,
            use_pulse: true,
            pickups: true,
            pickup_corner: Corner::BottomRight,
            pickup_xp: true,
            pickup_names: true,
            pickup_time: 5.0,
            pickup_rows: 6,
            pickup_scale: 100.0,
        }
    }
}

impl Config {
    /// Whether the hotbar is drawn by this mod (any motion setting on).
    pub fn moves_hotbar(&self) -> bool {
        self.glide || self.lift || self.use_pulse
    }
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn set_config(config: Config) {
    *CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
}
pub fn config() -> Config {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or_default()
}
pub fn config_loaded() -> bool {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}
