// SPDX-License-Identifier: MIT
// The mouse buttons on Better Inventory's settings card.
fn better_inventory_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::better_inventory_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::better_inventory_card::mouse(game, pressed, right).map(|()| true))
}
