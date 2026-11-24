// Better Inventory in the game (not in the Cargo library): the sort buttons and key, auto-refill,
// the equipment HUD, the hotbar's motion and the pickup list, on the game's own inventory, menu
// clicks, pickups and HUD.
use super::{Game, Screen};
use mod_better_inventory as bi;
use std::cell::RefCell;
use std::time::Instant;
use sunrise_client::containers::BlockEntity;
use sunrise_client::hud::{HudBatch, rgba};
use sunrise_client::hud_layout::{self, GuiDraw, PoseOp};
use sunrise_client::items::ItemStack;
use sunrise_client::menu::Input;
use winit::keyboard::KeyCode;

// ---- sorting ----------------------------------------------------------------------------------

/// A sort button's side in GUI pixels.
const BUTTON: f32 = 9.0;
/// Its left edge from the screen's, beside the game's own place for one.
const BUTTON_X: f32 = 150.0;

/// The open screen's sortable parts: each one's button corner (from the screen's origin) and
/// slots. The player's main inventory on every container screen with the usual inventory rows;
/// the container's own slots for chest-like ones (chests, barrels, shulker boxes, a server's
/// storage menus).
fn parts(game: &Game<'_>) -> Vec<((f32, f32), Vec<usize>)> {
    if !game.on(bi::ID)
        || game.screen != Screen::Inventory
        || game.station.is_some()
        || game.mount_active()
        || game.player.is_none()
    {
        return Vec::new();
    }
    let chest_like = if game.remote.is_some() {
        game.remote_storage_open()
    } else {
        match game.container_kind() {
            Some(BlockEntity::Chest(_)) => {
                game.chest_rows().is_some() || game.container_block_is_shulker()
            }
            Some(_) | None => false,
        }
    };
    let mut out = Vec::new();
    if chest_like {
        let container: Vec<usize> = game
            .slots()
            .into_iter()
            .filter(|s| (300..500).contains(s))
            .collect();
        if !container.is_empty() {
            out.push(((BUTTON_X, 5.0), container));
        }
    }
    out.push(((BUTTON_X, game.screen_height() - 95.0), (9..36).collect()));
    out
}

fn over(cursor: (f32, f32), x: f32, y: f32) -> bool {
    cursor.0 >= x && cursor.0 < x + BUTTON && cursor.1 >= y && cursor.1 < y + BUTTON
}

/// The buttons, drawn with the screen's slots: a raised square holding a small stepped
/// triangle, brighter under the cursor.
pub(super) fn draw_buttons(game: &Game<'_>, batch: &mut HudBatch, ox: f32, oy: f32) {
    if !bi::config().sort_buttons {
        return;
    }
    let atlas = &game.gui.atlas;
    for ((x, y), _) in parts(game) {
        let (x, y) = (ox + x, oy + y);
        let hot = over(game.cursor, x, y);
        let edge = rgba(24, 24, 24, 255);
        let (light, face, shade) = if hot {
            (rgba(255, 255, 255, 255), rgba(178, 190, 214, 255), rgba(96, 104, 128, 255))
        } else {
            (rgba(222, 222, 222, 255), rgba(150, 150, 150, 255), rgba(86, 86, 86, 255))
        };
        batch.fill(atlas, x, y, BUTTON, BUTTON, edge);
        batch.fill(atlas, x + 1.0, y + 1.0, BUTTON - 2.0, BUTTON - 2.0, shade);
        batch.fill(atlas, x + 1.0, y + 1.0, BUTTON - 3.0, BUTTON - 3.0, light);
        batch.fill(atlas, x + 2.0, y + 2.0, BUTTON - 4.0, BUTTON - 4.0, face);
        // A stepped triangle: short, middle and long rows, right-aligned.
        let ink = rgba(40, 40, 40, 255);
        for (row, w) in [1.0_f32, 3.0, 5.0].into_iter().enumerate() {
            batch.fill(atlas, x + 7.0 - w, y + 2.0 + 2.0 * row as f32, w, 1.0, ink);
        }
    }
}

/// A left press on a sort button sorts its part (true: the press is used).
pub(super) fn press(game: &mut Game<'_>) -> bool {
    if !bi::config().sort_buttons {
        return false;
    }
    let Some(r) = game.renderer.as_ref() else {
        return false;
    };
    let batch = HudBatch::new(
        [r.config.width, r.config.height],
        game.settings.gui_scale_inputs(),
    );
    let (ox, oy) = game.inventory_origin(&batch);
    let cursor = game.cursor;
    let hit = parts(game)
        .into_iter()
        .find(|((x, y), _)| over(cursor, ox + x, oy + y));
    let Some((_, slots)) = hit else {
        return false;
    };
    sort(game, &slots);
    true
}

/// The sort key over a container screen: sorts the part under the cursor (the container when the
/// cursor is over neither part). True when it was the sort key and something was sortable.
pub(super) fn key(game: &mut Game<'_>, key: KeyCode) -> bool {
    if game.screen != Screen::Inventory || game.recipe_search_focused() {
        return false;
    }
    let Some(name) = sunrise_client::keybinds::key_name(key) else {
        return false;
    };
    let Ok(mapping) = game.mod_key_mapping(bi::ID, bi::SORT_KEY) else {
        return false;
    };
    let Ok((bound, _)) = game.mod_key_mapping_binding(mapping) else {
        return false;
    };
    if bound != name {
        return false;
    }
    let parts = parts(game);
    if parts.is_empty() {
        return false;
    }
    let hovered = game.inventory_target().0;
    let slots = match hovered {
        Some(h) if h < 36 => parts.last(),
        _ => parts.first(),
    }
    .map(|(_, slots)| slots.clone());
    if let Some(slots) = slots {
        sort(game, &slots);
    }
    true
}

/// The order of the different stacks in `kinds` for the chosen sort order: each kind's place.
fn kind_ranks(game: &Game<'_>, kinds: &[ItemStack], totals: &[u32]) -> Vec<u64> {
    let names: Vec<String> = kinds
        .iter()
        .map(|s| game.stack_hover_name(s).to_lowercase())
        .collect();
    let groups: Vec<usize> = kinds
        .iter()
        .map(|s| sunrise_client::creative::Creative::order_of(game.creative_screen.search_items(), s).unwrap_or(usize::MAX))
        .collect();
    let mut order: Vec<usize> = (0..kinds.len()).collect();
    let by_group = |a: usize, b: usize| {
        groups[a]
            .cmp(&groups[b])
            .then_with(|| names[a].cmp(&names[b]))
            .then_with(|| kinds[a].item.cmp(&kinds[b].item))
            .then_with(|| kinds[a].damage.cmp(&kinds[b].damage))
    };
    match bi::config().sort_order {
        bi::SortOrder::Groups => order.sort_by(|&a, &b| by_group(a, b)),
        bi::SortOrder::Name => {
            order.sort_by(|&a, &b| names[a].cmp(&names[b]).then_with(|| by_group(a, b)))
        }
        bi::SortOrder::Amount => {
            order.sort_by(|&a, &b| totals[b].cmp(&totals[a]).then_with(|| by_group(a, b)))
        }
    }
    let mut ranks = vec![0; kinds.len()];
    for (place, kind) in order.into_iter().enumerate() {
        ranks[kind] = place as u64;
    }
    ranks
}

/// Sorts `slots` with ordinary left clicks (needs an empty cursor).
fn sort(game: &mut Game<'_>, slots: &[usize]) {
    if game.carried.is_some() {
        return;
    }
    let mut kinds: Vec<ItemStack> = Vec::new();
    let mut cells = Vec::with_capacity(slots.len());
    for &slot in slots {
        match game.slot_view(slot) {
            None => cells.push(None),
            Some(stack) if stack.count == 0 => cells.push(None),
            Some(stack) => {
                let kind = match kinds.iter().position(|k| k.same_item_same_components(&stack)) {
                    Some(k) => k,
                    None => {
                        kinds.push(stack.clone());
                        kinds.len() - 1
                    }
                };
                cells.push(Some(bi::sort::Cell {
                    kind: kind as u32,
                    count: stack.count,
                }));
            }
        }
    }
    let maxes: Vec<u32> = kinds.iter().map(|k| game.max_stack(k).max(1)).collect();
    let mut totals = vec![0u32; kinds.len()];
    for c in cells.iter().flatten() {
        totals[c.kind as usize] += c.count;
    }
    let ranks = kind_ranks(game, &kinds, &totals);
    let max = |k: u32| maxes.get(k as usize).copied().unwrap_or(64);
    let rank = |k: u32| ranks.get(k as usize).copied().unwrap_or(u64::MAX);
    let Some(clicks) = bi::sort::plan(&cells, &max, &rank) else {
        return;
    };
    for i in clicks {
        if !game.slot_click(slots[i], 0, Input::Pickup) {
            break;
        }
    }
}

// ---- auto-refill ------------------------------------------------------------------------------

#[derive(Default)]
struct Watch {
    tick: u64,
    selected: usize,
    main: Option<ItemStack>,
    off: Option<ItemStack>,
    /// The last tick the drop key or the swap-hands key went down.
    hands_key: Option<u64>,
}

thread_local! {
    static WATCH: RefCell<Watch> = RefCell::new(Watch::default());
}

/// The drop key (Q) or swap-hands key (F) went down: a hand emptied by it isn't refilled.
pub(super) fn key_input(game: &Game<'_>, key: &str, action: mod_api::KeyAction) {
    let down = match action {
        mod_api::KeyAction::Press | mod_api::KeyAction::Repeat => true,
        mod_api::KeyAction::Release => false,
    };
    if down && game.screen == Screen::Playing && matches!(key, "key.keyboard.q" | "key.keyboard.f") {
        WATCH.with(|w| {
            let mut w = w.borrow_mut();
            w.hands_key = Some(w.tick);
        });
    }
}

fn held(stack: &Option<ItemStack>) -> Option<ItemStack> {
    stack.as_ref().filter(|s| s.count > 0).cloned()
}

fn as_held(game: &Game<'_>, slot: usize, stack: &ItemStack) -> bi::refill::Held {
    bi::refill::Held {
        slot,
        item: stack.item.to_string(),
        count: stack.count,
        wears: sunrise_client::stations::is_damageable(game.res.items(), stack),
    }
}

/// The inventory slot (9-35) to refill a hand that held `gone` from: the same stack first, then
/// the same item (a tool), then a tool of the same kind.
fn refill_source(game: &Game<'_>, slot: usize, gone: &ItemStack, similar: bool) -> Option<usize> {
    let held = as_held(game, slot, gone);
    let inv = |i: usize| game.inventory.get(i).and_then(|s| s.as_ref()).filter(|s| s.count > 0);
    (9..36)
        .find(|&i| inv(i).is_some_and(|s| s.same_item_same_components(gone)))
        .or_else(|| {
            held.wears
                .then(|| (9..36).find(|&i| inv(i).is_some_and(|s| s.item == gone.item)))
                .flatten()
        })
        .or_else(|| {
            (9..36).find(|&i| {
                inv(i).is_some_and(|s| s.item != gone.item && bi::refill::may_replace(&held, &s.item, similar))
            })
        })
}

/// Once a client tick: a hand that held something last tick and holds nothing now, in the same
/// hotbar slot, without the drop or swap key, is refilled with one swap click.
pub(super) fn tick(game: &mut Game<'_>) {
    let cfg = bi::config();
    if game.player.is_none() || !cfg.pickups {
        PICKS.with(|p| p.borrow_mut().clear());
    }
    let active = game.on(bi::ID)
        && cfg.refill
        && game.player.is_some()
        && game.title.is_none()
        && game.screen == Screen::Playing
        && !game.creative
        && !game.spectator()
        && game.carried.is_none();
    let selected = game.selected.min(8);
    let main_now = held(&game.inventory[selected]);
    let off_now = held(&game.offhand);
    let (main_before, off_before, same_slot, keyed) = WATCH.with(|w| {
        let mut w = w.borrow_mut();
        w.tick += 1;
        let keyed = w.hands_key.is_some_and(|t| w.tick.saturating_sub(t) <= 10);
        (w.main.clone(), w.off.clone(), w.selected == selected, keyed)
    });
    if active {
        // A swap of the hands (the other hand now holds what this one had) isn't running out.
        let moved = |before: &Option<ItemStack>, other_now: &Option<ItemStack>| {
            matches!((before, other_now), (Some(b), Some(o)) if b.item == o.item)
        };
        let main_gone = bi::refill::ran_out(main_before.is_some(), main_now.is_none(), same_slot, keyed)
            && !moved(&main_before, &off_now);
        if main_gone
            && let Some(gone) = &main_before
            && let Some(from) = refill_source(game, selected, gone, cfg.refill_similar)
        {
            game.slot_click(from, selected as i32, Input::Swap);
        }
        let off_gone = cfg.refill_offhand
            && bi::refill::ran_out(off_before.is_some(), off_now.is_none(), true, keyed)
            && !moved(&off_before, &main_now);
        if off_gone
            && let Some(gone) = &off_before
            && let Some(from) = refill_source(game, 40, gone, cfg.refill_similar)
        {
            game.slot_click(from, 40, Input::Swap);
        }
    }
    let main = held(&game.inventory[selected]);
    let off = held(&game.offhand);
    WATCH.with(|w| {
        let mut w = w.borrow_mut();
        w.selected = selected;
        w.main = main;
        w.off = off;
    });
}

// ---- equipment HUD ----------------------------------------------------------------------------

/// After the hotbar: armour and both hands in columns beside it, with what is left of each.
pub(super) fn layer_post(game: &Game<'_>, layer: &str, batch: &mut HudBatch) {
    if layer != "minecraft:hotbar" || !game.on(bi::ID) {
        return;
    }
    draw_pickups(game, batch);
    let cfg = bi::config();
    if !cfg.hud || game.player.is_none() || game.spectator() {
        return;
    }
    use bi::hud::Piece;
    let selected = game.selected.min(8);
    let wanted: [(bool, Piece, &Option<ItemStack>); 6] = [
        (cfg.hud_main_hand, Piece::MainHand, &game.inventory[selected]),
        (cfg.hud_off_hand, Piece::OffHand, &game.offhand),
        (cfg.hud_armor, Piece::Head, &game.armor[0]),
        (cfg.hud_armor, Piece::Chest, &game.armor[1]),
        (cfg.hud_armor, Piece::Legs, &game.armor[2]),
        (cfg.hud_armor, Piece::Feet, &game.armor[3]),
    ];
    let mut shown: Vec<(Piece, &ItemStack)> = Vec::new();
    for (on, piece, stack) in wanted {
        if let Some(s) = stack.as_ref().filter(|s| on && s.count > 0) {
            shown.push((piece, s));
        }
    }
    let items = game.res.items();
    let wear = |s: &ItemStack| -> Option<(i32, i32)> {
        if !sunrise_client::stations::is_damageable(items, s) {
            return None;
        }
        let max = sunrise_client::stations::max_damage(items, s);
        (max > 0).then(|| (s.effective_damage(items.get(&s.item)), max))
    };
    let shown: Vec<(Piece, &ItemStack, Option<(i32, i32)>)> = shown
        .into_iter()
        .map(|(p, s)| (p, s, wear(s)))
        .filter(|(p, _, w)| {
            w.is_some() || cfg.hud_plain_items || matches!(p, Piece::Head | Piece::Chest | Piece::Legs | Piece::Feet)
        })
        .collect();
    if shown.is_empty() {
        return;
    }
    let atlas = &game.gui.atlas;
    let style = cfg.hud_style;
    let labels: Vec<String> = shown
        .iter()
        .map(|(_, _, w)| w.map_or(String::new(), |(d, m)| bi::hud::label(style, d, m)))
        .collect();
    let label_w = match style {
        bi::WearStyle::Percent => HudBatch::text_width(atlas, "100%").ceil() as i32,
        bi::WearStyle::Uses => labels
            .iter()
            .map(|l| HudBatch::text_width(atlas, l).ceil() as i32)
            .max()
            .unwrap_or(0),
        bi::WearStyle::Bar => bi::hud::BAR_W,
        bi::WearStyle::IconOnly => 0,
    };
    let (w, h) = (batch.width as i32, batch.height as i32);
    for ((piece, stack, wear), label) in shown.iter().zip(&labels) {
        let place = bi::hud::place(*piece, cfg.hud_sides, w, h, label_w);
        game.draw_item_model(batch, stack, place.icon.0 as f32, place.icon.1 as f32);
        let Some((damage, max)) = *wear else {
            continue;
        };
        let fraction = bi::hud::left_fraction(damage, max);
        let [r, g, b] = bi::hud::wear_rgb(fraction, cfg.hud_color);
        match style {
            bi::WearStyle::Percent | bi::WearStyle::Uses => {
                let tw = HudBatch::text_width(atlas, label);
                let x = if place.text_right {
                    place.text_x as f32
                } else {
                    place.text_x as f32 - tw
                };
                batch.text(atlas, label, x, place.icon.1 as f32 + 4.0, rgba(r, g, b, 255), true);
            }
            bi::WearStyle::Bar => {
                let (bw, bh) = (bi::hud::BAR_W as f32, bi::hud::BAR_H as f32);
                let x = if place.text_right {
                    place.text_x as f32
                } else {
                    place.text_x as f32 - bw
                };
                let y = place.icon.1 as f32 + 7.0;
                batch.fill(atlas, x - 1.0, y - 1.0, bw + 2.0, bh + 2.0, rgba(0, 0, 0, 200));
                batch.fill(atlas, x, y, (bw * fraction).round().max(1.0), bh, rgba(r, g, b, 255));
            }
            bi::WearStyle::IconOnly => {}
        }
    }
}

// ---- pickup list ------------------------------------------------------------------------------

/// A listed pickup: an item (arrows and tridents are their item) or experience.
enum Pick {
    Item(ItemStack),
    Xp,
}

fn same_pick(a: &Pick, b: &Pick) -> bool {
    match (a, b) {
        (Pick::Item(a), Pick::Item(b)) => a.same_item_same_components(b),
        (Pick::Xp, Pick::Xp) => true,
        (Pick::Item(_), Pick::Xp) | (Pick::Xp, Pick::Item(_)) => false,
    }
}

thread_local! {
    static PICKS: RefCell<bi::pickups::Log<Pick>> = RefCell::new(bi::pickups::Log::default());
}

/// As the game starts something flying to a collector: when that is you, it joins the list.
pub(super) fn picked(game: &Game<'_>, taken: &super::pickups::Taken, collector: &super::pickups::Collector) {
    use super::pickups::{Collector, Taken};
    let cfg = bi::config();
    if !game.on(bi::ID) || !cfg.pickups || game.player.is_none() {
        return;
    }
    match collector {
        Collector::Player => {}
        Collector::Mob(_) => return,
    }
    let (pick, amount) = match taken {
        Taken::Item(item) => (Pick::Item(item.stack.clone()), u64::from(item.stack.count)),
        Taken::Orb(orb) => {
            if !cfg.pickup_xp {
                return;
            }
            (Pick::Xp, u64::from(orb.points()))
        }
        Taken::Arrow(shot) => {
            let stack = shot
                .arrow
                .as_ref()
                .map_or_else(|| ItemStack::new("minecraft:arrow", 1), |a| a.pickup_item.clone());
            let n = u64::from(stack.count.max(1));
            (Pick::Item(stack), n)
        }
        Taken::Trident(t) => (Pick::Item(t.stack.clone()), 1),
    };
    if let Pick::Item(s) = &pick
        && s.count == 0
    {
        return;
    }
    let mut pick = pick;
    if let Pick::Item(s) = &mut pick {
        s.count = 1;
    }
    PICKS.with(|p| p.borrow_mut().add(pick, amount, Instant::now(), same_pick));
}

/// How much of the screen's bottom to keep free on one side: the hotbar, the equipment HUD's
/// rows on that side and, on the left, the chat.
fn bottom_clear(cfg: &bi::Config, left: bool) -> f32 {
    let mut clear: f32 = 26.0;
    if cfg.hud {
        let rows = match (cfg.hud_sides, left) {
            (bi::Sides::Both, _) => 3.0,
            (bi::Sides::Left, true) | (bi::Sides::Right, false) => 6.0,
            (bi::Sides::Left, false) | (bi::Sides::Right, true) => 0.0,
        };
        if rows > 0.0 {
            clear = clear.max(40.0 + (rows - 1.0) * bi::hud::ROW as f32 + 2.0);
        }
    }
    if left {
        // The chat's lines sit above the hotbar on the left.
        clear = clear.max(130.0);
    }
    clear
}

fn draw_pickups(game: &Game<'_>, batch: &mut HudBatch) {
    let cfg = bi::config();
    if !cfg.pickups || game.player.is_none() {
        return;
    }
    let now = Instant::now();
    let life = std::time::Duration::from_secs_f32(cfg.pickup_time);
    let rows: Vec<(Pick, u64, f32)> = PICKS.with(|p| {
        let mut p = p.borrow_mut();
        p.expire(now, life);
        p.shown(now, life, cfg.pickup_rows as usize)
            .into_iter()
            .filter(|(e, _)| cfg.pickup_xp || !matches!(e.what, Pick::Xp))
            .map(|(e, a)| {
                let what = match &e.what {
                    Pick::Item(s) => Pick::Item(s.clone()),
                    Pick::Xp => Pick::Xp,
                };
                (what, e.amount, a)
            })
            .collect()
    });
    if rows.is_empty() {
        return;
    }
    let corner = cfg.pickup_corner;
    let left = matches!(corner, bi::Corner::TopLeft | bi::Corner::BottomLeft);
    let top_clear = match corner {
        bi::Corner::TopRight if !game.potions.effects.list.is_empty() => 50.0,
        bi::Corner::TopRight | bi::Corner::TopLeft | bi::Corner::BottomLeft | bi::Corner::BottomRight => 0.0,
    };
    let ((ax, ay), up, right) = bi::pickups::anchor(
        corner,
        batch.width,
        batch.height,
        bottom_clear(&cfg, left),
        top_clear,
    );
    let atlas = &game.gui.atlas;
    let all = batch.vertices.len();
    for (i, (pick, amount, alpha)) in rows.iter().enumerate() {
        if *alpha <= 0.0 {
            continue;
        }
        let row = bi::pickups::ROW;
        let y = if up { ay - (i as f32 + 1.0) * row } else { ay + i as f32 * row };
        let (icon, name, color) = match pick {
            Pick::Item(s) => (s.clone(), game.stack_hover_name(s), rgba(255, 255, 255, 255)),
            Pick::Xp => (
                ItemStack::new("minecraft:experience_bottle", 1),
                sunrise_client::lang::get_or("better_inventory.pickup.experience", "Experience"),
                rgba(150, 240, 70, 255),
            ),
        };
        let count = match pick {
            Pick::Item(_) => bi::pickups::amount_text(*amount),
            Pick::Xp => format!("+{amount}"),
        };
        let text = match (cfg.pickup_names, pick) {
            (true, Pick::Item(_)) if count.is_empty() => name,
            (true, Pick::Item(_)) => format!("{name} {count}"),
            (true, Pick::Xp) => format!("{count} {name}"),
            (false, Pick::Item(_) | Pick::Xp) => count,
        };
        let start = batch.vertices.len();
        let icon_x = if right { ax - 16.0 } else { ax };
        game.draw_item_model(batch, &icon, icon_x, y + 1.0);
        if !text.is_empty() {
            let tw = HudBatch::text_width(atlas, &text);
            let tx = if right { icon_x - 3.0 - tw } else { icon_x + 16.0 + 3.0 };
            batch.text(atlas, &text, tx, y + 5.0, color, true);
        }
        for v in &mut batch.vertices[start..] {
            let a = ((v.color >> 24) & 0xff) as f32 * alpha;
            v.color = (v.color & 0x00ff_ffff) | ((a.round().clamp(0.0, 255.0) as u32) << 24);
        }
    }
    let scale = cfg.pickup_scale / 100.0;
    if (scale - 1.0).abs() > f32::EPSILON {
        batch.scale_since(all, ax, ay, scale, scale);
    }
}

// ---- hotbar motion ----------------------------------------------------------------------------

fn item_hash(item: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut h);
    h.finish()
}

/// Before the hotbar: with any motion setting on, the hotbar is drawn here (the game's own
/// layout, then the selection frame moved to where it is gliding and each item sized) and the
/// game's drawing of it is skipped. Spectators keep the game's.
pub(super) fn layer_pre(game: &Game<'_>, layer: &str, batch: &mut HudBatch, canceled: &mut bool) {
    if *canceled || layer != "minecraft:hotbar" || !game.on(bi::ID) {
        return;
    }
    let cfg = bi::config();
    if !cfg.moves_hotbar() || game.spectator() || game.player.is_none() {
        return;
    }
    let (w, h) = (batch.width as i32, batch.height as i32);
    let partial = game.clock.fraction() as f32;
    let selected = game.selected.min(8);
    let inputs = hud_layout::HotbarInputs {
        selected: selected as i32,
        right_handed: !game.settings.main_hand_left,
        offhand: game.offhand.is_some(),
        slots: std::array::from_fn(|i| {
            game.inventory[i]
                .as_ref()
                .map(|_| (f32::from(game.hotbar_pop[i]) - partial).max(0.0))
        }),
        offhand_pop: (f32::from(game.hotbar_pop[9]) - partial).max(0.0),
    };
    let draws = hud_layout::hotbar(w, h, &inputs);
    let looks: [Option<bi::motion::SlotLook>; 10] = std::array::from_fn(|i| {
        let stack = if i < 9 {
            game.inventory[i].as_ref()
        } else {
            game.offhand.as_ref()
        };
        stack.map(|s| bi::motion::SlotLook {
            item: item_hash(&s.item),
            count: s.count,
            damage: s.damage,
        })
    });
    let pose = bi::motion::step(Instant::now(), selected, looks, &cfg);
    let center = w / 2;
    // The frame: vanilla's sits at `center - 92 + 20 * selected`.
    let frame_x = |at: f32| (center - 92) as f32 + 20.0 * at;
    let mut before = Vec::new();
    let mut frames = Vec::new();
    let mut after = Vec::new();
    let mut seen_frame = false;
    for d in draws {
        match d {
            GuiDraw::Sprite {
                sprite,
                x: _,
                y,
                w: sw,
                h: sh,
                region,
                color,
                blend,
            } if sprite == "hud/hotbar_selection" => {
                seen_frame = true;
                let mut at = vec![pose.selector];
                // Past an edge: the rest of it shows at the other end.
                if pose.selector > 8.0 {
                    at.push(pose.selector - 9.0);
                } else if pose.selector < 0.0 {
                    at.push(pose.selector + 9.0);
                }
                for a in at {
                    frames.push(GuiDraw::Push(vec![PoseOp::Translate(frame_x(a), 0.0)]));
                    frames.push(GuiDraw::Sprite {
                        sprite: sprite.clone(),
                        x: 0,
                        y,
                        w: sw,
                        h: sh,
                        region,
                        color,
                        blend,
                    });
                    frames.push(GuiDraw::Pop);
                }
            }
            GuiDraw::Item {
                slot,
                x,
                y,
                seed,
                scale,
                pivot: _,
            } => {
                let i = if slot == 40 { 9 } else { (slot as usize).min(9) };
                let m = pose.scale[i];
                let item = GuiDraw::Item {
                    slot,
                    x,
                    y,
                    seed,
                    scale: [scale[0] * m, scale[1] * m],
                    pivot: [(x + 8) as f32, (y + 12) as f32],
                };
                if seen_frame {
                    after.push(item);
                } else {
                    before.push(item);
                }
            }
            other => {
                if seen_frame {
                    after.push(other);
                } else {
                    before.push(other);
                }
            }
        }
    }
    let atlas = &game.gui.atlas;
    let mut items = crate::game::gui_paint_items::HotbarItems { game, source: crate::game::gui_paint_items::HotbarSource::Own };
    hud_layout::paint_items(batch, atlas, &before, &mut items);
    let start = batch.vertices.len();
    hud_layout::paint(batch, atlas, &frames);
    if pose.selector > 8.0 || pose.selector < 0.0 {
        // Gliding round the end: only the part over the hotbar shows.
        let (x0, x1) = ((center - 92) as f32, (center + 92) as f32);
        batch.clip_since(start, x0, 0.0, x1, h as f32);
    }
    hud_layout::paint_items(batch, atlas, &after, &mut items);
    *canceled = true;
}
