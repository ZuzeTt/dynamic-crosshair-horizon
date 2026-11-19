# Dynamic Crosshair for Horizon

A native Horizon port of [Crendgrim/DynamicCrosshair](https://github.com/Crendgrim/DynamicCrosshair),
the Fabric/Forge/NeoForge mod that hides or changes the crosshair depending on what the
player looks at and holds.

The original is 84 mixins over vanilla classes. Horizon does not run Java mods, so this
project reimplements the mod's decision pipeline and interaction rules as a native Rust mod:

- `CrosshairHandler` / `CrosshairComponent` keep the original's primary/secondary
  interaction and style-coalescing rules.
- `VanillaApiImpl` keeps the policy handling for tools, melee weapons, ranged weapons,
  throwables, shields, placeable blocks and usable items.
- The item, block and entity mixins are expressed as registry-id, block-state-property
  and tag checks in `src/rules.rs`.
- The 18 builtin crosshair sprites and the original tag files ship with the mod.
- Custom PNG crosshairs from `config/dynamiccrosshair/crosshairs/` are registered as
  runtime GUI sprites and can be picked on the mod's Options page.

## Install

The latest release carries `dynamiccrosshair-9.15.0.hmod`, a native source package.
Install it from GitHub with the launcher:

```sh
horizon-launcher mods add-github https://github.com/ZuzeTt/dynamic-crosshair-horizon \
  --trust-native-code
```

or open **Mods → Add from GitHub** and paste the repository URL. The launcher compiles
the package locally against the Horizon build the manifest pins.

## Build from source

```sh
horizon-launcher native-build /home/akula/Projects/dynamic-crosshair \
  --instance "Horizon" --trust-native-code
```

The manifest pins Horizon build `0b14f8acfd97fa2c750e42795249dcc981178ca4`
(Minecraft 26.3, nightly 2026-11-18). Moving to another build needs
`cargo horizon-mod update` or a new pin in `horizon-mod.toml`.

## Settings

Everything is on **Options > Mods > Dynamic Crosshair**:

- behavior toggles and the Always / If Targeting / If Interactable / Disabled policies;
- the 13 style slots (regular, on block, on entity, tool, melee, ranged, throwable,
  block, interact, usable item, shield and the two tool markers), each with its own
  sprite, colour, blend and coalesce switch;
- the global crosshair colour.

The original's free-form extra item and block lists are in
`config/dynamiccrosshair.json`:

```json
{
  "additional_tools": ["mymod:hammer", "#mymod:tools"],
  "additional_melee_weapons": [],
  "additional_ranged_weapons": [],
  "additional_throwables": [],
  "additional_usable_items": [],
  "additional_interactable_blocks": ["mymod:machine"]
}
```

Entries are item/block ids, or `#tag` for a tag. The lists are ignored when
`enable_tweaks` is off.

## Datapack API

The original tags are in `data/dynamiccrosshair/tags/` and can be extended by a
datapack or another content mod, exactly like upstream:

- `dynamiccrosshair:always_interactable` (blocks and entity types)
- `dynamiccrosshair:always_interactable_in_creative_mode` (blocks)
- `dynamiccrosshair:interactable` (blocks and entity types)
- `dynamiccrosshair:always_usable`, `usable`, `tools`, `melee_weapons`,
  `ranged_weapons`, `throwables`, `shields`, `blocks` (items)

The original Java API (`DynamicCrosshairApi` for other mods) has no direct native
equivalent. Another Horizon mod can add entries to the tags above, or contribute
a block/item part with its own crosshair rules.

## Notes on the port

- The server-side parts of the original (which are none) are not needed; the mod is
  client-side.
- Vanilla's attack indicator is redrawn together with a replaced crosshair from the
  attack mapping and the attack-speed attribute; the client does not expose the exact
  hand-strength ticker, so its phase is mirrored one tick at a time.
- Horizon's mod-facing GUI draw list has no invert blend for mod-namespaced sprites,
  so `enableBlend` styles are drawn with straight alpha instead of inverting the
  pixels under them. The vanilla sprite path keeps vanilla's inverting pipeline.
- Entity rules that need private mob state the client API does not expose around a
  remote entity (tamed, saddled, collar colour, frame contents) fall back to the
  item and entity-type tests that are observable on the client, as the original does
  for its own optional API compat.
- The original's pixel crosshair editor is not part of this port; custom PNGs are
  dropped into the config folder instead.
- `disable_debug_crosshair` is kept as a setting, but Horizon draws the F3 crosshair
  outside the HUD-layer hooks, so this port cannot replace it.

## Credits and licence

Dynamic Crosshair is by [Crendgrim](https://github.com/Crendgrim); this is an
independent Horizon port, not an official Horizon project. The original is licensed
under the GNU Lesser General Public License v3.0; the port keeps that licence and the
original copyright notice in `LICENSE`.

