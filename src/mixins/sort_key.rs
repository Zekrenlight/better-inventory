// A key press: the sort key over a container screen sorts and takes the key.
fn better_inventory_sort_key(
    game: &mut crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
) -> ::core::ops::ControlFlow<bool> {
    if crate::game::better_inventory_glue::key(game, key) {
        ::core::ops::ControlFlow::Break(true)
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
