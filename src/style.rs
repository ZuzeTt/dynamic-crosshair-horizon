//! Which sprite parts make up the crosshair for the current interaction.

use crate::config::{Config, StyleSlot};
use crate::interaction::{Crosshair, InteractionType as I};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    None,
    Regular,
    OnBlock,
    OnEntity,
    HoldingTool,
    HoldingMeleeWeapon,
    HoldingRangedWeapon,
    HoldingThrowable,
    HoldingBlock,
    HoldingUsableItem,
    HoldingShield,
    CanInteract,
}

impl Variant {
    fn slot(self) -> Option<StyleSlot> {
        Some(match self {
            Self::None => return None,
            Self::Regular => StyleSlot::Regular,
            Self::OnBlock => StyleSlot::OnBlock,
            Self::OnEntity => StyleSlot::OnEntity,
            Self::HoldingTool => StyleSlot::HoldingTool,
            Self::HoldingMeleeWeapon => StyleSlot::HoldingMeleeWeapon,
            Self::HoldingRangedWeapon => StyleSlot::HoldingRangedWeapon,
            Self::HoldingThrowable => StyleSlot::HoldingThrowable,
            Self::HoldingBlock => StyleSlot::HoldingBlock,
            Self::HoldingUsableItem => StyleSlot::UseItem,
            Self::HoldingShield => StyleSlot::Shield,
            Self::CanInteract => StyleSlot::Interact,
        })
    }
}

/// One sprite to draw, in draw order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    pub sprite: String,
    pub color: u32,
    pub blend: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Component {
    pub blend: Vec<Part>,
    pub no_blend: Vec<Part>,
    /// The computed crosshair is exactly the vanilla sprite with the usual
    /// inverting blend, so the vanilla HUD can draw it (and its attack
    /// indicator) itself.
    pub vanilla_exact: bool,
}

impl Component {
    fn push(&mut self, part: Part) {
        if part.blend {
            self.blend.push(part);
        } else {
            self.no_blend.push(part);
        }
    }
}

fn style_part(config: &Config, slot: StyleSlot) -> Part {
    let (sprite, color, blend, _) = config.resolved_style(slot);
    Part {
        sprite,
        color: color | 0xFF00_0000,
        blend,
    }
}

fn coalesce(config: &Config, slot: StyleSlot) -> bool {
    config.style_for(slot).coalesce
}

/// `CrosshairComponent` from the original.
pub fn component(crosshair: Crosshair, config: &Config) -> Component {
    let mut primary = Variant::Regular;
    let mut secondary = Variant::None;
    let mut modifier = None;

    match crosshair.primary {
        I::TargetBlock => primary = Variant::OnBlock,
        I::TargetEntity => primary = Variant::OnEntity,
        I::MeleeWeapon => primary = Variant::HoldingMeleeWeapon,
        I::Tool => primary = Variant::HoldingTool,
        I::CorrectTool => {
            primary = Variant::HoldingTool;
            modifier = Some(StyleSlot::ModCorrectTool);
        }
        I::IncorrectTool => {
            primary = Variant::HoldingTool;
            modifier = Some(StyleSlot::ModIncorrectTool);
        }
        _ => {}
    }

    if crosshair.primary != I::ForceRegularCrosshair {
        match crosshair.secondary {
            I::UseItem
            | I::EquipItem
            | I::ConsumeItem
            | I::ChargeItem
            | I::UseItemOnBlock
            | I::PlaceItemOnBlock
            | I::FillItemFromBlock
            | I::FillBlockFromItem
            | I::UseBlock
            | I::UseItemOnEntity
            | I::PickUpBlock
            | I::PickUpEntity
            | I::PlaceItemOnEntity
            | I::FillItemFromEntity
            | I::FillEntityFromItem => secondary = Variant::HoldingUsableItem,
            I::Spyglass => {
                if config.force_holding_spyglass {
                    primary = Variant::Regular;
                } else {
                    secondary = Variant::HoldingUsableItem;
                }
            }
            I::ThrowItem => secondary = Variant::HoldingThrowable,
            I::PlaceBlock | I::PlaceEntity => secondary = Variant::HoldingBlock,
            I::InteractWithBlock
            | I::TakeItemFromBlock
            | I::MountBlock
            | I::InteractWithEntity
            | I::TakeItemFromEntity
            | I::MountEntity
            | I::PickUpItem => secondary = Variant::CanInteract,
            I::RangedWeaponCharging => primary = Variant::Regular,
            I::RangedWeapon | I::RangedWeaponCharged => secondary = Variant::HoldingRangedWeapon,
            I::UsableTool => {
                primary = Variant::HoldingTool;
                secondary = Variant::HoldingUsableItem;
            }
            I::Shield => secondary = Variant::HoldingShield,
            I::ForceRegularCrosshair => modifier = None,
            _ => {}
        }
    }

    let mut primary_slot = primary.slot();
    let mut secondary_slot = secondary.slot();

    if let Some(primary_slot_value) = primary_slot {
        if let Some(secondary_slot_value) = secondary_slot {
            if !coalesce(config, secondary_slot_value) {
                primary_slot = None;
            } else if !coalesce(config, primary_slot_value) {
                secondary_slot = None;
            }
        }
    }

    let mut component = Component::default();

    let mut parts = Vec::new();
    match primary_slot {
        Some(slot) => parts.push(style_part(config, slot)),
        None if secondary_slot.is_none() => {
            parts.push(style_part(config, StyleSlot::Regular));
        }
        None => {}
    }
    if let Some(slot) = secondary_slot {
        parts.push(style_part(config, slot));
    }
    if config.display_correct_tool {
        if let Some(slot) = modifier {
            parts.push(style_part(config, slot));
        }
    }
    for part in parts {
        component.push(part);
    }

    component.vanilla_exact = component.blend.len() == 1
        && component.no_blend.is_empty()
        && component.blend[0].sprite == "minecraft:hud/crosshair"
        && component.blend[0].color == 0xFFFF_FFFF
        && component.blend[0].blend;

    component
}

/// The style used when `dynamic_crosshair_style` is off: the original's
/// `getDefaultStyle()`, i.e. the vanilla sprite with the global colour.
pub fn default_style(config: &Config) -> Part {
    Part {
        sprite: "minecraft:hud/crosshair".into(),
        color: if config.global_override_color {
            config.global_color | 0xFF00_0000
        } else {
            0xFFFF_FFFF
        },
        blend: config.global_blend,
    }
}

pub fn default_is_vanilla(config: &Config) -> bool {
    let style = default_style(config);
    style.sprite == "minecraft:hud/crosshair" && style.color == 0xFFFF_FFFF && style.blend
}
