// Before each HUD layer: the hotbar's motion draws the hotbar itself.
fn better_inventory_layer_pre(
    game: &crate::game::Game<'_>,
    layer: &str,
    batch: &mut sunrise_client::hud::HudBatch,
    canceled: &mut bool,
) {
    crate::game::better_inventory_glue::layer_pre(game, layer, batch, canceled);
}
