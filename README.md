# Better Inventory

A Sunrise client mod by Zekrenlight (MIT).
Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence.

Settings: Options > Mods... > Better Inventory > Settings. The sort key is in Options > Controls.
Every part has its own switch.

<img width="555" height="158" alt="image" src="https://github.com/user-attachments/assets/db00729c-81ed-49a5-be1f-7b4748fee573" />
<img width="921" height="982" alt="image" src="https://github.com/user-attachments/assets/0fbfb70e-9641-4f23-8a50-8550681cc249" />
<img width="921" height="982" alt="image" src="https://github.com/user-attachments/assets/4db3cbe1-6c9f-4a2c-8e70-4b8f3b30a6ca" />
<img width="921" height="982" alt="image" src="https://github.com/user-attachments/assets/2f7b3d7e-1eb1-4800-9887-6b705b72d321" />
<img width="921" height="982" alt="image" src="https://github.com/user-attachments/assets/284b4ca8-5563-4f48-acaf-3a6719f70da0" />


## Sorting
- A small sort button by your inventory, and by chests, barrels and shulker boxes (Sort Buttons).
- The sort key (R at first) sorts the part under the cursor: your inventory, or the container.
- Part stacks are joined, then stacks line up by Sort Order: Groups (the creative inventory's
  order), Name, or Amount. The hotbar is left alone.
- It sorts with ordinary left clicks, the same ones you would make, so it works on servers.

## Auto-refill
- When the stack in your hand runs out or your tool breaks, the next matching stack from your
  inventory moves into that hotbar slot (Auto-Refill, on at first).
- Refill Off Hand: the off hand too. Refill With Similar Tools: any pickaxe for a broken pickaxe.
- Dropping (Q) or swapping hands (F) doesn't refill. No sounds, flashes or warnings, ever.

## Equipment HUD
- Your armor and what you hold, beside the hotbar: main hand, helmet and chestplate on the left;
  off hand, leggings and boots on the right (or all on one side, HUD Sides).
- Wear shown as percent, a bar, uses left, or the icon alone; colored by wear or plain white.
- A switch each for armor, main hand, off hand, and items that never wear out.

## Hotbar motion
- Gliding Selection (with Glide Speed), Springy Selection, Glide Round The Ends.
- Bigger Held Item (with Held Item Size), Item Wobble, Use Squeeze.
- With all of these off, the game draws its own hotbar untouched.

## Pickup list
- Items, experience and arrows (and tridents) you pick up are listed in a corner of the screen:
  icon, name and amount (Pickup List, on at first). More of something still listed adds to its
  line and starts its time again; lines fade out at the end.
- Pickup List Corner: any of the four (bottom right at first). The list keeps above the hotbar,
  the equipment HUD and, on the left, the chat; at the top right it moves below the effect icons.
- List Experience, Show Item Names, Pickup List Time (1-20 s, 5 at first), Pickup List Lines
  (1-12, 6 at first), Pickup List Size (50-150%).
- On a server an item counts as its whole stack, even when only part of it fits.

## Look and settings

Better Inventory uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

Better Inventory works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/better_inventory.smod
```

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
