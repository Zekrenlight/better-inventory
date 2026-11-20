//! Auto-refill: when the stack in a hand runs out or the tool in it breaks, the next matching
//! stack from the inventory takes its place. Quietly: no sound, no flash, no message.

/// What a hand held at the end of a tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The hotbar slot (0-8), or 40 for the off hand.
    pub slot: usize,
    pub item: String,
    pub count: u32,
    /// The item wears out (tools, weapons, armour).
    pub wears: bool,
}

/// What kind of tool an item is, by its id, for "similar tools": any pickaxe for a pickaxe.
pub fn tool_family(item: &str) -> Option<&'static str> {
    const FAMILIES: [&str; 12] = [
        "_pickaxe", "_axe", "_shovel", "_hoe", "_sword", "_spear", "shears", "fishing_rod",
        "flint_and_steel", "bow", "crossbow", "brush",
    ];
    let path = item.rsplit(':').next().unwrap_or(item);
    FAMILIES.into_iter().find(|f| {
        if f.starts_with('_') {
            path.ends_with(f)
        } else {
            path == *f
        }
    })
}

/// Whether `candidate` may take the place of `gone` (the same item; or, for a tool with
/// `similar` on, a tool of the same family).
pub fn may_replace(gone: &Held, candidate: &str, similar: bool) -> bool {
    if candidate == gone.item {
        return true;
    }
    similar && gone.wears && tool_family(&gone.item).is_some_and(|f| tool_family(candidate) == Some(f))
}

/// The hand ran out between two looks at it: it held something and now holds nothing, in the
/// same slot, with nothing thrown away or swapped by a key meanwhile.
pub fn ran_out(had: bool, now_empty: bool, same_slot: bool, keyed: bool) -> bool {
    had && now_empty && same_slot && !keyed
}

#[cfg(test)]
mod check {
    use super::*;

    #[test]
    fn families() {
        assert_eq!(tool_family("minecraft:diamond_pickaxe"), Some("_pickaxe"));
        assert_eq!(tool_family("minecraft:iron_axe"), Some("_axe"));
        assert_eq!(tool_family("minecraft:pickaxe_like"), None);
        assert_eq!(tool_family("minecraft:bow"), Some("bow"));
        assert_eq!(tool_family("minecraft:crossbow"), Some("crossbow"));
        let gone = Held { slot: 0, item: "minecraft:stone_pickaxe".into(), count: 1, wears: true };
        assert!(may_replace(&gone, "minecraft:iron_pickaxe", true));
        assert!(!may_replace(&gone, "minecraft:iron_pickaxe", false));
        assert!(!may_replace(&gone, "minecraft:iron_axe", true));
    }
}
