// SPDX-License-Identifier: MIT
// Keys on Better Inventory's settings card; Escape (search let go) is Sunrise's, to leave.
fn better_inventory_card_options_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    text: Option<&str>,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::better_inventory_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    match crate::game::better_inventory_card::key(game, key, text) {
        Some(done) => ::core::ops::ControlFlow::Break(done.map(|()| true)),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
