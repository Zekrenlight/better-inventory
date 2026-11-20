// With the container screen's slots: the sort buttons.
fn better_inventory_draw_buttons(
    game: &crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
    ox: f32,
    oy: f32,
) {
    crate::game::better_inventory_glue::draw_buttons(game, batch, ox, oy);
}
