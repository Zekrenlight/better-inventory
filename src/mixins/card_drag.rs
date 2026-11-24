// SPDX-License-Identifier: MIT
// Dragging on Better Inventory's settings card: its sliders and scrollbar.
fn better_inventory_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::better_inventory_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::better_inventory_card::drag(game).map(|()| true))
}
