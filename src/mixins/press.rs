// A left press on a container screen: a sort button sorts and takes the press.
fn better_inventory_press(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::better_inventory_glue::press(game) {
        ::core::ops::ControlFlow::Break(true)
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
