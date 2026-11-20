// Once a client tick: auto-refill.
fn better_inventory_tick(game: &mut crate::game::Game<'_>) {
    crate::game::better_inventory_glue::tick(game);
}
