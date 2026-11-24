// SPDX-License-Identifier: MIT
// The wheel on Better Inventory's settings card.
fn better_inventory_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::better_inventory_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::better_inventory_card::scroll(game, amount).map(|()| true))
}
