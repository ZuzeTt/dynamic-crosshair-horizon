//! The vanilla interaction rules, ported from DynamicCrosshair's mixins.
//!
//! The original patches roughly eighty vanilla classes. The same decisions
//! are expressed here as one function per item, block and entity, keyed by
//! registry id, block state properties and tags. The control flow follows
//! `CrosshairHandler`, `VanillaApiImpl` and `DynamicCrosshairBaseItem`.

use mod_api::{BlockState, EquipmentSlot, HostBlockStates, ItemStack};

use crate::config::{BlockPolicy, Config, Policy, UsablePolicy};
use crate::ctx::Ctx;
use crate::interaction::{Crosshair, InteractionType as I};

fn any(id: &str, list: &[&str]) -> bool {
    list.contains(&id)
}

fn suffix(id: &str, list: &[&str]) -> bool {
    list.iter().any(|suffix| id.ends_with(suffix))
}

// ---------------------------------------------------------------------------
// Item categories

fn extra_item(ctx: &Ctx, stack: &ItemStack, list: &[String]) -> bool {
    !list.is_empty()
        && extra_contains(list, &ctx.stack_id(stack), |tag| {
            ctx.stack_in_tag(stack, tag)
        })
}

fn extra_block(ctx: &Ctx, state: BlockState, list: &[String]) -> bool {
    !list.is_empty()
        && extra_contains(list, &ctx.block_id(state), |tag| {
            ctx.host.block_in_tag(state.block(), tag).unwrap_or(false)
        })
}

pub fn is_block(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.host.item_block(stack.item()).is_some()
}

pub fn is_tool(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:pickaxes")
        || ctx.stack_in_tag(stack, "minecraft:shovels")
        || ctx.stack_in_tag(stack, "minecraft:axes")
        || ctx.stack_in_tag(stack, "minecraft:hoes")
        || any(&id, &["minecraft:shears", "minecraft:flint_and_steel"])
        || suffix(&id, &["_pickaxe", "_shovel", "_axe", "_hoe"])
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.tools))
}

pub fn is_melee_weapon(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:swords")
        || any(&id, &["minecraft:trident", "minecraft:mace"])
        || suffix(&id, &["_sword", "_mace"])
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.melee_weapons))
}

pub fn is_ranged_weapon(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:bow")
        || ctx.stack_in_tag(stack, "minecraft:crossbow")
        || any(
            &id,
            &[
                "minecraft:bow",
                "minecraft:crossbow",
                "minecraft:trident",
                "minecraft:fishing_rod",
            ],
        )
        || suffix(&id, &["_bow", "_crossbow", "_spear", "fishing_rod"])
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.ranged_weapons))
}

pub fn is_shield(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.stack_id(stack) == "minecraft:shield"
}

pub fn is_throwable(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    any(
        &id,
        &[
            "minecraft:egg",
            "minecraft:snowball",
            "minecraft:splash_potion",
            "minecraft:lingering_potion",
            "minecraft:experience_bottle",
            "minecraft:ender_pearl",
            "minecraft:wind_charge",
        ],
    ) || ctx.stack_in_tag(stack, "c:ender_pearls")
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.throwables))
}

pub fn is_always_usable(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    any(
        &id,
        &[
            "minecraft:potion",
            "minecraft:honey_bottle",
            "minecraft:golden_apple",
            "minecraft:enchanted_golden_apple",
            "minecraft:milk_bucket",
            "minecraft:ominous_bottle",
            "minecraft:goat_horn",
            "minecraft:writable_book",
            "minecraft:written_book",
        ],
    ) || ctx.stack_in_tag(stack, "c:potions")
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.usable_items))
}

pub fn is_always_usable_on_block(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    id == "minecraft:brush"
}

pub fn is_always_usable_on_entity(_ctx: &Ctx, _stack: &ItemStack) -> bool {
    false
}

pub fn is_always_usable_on_miss(_ctx: &Ctx, _stack: &ItemStack) -> bool {
    false
}

pub fn is_usable(ctx: &Ctx, stack: &ItemStack, config: &Config) -> bool {
    let id = ctx.stack_id(stack);
    ctx.item_has_component(stack.item(), "minecraft:food")
        || ctx.stack_has_component(stack, "minecraft:food")
        || ctx.item_has_component(stack.item(), "minecraft:jukebox_playable")
        || ctx.stack_has_component(stack, "minecraft:jukebox_playable")
        || ctx.item_has_component(stack.item(), "minecraft:equippable")
        || any(
            &id,
            &[
                "minecraft:elytra",
                "minecraft:firework_rocket",
                "minecraft:fire_charge",
                "minecraft:honeycomb",
                "minecraft:ender_eye",
                "minecraft:glass_bottle",
                "minecraft:bucket",
                "minecraft:bone_meal",
                "minecraft:bundle",
            ],
        )
        || matches!(
            id.rsplit_once(':').map(|(_, path)| path).unwrap_or(""),
            "water_bucket"
                | "lava_bucket"
                | "milk_bucket"
                | "powder_snow_bucket"
                | "cod_bucket"
                | "salmon_bucket"
                | "tropical_fish_bucket"
                | "pufferfish_bucket"
                | "axolotl_bucket"
                | "tadpole_bucket"
        )
        || ctx.stack_in_tag(stack, "c:foods")
        || ctx.stack_in_tag(stack, "c:music_discs")
        || ctx.stack_in_tag(stack, "c:armors")
        || (config.enable_tweaks && extra_item(ctx, stack, &config.extra.usable_items))
}

pub fn is_food(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.item_has_component(stack.item(), "minecraft:food")
        || ctx.stack_has_component(stack, "minecraft:food")
}

fn food_can_always_eat(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    any(
        &id,
        &[
            "minecraft:golden_apple",
            "minecraft:enchanted_golden_apple",
            "minecraft:honey_bottle",
        ],
    )
}

pub fn is_spyglass(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.stack_id(stack) == "minecraft:spyglass"
}

pub fn is_sword(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:swords") || suffix(&id, &["_sword"])
}

pub fn is_bow(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    id == "minecraft:bow" || suffix(&id, &["_bow"])
}

pub fn is_crossbow(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    id == "minecraft:crossbow" || suffix(&id, &["_crossbow"])
}

pub fn is_trident(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.stack_id(stack) == "minecraft:trident"
}

pub fn is_shears(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    id == "minecraft:shears" || suffix(&id, &["_shears"])
}

pub fn is_axe(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:axes") || suffix(&id, &["_axe"])
}

pub fn is_hoe(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    ctx.stack_in_tag(stack, "minecraft:hoes") || suffix(&id, &["_hoe"])
}

// ---------------------------------------------------------------------------
// Hand policies

pub fn include_usable_item(ctx: &Ctx, config: &Config) -> bool {
    match config.holding_usable_item {
        UsablePolicy::Always => true,
        UsablePolicy::IfInteractable => !ctx.is_cooling_down(),
        UsablePolicy::Disabled => false,
    }
}

pub fn include_throwable(ctx: &Ctx, config: &Config) -> bool {
    match config.holding_throwable {
        UsablePolicy::Always => true,
        UsablePolicy::IfInteractable => !ctx.is_cooling_down(),
        UsablePolicy::Disabled => false,
    }
}

pub fn include_ranged_weapon(config: &Config) -> bool {
    config.holding_ranged_weapon != UsablePolicy::Disabled
}

pub fn include_melee_weapon(ctx: &Ctx, config: &Config) -> bool {
    ctx.is_main_hand() && config.holding_melee_weapon
}

pub fn include_tool(ctx: &Ctx, config: &Config) -> bool {
    ctx.is_main_hand()
        && match config.holding_tool {
            Policy::Always => true,
            Policy::IfTargeting => ctx.is_targeting(),
            Policy::Disabled => false,
        }
}

pub fn include_shield(ctx: &Ctx, config: &Config) -> bool {
    config.holding_shield && !ctx.is_cooling_down()
}

pub fn can_place_item_as_block(ctx: &Ctx) -> bool {
    let Some(hit) = ctx.block() else {
        return false;
    };
    let Some(stack) = ctx.hand_stack() else {
        return false;
    };
    let Some(_block) = ctx.host.item_block(stack.item()) else {
        return false;
    };
    // Vanilla tries the clicked face first and falls back to the clicked block.
    let level = ctx.level();
    let face = hit.face;
    let neighbour = [
        hit.pos[0] + face[0],
        hit.pos[1] + face[1],
        hit.pos[2] + face[2],
    ];
    if let Some(level) = level {
        if let Some(state) = level.block_state(ctx.host, neighbour) {
            if ctx.host.block_states().replaceable(state).unwrap_or(false) {
                return true;
            }
        }
    }
    ctx.host
        .block_states()
        .replaceable(hit.state)
        .unwrap_or(false)
}

pub fn include_holding_block(ctx: &Ctx, config: &Config, stack: &ItemStack) -> bool {
    if ctx.is_off_hand() && !config.holding_block_in_offhand {
        return false;
    }
    if !is_block(ctx, stack) {
        return false;
    }
    match config.holding_block {
        BlockPolicy::Always => true,
        BlockPolicy::IfTargeting => ctx.is_targeting(),
        BlockPolicy::IfInteractable => can_place_item_as_block(ctx),
        BlockPolicy::Disabled => false,
    }
}

// ---------------------------------------------------------------------------
// Tools

pub fn check_tool_with_block(ctx: &Ctx, config: &Config) -> I {
    let Some(hit) = ctx.block() else {
        return I::Empty;
    };
    let Some(stack) = ctx.hand_stack() else {
        return I::Empty;
    };
    let breakable = ctx
        .block_destroy_speed(hit.state)
        .is_some_and(|speed| speed >= 0.0);
    let speed = ctx.break_speed(hit.state, Some(stack)).unwrap_or(1.0);
    if is_tool(ctx, stack, config) {
        if speed > 1.0 && breakable {
            return I::CorrectTool;
        }
        return I::IncorrectTool;
    }
    if speed > 1.0 && breakable {
        return I::CorrectTool;
    }
    if is_shears(ctx, stack) {
        return I::IncorrectTool;
    }
    I::Empty
}

// ---------------------------------------------------------------------------
// Items

fn base_item_compute(ctx: &Ctx, config: &Config) -> I {
    let Some(stack) = ctx.hand_stack() else {
        return I::Empty;
    };
    if is_food(ctx, stack) && (ctx.can_eat(false) || food_can_always_eat(ctx, stack)) {
        return I::ConsumeItem;
    }
    if is_always_usable(ctx, stack, config) {
        return I::UseItem;
    }
    if is_always_usable_on_block(ctx, stack) && ctx.block().is_some() {
        return I::UseItemOnBlock;
    }
    if is_always_usable_on_entity(ctx, stack) && ctx.entity().is_some() {
        return I::UseItemOnEntity;
    }
    if is_always_usable_on_miss(ctx, stack) && !ctx.is_targeting() {
        return I::UseItem;
    }
    if is_tool(ctx, stack, config) {
        return I::Tool;
    }
    if is_throwable(ctx, stack, config) {
        return I::ThrowItem;
    }
    if is_shield(ctx, stack) {
        return I::Shield;
    }
    if is_melee_weapon(ctx, stack, config) {
        return I::MeleeWeapon;
    }
    if is_ranged_weapon(ctx, stack, config) {
        return I::RangedWeapon;
    }
    if is_block(ctx, stack) {
        return I::PlaceBlock;
    }
    if let Some(slot) = equip_slot(ctx, stack) {
        if player_can_equip(ctx, stack, slot, config) {
            return I::EquipItem;
        }
    }
    I::Empty
}

fn component_slot(ctx: &Ctx, stack: &ItemStack) -> Option<EquipmentSlot> {
    let tag = ctx.stack_component(stack, "minecraft:equippable")?;
    let mod_api::Tag::Compound(map) = tag else {
        return None;
    };
    let mut slot = None;
    for (key, value) in &map {
        if key.to_string_lossy() == "slot" {
            if let mod_api::Tag::String(id) = value {
                slot = match id.to_string_lossy().as_str() {
                    "head" | "minecraft:head" => Some(EquipmentSlot::Head),
                    "chest" | "minecraft:chest" => Some(EquipmentSlot::Chest),
                    "legs" | "minecraft:legs" => Some(EquipmentSlot::Legs),
                    "feet" | "minecraft:feet" => Some(EquipmentSlot::Feet),
                    "body" | "minecraft:body" => Some(EquipmentSlot::Body),
                    "saddle" | "minecraft:saddle" => Some(EquipmentSlot::Saddle),
                    "mainhand" | "minecraft:mainhand" => Some(EquipmentSlot::MainHand),
                    "offhand" | "minecraft:offhand" => Some(EquipmentSlot::OffHand),
                    _ => None,
                };
            }
        }
    }
    slot
}

fn equip_slot(ctx: &Ctx, stack: &ItemStack) -> Option<EquipmentSlot> {
    component_slot(ctx, stack)
}

fn player_can_equip(ctx: &Ctx, stack: &ItemStack, slot: EquipmentSlot, _config: &Config) -> bool {
    if matches!(slot, EquipmentSlot::Body | EquipmentSlot::Saddle) {
        return false;
    }
    let current = ctx.player_equipment(slot);
    current.as_ref() != Some(stack)
}

fn item_compute(ctx: &Ctx, config: &Config) -> I {
    let Some(stack) = ctx.hand_stack() else {
        return I::Empty;
    };
    let id = ctx.stack_id(stack);

    if is_spyglass(ctx, stack) {
        return I::Spyglass;
    }

    // Swords are always melee weapons, even when the generic category would
    // also call them tools (some data packs put them in both).
    if is_sword(ctx, stack) || id == "minecraft:mace" {
        return I::MeleeWeapon;
    }

    if is_axe(ctx, stack) {
        if ctx.block().is_some() && config.holding_tool != Policy::Disabled {
            let hit = ctx.block().unwrap();
            if axe_transforms(ctx, hit.state) {
                return I::UsableTool;
            }
        }
        return base_item_compute(ctx, config);
    }

    if is_hoe(ctx, stack) {
        if ctx.block().is_some() && config.holding_usable_item != UsablePolicy::Disabled {
            if is_fertile_or_tillable(ctx) {
                return I::UsableTool;
            }
        }
        return base_item_compute(ctx, config);
    }

    if ctx.stack_in_tag(stack, "minecraft:shovels") || suffix(&id, &["_shovel"]) {
        if ctx.block().is_some() && config.holding_usable_item != UsablePolicy::Disabled {
            if is_path_or_flattenable(ctx) {
                return I::UsableTool;
            }
        }
        return base_item_compute(ctx, config);
    }

    if is_shears(ctx, stack) {
        if ctx.block().is_some() && config.holding_usable_item != UsablePolicy::Disabled {
            if is_growing_stem(ctx) || is_full_beehive(ctx) && !ctx.is_sneaking() {
                return I::UsableTool;
            }
        }
        return base_item_compute(ctx, config);
    }

    if id == "minecraft:armor_stand" {
        return I::PlaceEntity;
    }

    if id == "minecraft:bow" || is_bow(ctx, stack) {
        return item_override_for_ranged(ctx, config, stack, I::RangedWeapon);
    }

    if id == "minecraft:crossbow" || is_crossbow(ctx, stack) {
        return item_override_for_ranged(ctx, config, stack, I::RangedWeapon);
    }

    if is_trident(ctx, stack) {
        return item_override_for_ranged(ctx, config, stack, I::RangedWeapon);
    }

    if id == "minecraft:brush" {
        if let Some(hit) = ctx.block() {
            let block = ctx.block_id(hit.state);
            if block.ends_with("suspicious_sand") || block.ends_with("suspicious_gravel") {
                return I::UseItemOnBlock;
            }
        }
        return I::NoAction;
    }

    if id.ends_with("_bucket") || id == "minecraft:bucket" {
        return bucket_compute(ctx);
    }

    if id == "minecraft:bundle" {
        if !ctx.stack_component_is_empty_list(stack, "minecraft:bundle_contents") {
            return I::UseItem;
        }
        return I::NoAction;
    }

    if id == "minecraft:compass" || id == "minecraft:recovery_compass" {
        if let Some(hit) = ctx.block() {
            if ctx.block_id(hit.state) == "minecraft:lodestone" {
                return I::UseItemOnBlock;
            }
        }
        return I::NoAction;
    }

    if id == "minecraft:end_crystal" {
        if let Some(hit) = ctx.block() {
            let above = [hit.pos[0], hit.pos[1] + 1, hit.pos[2]];
            let block = ctx.block_id(hit.state);
            if (block == "minecraft:obsidian" || block == "minecraft:bedrock")
                && ctx
                    .level()
                    .and_then(|level| level.block_state(ctx.host, above))
                    .is_some_and(|state| ctx.block_id(state) == "minecraft:air")
            {
                return I::PlaceBlock;
            }
        }
        return I::NoAction;
    }

    if id == "minecraft:flint_and_steel" || id == "minecraft:fire_charge" {
        return fire_item_compute(ctx);
    }

    if id == "minecraft:firework_rocket" {
        if ctx.block().is_some() || ctx.is_gliding() {
            return I::UseItem;
        }
        return I::NoAction;
    }

    if id == "minecraft:fishing_rod" {
        return I::RangedWeapon;
    }

    if id == "minecraft:glass_bottle" {
        if let Some(hit) = ctx.block() {
            let block = ctx.block_id(hit.state);
            if block.ends_with("beehive") || block.ends_with("bee_nest") {
                if ctx
                    .block_int_property(hit.state, "honey_level")
                    .is_some_and(|level| level >= 5)
                    && !ctx.is_sneaking()
                {
                    return I::FillItemFromBlock;
                }
            }
        }
        if glass_bottle_dragon_breath(ctx) {
            return I::FillItemFromEntity;
        }
        if let Some(hit) = ctx.raycast_fluid(false, 5.0) {
            if let Some(fluid) = ctx.fluid_state_at(hit.pos) {
                if !fluid.is_empty() && ctx.fluid_in_tag(fluid.fluid, "minecraft:water") {
                    return I::FillItemFromBlock;
                }
            }
        }
        return I::NoAction;
    }

    if id == "minecraft:honeycomb" {
        if let Some(hit) = ctx.block() {
            let block = ctx.block_id(hit.state);
            if is_waxable(&block) {
                return I::UseItemOnBlock;
            }
        }
        return I::NoAction;
    }

    if id.ends_with("_minecart") || id == "minecraft:minecart" {
        if let Some(hit) = ctx.block() {
            if ctx.block_in_tag(hit.state, "minecraft:rails") {
                return I::PlaceEntity;
            }
        }
        return I::NoAction;
    }

    if id == "minecraft:potion" || id.ends_with("_potion") {
        if let Some(hit) = ctx.block() {
            if hit.face[1] != -1
                && ctx.block_in_tag(hit.state, "minecraft:convertable_to_mud")
                && potion_is_water(ctx, stack)
            {
                return I::UseItemOnBlock;
            }
        }
        return I::UseItem;
    }

    if id.ends_with("_spawn_egg") {
        if ctx.block().is_some() {
            return I::UseItemOnBlock;
        }
        if ctx.entity().is_some() {
            return I::UseItemOnEntity;
        }
        return I::NoAction;
    }

    base_item_compute(ctx, config)
}

/// The original uses `Item.getUseAction` for a bow, crossbow or spear. The
/// override path is shared with the ranged policy.
fn item_override_for_ranged(ctx: &Ctx, config: &Config, _stack: &ItemStack, base: I) -> I {
    let _ = ctx;
    let _ = config;
    base
}

fn potion_is_water(ctx: &Ctx, stack: &ItemStack) -> bool {
    // Water bottles are the only potion whose contents use the water potion
    // key; the component carries it as `minecraft:potion_contents.potion`.
    match ctx.stack_component(stack, "minecraft:potion_contents") {
        Some(mod_api::Tag::Compound(map)) => map.iter().any(|(key, value)| {
            key.to_string_lossy() == "potion"
                && matches!(value, mod_api::Tag::String(id) if id.to_string_lossy().contains("water"))
        }),
        _ => false,
    }
}

fn glass_bottle_dragon_breath(ctx: &Ctx) -> bool {
    let Some(level) = ctx.level() else {
        return false;
    };
    let Some(player) = ctx.entity_bounding_box(ctx.player.entity()) else {
        return false;
    };
    let min = [player[0][0] - 2.0, player[0][1] - 2.0, player[0][2] - 2.0];
    let max = [player[1][0] + 2.0, player[1][1] + 2.0, player[1][2] + 2.0];
    let Some(entities) = level.entities_in(ctx.host, min, max) else {
        return false;
    };
    entities
        .into_iter()
        .any(|entity| ctx.entity_is(entity, "minecraft:area_effect_cloud"))
}

fn bucket_compute(ctx: &Ctx) -> I {
    let Some(stack) = ctx.hand_stack() else {
        return I::NoAction;
    };
    let id = ctx.stack_id(stack);
    let empty = id == "minecraft:bucket";
    let water = id == "minecraft:water_bucket";
    let source = ctx.raycast_fluid(true, 5.0);
    let any_hit = ctx.raycast_fluid(false, 5.0);
    if empty {
        if let Some(hit) = source {
            if let Some(fluid) = ctx.fluid_state_at(hit.pos) {
                if !fluid.is_empty() && fluid.is_source() {
                    return I::FillItemFromBlock;
                }
            }
        }
        // A cauldron holds its fluid as a block, not as a fluid state.
        if let Some(hit) = any_hit {
            let block = ctx.block_id(hit.state);
            if block.contains("cauldron")
                && block != "minecraft:cauldron"
                && !block.contains("powder_snow")
            {
                return I::FillItemFromBlock;
            }
        }
        return I::NoAction;
    }
    if let Some(hit) = any_hit {
        if water {
            let block = ctx.block_id(hit.state);
            if block.ends_with("cauldron") || block.ends_with("waterlogged") {
                return I::FillBlockFromItem;
            }
            if ctx.block_bool_property(hit.state, "waterlogged") {
                return I::FillBlockFromItem;
            }
            if block == "minecraft:water" {
                return I::FillBlockFromItem;
            }
        }
        if id.ends_with("lava_bucket") && ctx.block_id(hit.state) == "minecraft:lava" {
            return I::FillBlockFromItem;
        }
        return I::PlaceBlock;
    }
    if let Some(entity) = ctx.entity() {
        if water {
            // Bucketable mobs: axolotl, cod, salmon, tropical fish, pufferfish,
            // tadpole. The client cannot read the mob's NBT, so the kind is
            // the practical test.
            let ty = ctx.entity_type_id(entity).unwrap_or_default();
            if any(
                &ty,
                &[
                    "minecraft:axolotl",
                    "minecraft:cod",
                    "minecraft:salmon",
                    "minecraft:tropical_fish",
                    "minecraft:pufferfish",
                    "minecraft:tadpole",
                ],
            ) {
                return I::PickUpEntity;
            }
        }
    }
    I::PlaceBlock
}

fn fire_item_compute(ctx: &Ctx) -> I {
    let Some(hit) = ctx.block() else {
        return I::NoAction;
    };
    let block = ctx.block_id(hit.state);
    let lit = ctx.block_bool_property(hit.state, "lit");
    if !lit
        && (block.ends_with("campfire")
            || block.ends_with("candle")
            || block.ends_with("candle_cake"))
    {
        return I::UseItemOnBlock;
    }
    let neighbour = [
        hit.pos[0] + hit.face[0],
        hit.pos[1] + hit.face[1],
        hit.pos[2] + hit.face[2],
    ];
    if let Some(level) = ctx.level() {
        if let Some(state) = level.block_state(ctx.host, neighbour) {
            if ctx.host.block_states().replaceable(state).unwrap_or(false) {
                return I::PlaceBlock;
            }
        }
    }
    I::NoAction
}

fn axe_transforms(ctx: &Ctx, state: BlockState) -> bool {
    let block = ctx.block_id(state);
    // Stripping logs/wood, scraping oxidation, and waxing/unwaxing copper.
    block.ends_with("_log")
        || block.ends_with("_wood")
        || block.ends_with("_stem")
        || block.ends_with("_hyphae")
        || (block.contains("copper") && !block.contains("_ore") && !block.contains("raw_"))
}

fn is_fertile_or_tillable(ctx: &Ctx) -> bool {
    let Some(hit) = ctx.block() else {
        return false;
    };
    let block = ctx.block_id(hit.state);
    // Hoes till dirt-like blocks and make farmland from grass/dirt paths.
    block.ends_with("dirt")
        || block == "minecraft:grass_block"
        || block.ends_with("dirt_path")
        || block.ends_with("rooted_dirt")
        || block.ends_with("coarse_dirt")
}

fn is_path_or_flattenable(ctx: &Ctx) -> bool {
    let Some(hit) = ctx.block() else {
        return false;
    };
    let block = ctx.block_id(hit.state);
    if (block == "minecraft:grass_block" || block == "minecraft:dirt_path") && hit.pos[1] < 319 {
        if let Some(level) = ctx.level() {
            if let Some(above) =
                level.block_state(ctx.host, [hit.pos[0], hit.pos[1] + 1, hit.pos[2]])
            {
                if ctx.block_id(above) == "minecraft:air" {
                    return true;
                }
            }
        }
    }
    block.ends_with("campfire") && ctx.block_bool_property(hit.state, "lit")
}

fn is_growing_stem(ctx: &Ctx) -> bool {
    let Some(hit) = ctx.block() else {
        return false;
    };
    let block = ctx.block_id(hit.state);
    (block.contains("kelp")
        || block.contains("cave_vines")
        || block.contains("weeping_vines")
        || block.contains("twisting_vines"))
        && ctx
            .block_int_property(hit.state, "age")
            .is_some_and(|age| age < 25)
}

fn is_full_beehive(ctx: &Ctx) -> bool {
    ctx.block()
        .and_then(|hit| ctx.block_int_property(hit.state, "honey_level"))
        .is_some_and(|level| level >= 5)
}

fn is_waxable(block: &str) -> bool {
    block.contains("copper") && !block.contains("waxed")
}

// ---------------------------------------------------------------------------
// Entity helpers

fn tag_food(ctx: &Ctx, stack: &ItemStack, tag: &str) -> bool {
    ctx.stack_in_tag(stack, tag)
}

fn breeding_item(ctx: &Ctx, stack: &ItemStack, ty: &str) -> bool {
    let tag = match ty {
        "minecraft:cow" | "minecraft:mooshroom" => "minecraft:cow_food",
        "minecraft:sheep" => "minecraft:sheep_food",
        "minecraft:pig" => "minecraft:pig_food",
        "minecraft:chicken" => "minecraft:chicken_food",
        "minecraft:rabbit" => "minecraft:rabbit_food",
        "minecraft:turtle" => "minecraft:turtle_food",
        "minecraft:panda" => "minecraft:panda_food",
        "minecraft:fox" => "minecraft:fox_food",
        "minecraft:ocelot" => "minecraft:ocelot_food",
        "minecraft:axolotl" => "minecraft:axolotl_food",
        "minecraft:frog" => "minecraft:frog_food",
        "minecraft:sniffer" => "minecraft:sniffer_food",
        "minecraft:armadillo" => "minecraft:armadillo_food",
        "minecraft:bee" => "minecraft:bee_food",
        "minecraft:camel" => "minecraft:camel_food",
        "minecraft:goat" => "minecraft:goat_food",
        "minecraft:horse"
        | "minecraft:donkey"
        | "minecraft:mule"
        | "minecraft:skeleton_horse"
        | "minecraft:zombie_horse"
        | "minecraft:llama"
        | "minecraft:trader_llama" => "minecraft:horse_food",
        "minecraft:wolf" => "minecraft:wolf_food",
        "minecraft:cat" => "minecraft:cat_food",
        "minecraft:parrot" => "minecraft:parrot_food",
        "minecraft:hoglin" | "minecraft:zoglin" => "minecraft:hoglin_food",
        "minecraft:strider" => "minecraft:strider_food",
        _ => return false,
    };
    tag_food(ctx, stack, tag) || tag_food(ctx, stack, "c:foods")
}

fn is_leashable(ty: &str) -> bool {
    !any(
        ty,
        &[
            "minecraft:player",
            "minecraft:armor_stand",
            "minecraft:item_frame",
            "minecraft:glow_item_frame",
            "minecraft:painting",
            "minecraft:leash_knot",
        ],
    )
}

fn has_custom_name(ctx: &Ctx, stack: &ItemStack) -> bool {
    ctx.stack_has_component(stack, "minecraft:custom_name")
}

fn entity_compute(ctx: &Ctx, _config: &Config) -> I {
    let Some(entity) = ctx.entity() else {
        return I::Empty;
    };
    let ty = ctx.entity_type_id(entity).unwrap_or_default();
    let hand = ctx.hand_stack().cloned();
    let id = hand
        .as_ref()
        .map(|stack| ctx.stack_id(stack))
        .unwrap_or_default();

    // Always-interactable tags first, as the original base entity does for the
    // subclasses that do not override it. Specific rules below replace it.
    let tagged = ctx.entity_type_in_tag(entity, "dynamiccrosshair:always_interactable");

    if any(
        &ty,
        &[
            "minecraft:leash_knot",
            "minecraft:chest_minecart",
            "minecraft:hopper_minecart",
        ],
    ) {
        return I::InteractWithEntity;
    }

    if ty == "minecraft:item_frame" || ty == "minecraft:glow_item_frame" {
        return if hand.is_some() {
            I::PlaceItemOnEntity
        } else {
            I::NoAction
        };
    }

    if ty == "minecraft:allay" {
        return if hand.is_some() {
            I::PlaceItemOnEntity
        } else {
            I::TakeItemFromEntity
        };
    }

    if ty == "minecraft:armor_stand" {
        return armor_stand_compute(ctx, hand.as_ref());
    }

    if ty.ends_with("_boat")
        || ty.ends_with("_raft")
        || ty.ends_with("_chest_boat")
        || ty.ends_with("_chest_raft")
    {
        if ty.contains("chest") {
            return I::InteractWithEntity;
        }
        return if !ctx.is_sneaking() {
            I::MountEntity
        } else {
            I::NoAction
        };
    }

    if ty.ends_with("_minecart") {
        return I::MountEntity;
    }

    if any(
        &ty,
        &[
            "minecraft:horse",
            "minecraft:donkey",
            "minecraft:mule",
            "minecraft:skeleton_horse",
            "minecraft:zombie_horse",
            "minecraft:camel",
            "minecraft:llama",
            "minecraft:trader_llama",
        ],
    ) {
        return horse_compute(ctx, entity, &ty, hand.as_ref());
    }

    if ty == "minecraft:cat" {
        if let Some(stack) = hand.as_ref() {
            if id.ends_with("_dye")
                || id == "minecraft:lead"
                || has_custom_name(ctx, stack)
                || breeding_item(ctx, stack, &ty)
            {
                return I::UseItemOnEntity;
            }
        }
        return I::InteractWithEntity;
    }

    if ty == "minecraft:wolf" {
        if id == "minecraft:bone"
            || id.ends_with("_dye")
            || id == "minecraft:shears"
            || id == "minecraft:wolf_armor"
        {
            return I::UseItemOnEntity;
        }
        if let Some(stack) = hand.as_ref() {
            if breeding_item(ctx, stack, &ty) {
                return I::UseItemOnEntity;
            }
        }
        return I::InteractWithEntity;
    }

    if ty == "minecraft:parrot" {
        if let Some(stack) = hand.as_ref() {
            if id == "minecraft:cookie"
                || id.ends_with("_seeds")
                || tag_food(ctx, stack, "minecraft:parrot_food")
            {
                return I::UseItemOnEntity;
            }
        }
        return I::InteractWithEntity;
    }

    if ty == "minecraft:villager" || ty == "minecraft:wandering_trader" {
        return I::InteractWithEntity;
    }

    if any(&ty, &["minecraft:cow", "minecraft:goat"]) {
        if id == "minecraft:bucket" && !ctx.entity_is_baby(entity) {
            return I::UseItemOnEntity;
        }
        if let Some(stack) = hand.as_ref() {
            if breeding_item(ctx, stack, &ty) {
                return I::UseItemOnEntity;
            }
        }
    }

    if any(
        &ty,
        &[
            "minecraft:sheep",
            "minecraft:mooshroom",
            "minecraft:snow_golem",
            "minecraft:bogged",
        ],
    ) {
        if id == "minecraft:shears" {
            return I::UseItemOnEntity;
        }
        if let Some(stack) = hand.as_ref() {
            if breeding_item(ctx, stack, &ty) {
                return I::UseItemOnEntity;
            }
        }
    }

    if ty == "minecraft:creeper" && id == "minecraft:flint_and_steel" {
        return I::UseItemOnEntity;
    }

    if ty == "minecraft:dolphin"
        && hand
            .as_ref()
            .is_some_and(|stack| ctx.stack_in_tag(stack, "minecraft:fishes"))
    {
        return I::UseItemOnEntity;
    }

    if ty == "minecraft:furnace_minecart"
        && hand
            .as_ref()
            .is_some_and(|stack| ctx.stack_in_tag(stack, "minecraft:furnace_minecart_fuel"))
    {
        return I::UseItemOnEntity;
    }

    if ty == "minecraft:iron_golem" && id == "minecraft:iron_ingot" {
        return I::UseItemOnEntity;
    }

    if ty == "minecraft:panda" {
        // The original returns INTERACT only while the panda lies on its back;
        // the client cannot read that flag, so a panda is always interactive.
        return I::InteractWithEntity;
    }

    if ty == "minecraft:zombie_villager" && id == "minecraft:golden_apple" {
        return I::UseItemOnEntity;
    }

    if ty == "minecraft:copper_golem" {
        if id == "minecraft:shears" || id == "minecraft:honeycomb" || id.ends_with("_axe") {
            return I::UseItemOnEntity;
        }
    }

    if let Some(stack) = hand.as_ref() {
        if breeding_item(ctx, stack, &ty) {
            return I::UseItemOnEntity;
        }
    }

    if id.ends_with("_spawn_egg") {
        return I::UseItemOnEntity;
    }
    if id == "minecraft:lead" && is_leashable(&ty) {
        return I::UseItemOnEntity;
    }
    if id == "minecraft:name_tag"
        && hand
            .as_ref()
            .is_some_and(|stack| has_custom_name(ctx, stack))
    {
        return I::UseItemOnEntity;
    }

    if tagged {
        return I::InteractWithEntity;
    }
    I::Empty
}

fn armor_stand_compute(ctx: &Ctx, hand: Option<&ItemStack>) -> I {
    if !ctx.is_main_hand() {
        return I::NoAction;
    }
    match hand {
        None => I::TakeItemFromEntity,
        Some(stack) => {
            let id = ctx.stack_id(stack);
            if id == "minecraft:name_tag" {
                if has_custom_name(ctx, stack) {
                    return I::UseItemOnEntity;
                }
                I::NoAction
            } else if let Some(slot) = equip_slot(ctx, stack) {
                if slot != EquipmentSlot::OffHand {
                    I::PlaceItemOnEntity
                } else {
                    I::NoAction
                }
            } else {
                I::PlaceItemOnEntity
            }
        }
    }
}

fn horse_compute(ctx: &Ctx, _entity: mod_api::EntityRef, _ty: &str, hand: Option<&ItemStack>) -> I {
    if ctx.entity_passengers(_entity) > 0 {
        return I::Empty;
    }
    if ctx.is_sneaking() {
        return I::InteractWithEntity;
    }
    let Some(stack) = hand else {
        return I::MountEntity;
    };
    let id = ctx.stack_id(stack);
    if id == "minecraft:saddle" || id == "minecraft:chest" {
        return I::PlaceItemOnEntity;
    }
    if ctx.item_has_component(stack.item(), "minecraft:equippable") {
        if let Some(slot) = component_slot(ctx, stack) {
            if slot == EquipmentSlot::Body && ctx.entity_equipment(_entity, slot).is_none() {
                return I::PlaceItemOnEntity;
            }
        }
    }
    I::MountEntity
}

// ---------------------------------------------------------------------------
// Blocks

fn is_potted(block: &str) -> bool {
    block.starts_with("minecraft:potted_")
}

fn item_is_pottable(ctx: &Ctx, stack: &ItemStack) -> bool {
    let Some(block) = ctx.host.item_block(stack.item()) else {
        return false;
    };
    let id = block.id(ctx.host);
    let path = id.strip_prefix("minecraft:").unwrap_or(&id);
    ctx.host
        .block_key(&format!("minecraft:potted_{path}"))
        .is_some()
}

fn sign_compute(ctx: &Ctx) -> I {
    let id = ctx
        .hand_stack()
        .map(|stack| ctx.stack_id(stack))
        .unwrap_or_default();
    if id.contains("dye")
        || id == "minecraft:glow_ink_sac"
        || id == "minecraft:ink_sac"
        || id == "minecraft:honeycomb"
    {
        return I::UseItemOnEntity;
    }
    I::InteractWithEntity
}

fn block_compute(ctx: &Ctx, config: &Config) -> I {
    let Some(hit) = ctx.block() else {
        return I::Empty;
    };
    let block = ctx.block_id(hit.state);
    let hand = ctx.hand_stack().cloned();
    let id = hand
        .as_ref()
        .map(|stack| ctx.stack_id(stack))
        .unwrap_or_default();

    // Fire on candles and campfires only when the held item has no use of its own.
    if block.contains("candle") && ctx.block_bool_property(hit.state, "lit") && ctx.is_main_hand() {
        let item = item_compute(ctx, config);
        return match item {
            I::UseItem
            | I::UseItemOnBlock
            | I::ConsumeItem
            | I::ChargeItem
            | I::EquipItem
            | I::Spyglass
            | I::PlaceBlock
            | I::NoAction => I::NoAction,
            _ => I::InteractWithBlock,
        };
    }
    if block.contains("candle") && !ctx.block_bool_property(hit.state, "lit") {
        return I::NoAction;
    }

    if block.contains("cauldron") {
        if block == "minecraft:water_cauldron" {
            if id == "minecraft:glass_bottle" || id == "minecraft:potion" {
                return if id == "minecraft:glass_bottle" {
                    I::FillItemFromBlock
                } else {
                    I::FillBlockFromItem
                };
            }
            if id.ends_with("shulker_box") || id.ends_with("_dye") || id.contains("banner") {
                return I::UseItemOnBlock;
            }
        }
        if id.contains("powder_snow_bucket") {
            return I::FillBlockFromItem;
        }
        return I::NoAction;
    }

    if block.ends_with("_sign") || block.ends_with("_hanging_sign") || block == "minecraft:sign" {
        return sign_compute(ctx);
    }

    if block == "minecraft:cake" || block.ends_with("_candle_cake") {
        if ctx.can_eat(false) && ctx.should_interact() {
            return I::UseBlock;
        }
        return I::NoAction;
    }

    if block.ends_with("campfire") {
        if let Some(stack) = hand.as_ref() {
            let item_id = ctx.stack_id(stack);
            if ctx.stack_in_tag(stack, "minecraft:campfire_input")
                || ctx.item_has_component(stack.item(), "minecraft:food")
                || item_id == "minecraft:campfire"
            {
                return I::PlaceItemOnBlock;
            }
        }
        return I::NoAction;
    }

    if block.contains("cave_vines") {
        if ctx.block_bool_property(hit.state, "berries") {
            return I::TakeItemFromBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:chiseled_bookshelf" {
        for slot in 0..6 {
            if ctx.block_bool_property(hit.state, &format!("slot_{slot}_occupied")) {
                return I::TakeItemFromBlock;
            }
        }
        if hand
            .as_ref()
            .is_some_and(|stack| ctx.stack_in_tag(stack, "minecraft:bookshelf_books"))
        {
            return I::PlaceItemOnBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:composter" {
        if let Some(stack) = hand.as_ref() {
            if ctx.host.item_compost_chance(stack.item()).is_some() {
                return I::UseItemOnBlock;
            }
        }
        return I::NoAction;
    }

    if block == "minecraft:copper_golem_statue" {
        return if id.ends_with("_axe") {
            I::UseItemOnBlock
        } else {
            I::InteractWithBlock
        };
    }

    if block == "minecraft:decorated_pot" {
        return if hand.is_some() {
            I::PlaceItemOnBlock
        } else {
            I::InteractWithBlock
        };
    }

    if block.ends_with("_door") {
        return if block.contains("iron") {
            I::Empty
        } else {
            I::InteractWithBlock
        };
    }

    if block.ends_with("_trapdoor") {
        return if block.contains("iron") {
            I::Empty
        } else {
            I::InteractWithBlock
        };
    }

    if block == "minecraft:end_portal_frame" {
        if !ctx.block_bool_property(hit.state, "eye") && id == "minecraft:ender_eye" {
            return I::PlaceItemOnBlock;
        }
        return I::NoAction;
    }

    if block.ends_with("_fence") || block == "minecraft:fence" {
        if let Some(level) = ctx.level() {
            let min = [
                f64::from(hit.pos[0]) - 7.0,
                f64::from(hit.pos[1]) - 7.0,
                f64::from(hit.pos[2]) - 7.0,
            ];
            let max = [
                f64::from(hit.pos[0]) + 7.0,
                f64::from(hit.pos[1]) + 7.0,
                f64::from(hit.pos[2]) + 7.0,
            ];
            if let Some(entities) = level.entities_in(ctx.host, min, max) {
                let held = entities.iter().any(|entity| {
                    matches!(
                        ctx.entity_leash_holder(*entity),
                        Some(mod_api::LeashHolder::Entity(holder)) if holder == ctx.player_entity()
                    )
                });
                if held {
                    return I::UseItemOnBlock;
                }
            }
        }
    }

    if is_potted(&block) {
        if hand
            .as_ref()
            .is_some_and(|stack| item_is_pottable(ctx, stack))
        {
            return I::NoAction;
        }
        return I::TakeItemFromBlock;
    }
    if block == "minecraft:flower_pot" {
        if hand
            .as_ref()
            .is_some_and(|stack| item_is_pottable(ctx, stack))
        {
            return I::PlaceItemOnBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:jukebox" {
        if ctx.block_bool_property(hit.state, "has_record") {
            return I::TakeItemFromBlock;
        }
        if hand
            .as_ref()
            .is_some_and(|stack| ctx.stack_has_component(stack, "minecraft:jukebox_playable"))
        {
            return I::PlaceItemOnBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:lectern" {
        if ctx.block_bool_property(hit.state, "has_book") && !ctx.is_sneaking() {
            return I::InteractWithBlock;
        }
        if id == "minecraft:written_book" || id == "minecraft:writable_book" {
            return I::PlaceItemOnBlock;
        }
        return I::NoAction;
    }

    if block.ends_with("redstone_ore") {
        if ctx.should_interact() {
            return I::InteractWithBlock;
        }
        return I::Empty;
    }

    if block == "minecraft:redstone_wire" {
        let none = ["north", "south", "east", "west"]
            .iter()
            .any(|side| ctx.block_property(hit.state, side).as_deref() == Some("none"));
        let side = ["north", "south", "east", "west"]
            .iter()
            .any(|direction| ctx.block_property(hit.state, direction).as_deref() == Some("side"));
        if none || side {
            return I::InteractWithBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:shelf" || block.ends_with("_shelf") {
        if ctx.is_main_hand() {
            if hand.is_some() {
                return I::PlaceItemOnBlock;
            }
            return I::TakeItemFromBlock;
        }
        return I::Empty;
    }

    if block == "minecraft:sweet_berry_bush" {
        if ctx
            .block_int_property(hit.state, "age")
            .is_some_and(|age| age > 1)
        {
            return I::TakeItemFromBlock;
        }
        return I::NoAction;
    }

    if block == "minecraft:vault" {
        if ctx.block_property(hit.state, "vault_state").as_deref() == Some("active")
            && hand.is_some()
        {
            return I::UseItemOnBlock;
        }
        return I::NoAction;
    }

    if config.enable_tweaks && extra_block(ctx, hit.state, &config.extra.interactable_blocks) {
        return I::InteractWithBlock;
    }

    // Generic always-interactable blocks from the API tag.
    if ctx.block_in_tag(hit.state, "dynamiccrosshair:always_interactable") {
        return I::InteractWithBlock;
    }
    if ctx.is_creative()
        && ctx.block_in_tag(
            hit.state,
            "dynamiccrosshair:always_interactable_in_creative_mode",
        )
    {
        return I::InteractWithBlock;
    }

    // A block the original's `isInteractable` tag names is only a target
    // highlight; the per-block rules above decide the action.
    I::Empty
}

// ---------------------------------------------------------------------------
// The DynamicCrosshairBaseItem / VanillaApiImpl pipeline

fn base_with_policies(ctx: &Ctx, config: &Config) -> I {
    let base = item_compute(ctx, config);
    match base {
        I::MeleeWeapon => {
            if !include_melee_weapon(ctx, config) {
                I::Empty
            } else {
                base
            }
        }
        I::RangedWeapon | I::RangedWeaponCharging | I::RangedWeaponCharged => {
            if !include_ranged_weapon(config) {
                I::Empty
            } else {
                base
            }
        }
        I::PlaceBlock => {
            if ctx
                .hand_stack()
                .is_some_and(|stack| include_holding_block(ctx, config, stack))
            {
                base
            } else {
                I::Empty
            }
        }
        I::Shield => {
            if include_shield(ctx, config) {
                base
            } else {
                I::Empty
            }
        }
        I::Tool => {
            if include_tool(ctx, config) {
                base
            } else {
                I::Empty
            }
        }
        I::UsableTool => {
            if include_tool(ctx, config) {
                I::UseItemOnBlock
            } else {
                I::Empty
            }
        }
        I::UseItem
        | I::UseItemOnBlock
        | I::UseItemOnEntity
        | I::ChargeItem
        | I::EquipItem
        | I::ConsumeItem
        | I::Spyglass => {
            if include_usable_item(ctx, config) {
                base
            } else {
                I::Empty
            }
        }
        I::ThrowItem => {
            if include_throwable(ctx, config) {
                base
            } else {
                I::Empty
            }
        }
        _ => base,
    }
}

pub fn build_from_item(ctx: &Ctx, config: &Config) -> Option<Crosshair> {
    let base = base_with_policies(ctx, config);
    override_from_item(ctx, config, base)
}

/// `VanillaApiImpl.overrideFromItem`.
fn override_from_item(ctx: &Ctx, config: &Config, mut interaction: I) -> Option<Crosshair> {
    let stack = ctx.hand_stack().cloned()?;
    let mut is_ranged_melee_weapon = false;

    if is_melee_weapon(ctx, &stack, config)
        && include_melee_weapon(ctx, config)
        && interaction != I::UsableTool
    {
        if ctx.block().is_some() && config.holding_tool != Policy::Disabled {
            let hit = ctx.block().unwrap();
            if ctx.block_destroy_speed(hit.state) == Some(0.0) {
                return Some(if config.melee_weapon_on_breakable_block {
                    Crosshair::of(I::MeleeWeapon)
                } else {
                    Crosshair::of(I::CorrectTool)
                });
            }
        }
        if ctx.entity().is_some() && !config.melee_weapon_on_entity {
            return Some(Crosshair::of(I::NoAction));
        }
        if is_ranged_weapon(ctx, &stack, config) {
            is_ranged_melee_weapon = true;
            interaction = I::RangedWeapon;
        }
    }

    match interaction {
        I::Tool | I::UsableTool => {
            let crosshair = Crosshair::of(interaction);
            if ctx.block().is_some() {
                return Some(
                    crosshair.combine(Some(Crosshair::of(check_tool_with_block(ctx, config)))),
                );
            }
            Some(crosshair)
        }
        I::RangedWeapon => match config.holding_ranged_weapon {
            UsablePolicy::Disabled => None,
            UsablePolicy::Always => Some(Crosshair::of(I::RangedWeapon)),
            UsablePolicy::IfInteractable => {
                let charged = ranged_is_charged(ctx, &stack);
                let charging = ranged_is_charging(ctx, &stack);
                if charged {
                    Some(if is_ranged_melee_weapon {
                        Crosshair::pair(I::MeleeWeapon, I::RangedWeaponCharged)
                    } else {
                        Crosshair::of(I::RangedWeaponCharged)
                    })
                } else if charging {
                    Some(if is_ranged_melee_weapon {
                        Crosshair::pair(I::MeleeWeapon, I::RangedWeaponCharging)
                    } else {
                        Crosshair::of(I::RangedWeaponCharging)
                    })
                } else if is_ranged_melee_weapon {
                    Some(Crosshair::of(I::MeleeWeapon))
                } else {
                    Some(Crosshair::of(I::ForceRegularCrosshair))
                }
            }
        },
        I::Empty => {
            if config.holding_usable_item != UsablePolicy::Disabled {
                if (config.holding_usable_item == UsablePolicy::Always || !ctx.is_cooling_down())
                    && is_always_usable(ctx, &stack, config)
                {
                    return Some(Crosshair::of(I::UseItem));
                }
                if config.holding_usable_item == UsablePolicy::Always
                    && is_usable(ctx, &stack, config)
                {
                    return Some(Crosshair::of(I::UseItem));
                }
            }
            None
        }
        _ => Some(Crosshair::of(interaction)),
    }
}

fn ranged_is_charging(ctx: &Ctx, stack: &ItemStack) -> bool {
    let Some(use_state) = ctx.host.local_item_use() else {
        return false;
    };
    if use_state.hand != ctx.hand {
        return false;
    }
    let id = ctx.stack_id(stack);
    if is_bow(ctx, stack) && id != "minecraft:crossbow" {
        use_state.ticks_used > 0
    } else if is_trident(ctx, stack) {
        use_state.ticks_used > 0
    } else {
        use_state.ticks_used > 0
    }
}

fn ranged_is_charged(ctx: &Ctx, stack: &ItemStack) -> bool {
    let id = ctx.stack_id(stack);
    if is_bow(ctx, stack) && id != "minecraft:crossbow" {
        let Some(use_state) = ctx.host.local_item_use() else {
            return false;
        };
        if use_state.hand != ctx.hand {
            return false;
        }
        let pull = use_state.ticks_used;
        return pull >= 20;
    }
    if is_crossbow(ctx, stack) {
        return !ctx.stack_component_is_empty_list(stack, "minecraft:charged_projectiles");
    }
    if is_trident(ctx, stack) {
        return ctx
            .host
            .local_item_use()
            .is_some_and(|use_state| use_state.hand == ctx.hand && use_state.ticks_used > 10);
    }
    true
}

// ---------------------------------------------------------------------------
// Aggregate

pub fn build_from_entity(ctx: &Ctx, config: &Config) -> Option<Crosshair> {
    let interaction = entity_compute(ctx, config);
    if interaction != I::Empty {
        Some(Crosshair::of(interaction))
    } else {
        None
    }
}

pub fn build_from_block(ctx: &Ctx, config: &Config) -> Option<Crosshair> {
    let interaction = block_compute(ctx, config);
    if interaction != I::Empty {
        Some(Crosshair::of(interaction))
    } else {
        None
    }
}

pub fn is_block_interactable(ctx: &Ctx, config: &Config) -> bool {
    let Some(hit) = ctx.block() else {
        return false;
    };
    if !ctx.should_interact() {
        return false;
    }
    if config.enable_tweaks && extra_block(ctx, hit.state, &config.extra.interactable_blocks) {
        return true;
    }
    if ctx.block_in_tag(hit.state, "dynamiccrosshair:always_interactable") {
        return true;
    }
    if ctx.is_creative()
        && ctx.block_in_tag(
            hit.state,
            "dynamiccrosshair:always_interactable_in_creative_mode",
        )
    {
        return true;
    }
    if ctx.block_in_tag(hit.state, "dynamiccrosshair:interactable") {
        return true;
    }
    let block = ctx.block_id(hit.state);
    (block.ends_with("_door") && !block.contains("iron"))
        || (block.ends_with("_trapdoor") && !block.contains("iron"))
        || block == "minecraft:redstone_wire"
        || block.ends_with("candle")
        || block.contains("cauldron")
        || block.ends_with("_sign")
        || block.ends_with("_hanging_sign")
        || is_potted(&block)
        || block == "minecraft:flower_pot"
        || block == "minecraft:cake"
        || block.ends_with("campfire")
        || block.contains("cave_vines")
        || block == "minecraft:chiseled_bookshelf"
        || block == "minecraft:composter"
        || block == "minecraft:decorated_pot"
        || block == "minecraft:jukebox"
        || block == "minecraft:lectern"
        || block == "minecraft:sweet_berry_bush"
        || block == "minecraft:vault"
        || block == "minecraft:shelf"
        || block.ends_with("_shelf")
        || block.ends_with("_fence")
}

/// The extra item and block names from `config/dynamiccrosshair.json`.
pub fn extra_contains(list: &[String], id: &str, tag_lookup: impl Fn(&str) -> bool) -> bool {
    list.iter().any(|entry| match entry.strip_prefix('#') {
        Some(tag) => tag_lookup(tag),
        None => {
            let entry = entry.strip_prefix("minecraft:").unwrap_or(entry);
            let id = id.strip_prefix("minecraft:").unwrap_or(id);
            entry == id
        }
    })
}
