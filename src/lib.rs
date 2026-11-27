//! Better Inventory 1.0.0 for Sunrise, by Zekrenlight (MIT).
//!
//! - Sorting: a small button by the inventory and by chest-like containers, and a key (R at
//!   first) that sorts the part under the cursor. Stacks are joined and ordered with ordinary
//!   left clicks, so it works on any server.
//! - Auto-refill: a hand whose stack runs out, or whose tool breaks, gets the next matching stack
//!   from the inventory. No sounds, flashes or warnings.
//! - Equipment HUD: armour and both hands beside the hotbar, with what is left of each.
//! - Hotbar motion: the selection frame glides between slots, the held item stands a little
//!   larger, and a used stack gives a small squeeze.
//! - Pickup list: items, experience and arrows you pick up, listed in a screen corner with their
//!   icon, name and amount; repeats join, and entries fade out.
//!
//! This crate holds what needs no game types: the settings (drawn by the game on the mod's page,
//! which Mod Menu opens from Options > Mods...), the sort planner, the refill rules, the HUD's
//! layout, the hotbar's motion and the pickup list. The game half is `src/native/glue.rs` with
//! the helpers in `src/mixins/`.

pub mod config;
pub mod hud;
pub mod motion;
pub mod pickups;
pub mod refill;
pub mod sort;

pub use config::*;

use mod_api::{
    ClientHost, ClientTickPostEvent, Host, Hooks, KeyRegistration, Registrar, RegistrationContext,
    SettingChangedEvent, SettingKey, SettingSpec,
};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Default)]
struct BetterInventory;

fn label(key: &str) -> String {
    format!("better_inventory.setting.{key}")
}

fn remember(keys: &mut HashMap<String, SettingKey>, key: &str, k: Option<SettingKey>) {
    if let Some(k) = k {
        keys.insert(key.to_string(), k);
    }
}

fn toggle(registrar: &mut dyn Registrar, keys: &mut HashMap<String, SettingKey>, key: &str, default: bool) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(keys, key, registrar.register_setting(SettingSpec::toggle(key, l, default).tooltip(tip)).ok());
}

fn range(
    registrar: &mut dyn Registrar,
    keys: &mut HashMap<String, SettingKey>,
    key: &str,
    min: f64,
    max: f64,
    step: f64,
    default: f64,
) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(
        keys,
        key,
        registrar
            .register_setting(SettingSpec::range(key, l, min, max, step, default).tooltip(tip))
            .ok(),
    );
}

fn choice(
    registrar: &mut dyn Registrar,
    keys: &mut HashMap<String, SettingKey>,
    key: &str,
    options: &[&str],
    default: usize,
) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(
        keys,
        key,
        registrar
            .register_setting(SettingSpec::choice(key, l, options.iter().copied(), default).tooltip(tip))
            .ok(),
    );
}

static SETTING_KEYS: OnceLock<HashMap<String, SettingKey>> = OnceLock::new();

fn key_of(name: &str) -> Option<SettingKey> {
    SETTING_KEYS.get()?.get(name).copied()
}

fn bool_of(host: &dyn Host, name: &str, default: bool) -> bool {
    key_of(name).and_then(|k| host.setting_bool(k).ok()).unwrap_or(default)
}
fn f32_of(host: &dyn Host, name: &str, default: f32) -> f32 {
    key_of(name)
        .and_then(|k| host.setting_f64(k).ok())
        .filter(|v| v.is_finite())
        .map_or(default, |v| v as f32)
}
fn choice_of(host: &dyn Host, name: &str, default: u32) -> u32 {
    key_of(name).and_then(|k| host.setting_choice(k).ok()).unwrap_or(default)
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn Host) {
    let d = Config::default();
    set_config(Config {
        sort_buttons: bool_of(host, KEY_SORT_BUTTONS, d.sort_buttons),
        sort_order: SortOrder::from_index(choice_of(host, KEY_SORT_ORDER, 0)),
        refill: bool_of(host, KEY_REFILL, d.refill),
        refill_offhand: bool_of(host, KEY_REFILL_OFFHAND, d.refill_offhand),
        refill_similar: bool_of(host, KEY_REFILL_SIMILAR, d.refill_similar),
        hud: bool_of(host, KEY_HUD, d.hud),
        hud_armor: bool_of(host, KEY_HUD_ARMOR, d.hud_armor),
        hud_main_hand: bool_of(host, KEY_HUD_MAIN_HAND, d.hud_main_hand),
        hud_off_hand: bool_of(host, KEY_HUD_OFF_HAND, d.hud_off_hand),
        hud_plain_items: bool_of(host, KEY_HUD_PLAIN_ITEMS, d.hud_plain_items),
        hud_style: WearStyle::from_index(choice_of(host, KEY_HUD_STYLE, 0)),
        hud_color: bool_of(host, KEY_HUD_COLOR, d.hud_color),
        hud_sides: Sides::from_index(choice_of(host, KEY_HUD_SIDES, 0)),
        glide: bool_of(host, KEY_GLIDE, d.glide),
        glide_speed: f32_of(host, KEY_GLIDE_SPEED, d.glide_speed).clamp(1.0, 10.0),
        spring: bool_of(host, KEY_SPRING, d.spring),
        wrap: bool_of(host, KEY_WRAP, d.wrap),
        lift: bool_of(host, KEY_LIFT, d.lift),
        lift_size: f32_of(host, KEY_LIFT_SIZE, d.lift_size).clamp(0.0, 50.0),
        wobble: bool_of(host, KEY_WOBBLE, d.wobble),
        use_pulse: bool_of(host, KEY_USE_PULSE, d.use_pulse),
        pickups: bool_of(host, KEY_PICKUPS, d.pickups),
        pickup_corner: Corner::from_index(choice_of(host, KEY_PICKUP_CORNER, Corner::DEFAULT as u32)),
        pickup_xp: bool_of(host, KEY_PICKUP_XP, d.pickup_xp),
        pickup_names: bool_of(host, KEY_PICKUP_NAMES, d.pickup_names),
        pickup_time: f32_of(host, KEY_PICKUP_TIME, d.pickup_time).clamp(1.0, 20.0),
        pickup_rows: f32_of(host, KEY_PICKUP_ROWS, d.pickup_rows as f32).round().clamp(1.0, 12.0) as u32,
        pickup_scale: f32_of(host, KEY_PICKUP_SCALE, d.pickup_scale).clamp(50.0, 150.0),
    });
}

#[mod_api::sunrise_mod]
impl Hooks for BetterInventory {
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        registrar.register_key_mapping(
            KeyRegistration::new(SORT_KEY, SORT_KEY_DEFAULT)
                .label("better_inventory.key.sort")
                .category("better_inventory"),
        );
        let d = Config::default();
        let mut keys = HashMap::new();
        // Sorting
        toggle(registrar, &mut keys, KEY_SORT_BUTTONS, d.sort_buttons);
        choice(registrar, &mut keys, KEY_SORT_ORDER, &SortOrder::OPTIONS, 0);
        // Refill
        toggle(registrar, &mut keys, KEY_REFILL, d.refill);
        toggle(registrar, &mut keys, KEY_REFILL_OFFHAND, d.refill_offhand);
        toggle(registrar, &mut keys, KEY_REFILL_SIMILAR, d.refill_similar);
        // Equipment HUD
        toggle(registrar, &mut keys, KEY_HUD, d.hud);
        toggle(registrar, &mut keys, KEY_HUD_ARMOR, d.hud_armor);
        toggle(registrar, &mut keys, KEY_HUD_MAIN_HAND, d.hud_main_hand);
        toggle(registrar, &mut keys, KEY_HUD_OFF_HAND, d.hud_off_hand);
        toggle(registrar, &mut keys, KEY_HUD_PLAIN_ITEMS, d.hud_plain_items);
        choice(registrar, &mut keys, KEY_HUD_STYLE, &WearStyle::OPTIONS, 0);
        toggle(registrar, &mut keys, KEY_HUD_COLOR, d.hud_color);
        choice(registrar, &mut keys, KEY_HUD_SIDES, &Sides::OPTIONS, 0);
        // Hotbar motion
        toggle(registrar, &mut keys, KEY_GLIDE, d.glide);
        range(registrar, &mut keys, KEY_GLIDE_SPEED, 1.0, 10.0, 1.0, f64::from(d.glide_speed));
        toggle(registrar, &mut keys, KEY_SPRING, d.spring);
        toggle(registrar, &mut keys, KEY_WRAP, d.wrap);
        toggle(registrar, &mut keys, KEY_LIFT, d.lift);
        range(registrar, &mut keys, KEY_LIFT_SIZE, 0.0, 50.0, 5.0, f64::from(d.lift_size));
        toggle(registrar, &mut keys, KEY_WOBBLE, d.wobble);
        toggle(registrar, &mut keys, KEY_USE_PULSE, d.use_pulse);
        // Pickup list
        toggle(registrar, &mut keys, KEY_PICKUPS, d.pickups);
        choice(registrar, &mut keys, KEY_PICKUP_CORNER, &Corner::OPTIONS, Corner::DEFAULT);
        toggle(registrar, &mut keys, KEY_PICKUP_XP, d.pickup_xp);
        toggle(registrar, &mut keys, KEY_PICKUP_NAMES, d.pickup_names);
        range(registrar, &mut keys, KEY_PICKUP_TIME, 1.0, 20.0, 1.0, f64::from(d.pickup_time));
        range(registrar, &mut keys, KEY_PICKUP_ROWS, 1.0, 12.0, 1.0, f64::from(d.pickup_rows));
        range(registrar, &mut keys, KEY_PICKUP_SCALE, 50.0, 150.0, 10.0, f64::from(d.pickup_scale));
        // The accent colour of better_inventory's cards (Sunrise orange by default).
        remember(
            &mut keys,
            "accent",
            registrar
                .register_setting(
                    mod_api::SettingSpec::color("accent", "better_inventory.setting.accent", 0xF9801D)
                        .tooltip("better_inventory.setting.accent.tooltip"),
                )
                .ok(),
        );
        let _ = SETTING_KEYS.set(keys);
    }

    fn setting_changed(&self, host: &dyn Host, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        // The first tick reads the settings; later changes come through `setting_changed`.
        if !config_loaded() {
            sync(host);
        }
    }
}
