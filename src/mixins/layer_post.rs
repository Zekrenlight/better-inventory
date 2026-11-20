// After each HUD layer: the equipment HUD beside the hotbar and the pickup list.
fn better_inventory_layer_post(
    game: &crate::game::Game<'_>,
    layer: &str,
    batch: &mut sunrise_client::hud::HudBatch,
) {
    crate::game::better_inventory_glue::layer_post(game, layer, batch);
}
