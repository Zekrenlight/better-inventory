// As something starts flying to its collector: the pickup list notes what reaches you.
fn better_inventory_pickup(
    game: &crate::game::Game<'_>,
    taken: &crate::game::pickups::Taken,
    collector: &crate::game::pickups::Collector,
) {
    crate::game::better_inventory_glue::picked(game, taken, collector);
}
