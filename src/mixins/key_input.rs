// Every key event: the drop and swap-hands keys hold off auto-refill for a moment.
fn better_inventory_key_input(game: &crate::game::Game<'_>, key: &str, action: mod_api::KeyAction) {
    crate::game::better_inventory_glue::key_input(game, key, action);
}
