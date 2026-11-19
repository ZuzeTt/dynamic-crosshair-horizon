//! The crosshair model, ported from the original mod's API types.

/// What the player's hands and the targeted block or entity can do.
///
/// The names and the primary/secondary mapping follow `InteractionType` in
/// DynamicCrosshair, so the rest of the port reads like the Java source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionType {
    UseItem,
    EquipItem,
    ConsumeItem,
    ThrowItem,
    ChargeItem,
    Spyglass,
    PickUpItem,

    TargetBlock,
    PlaceBlock,
    InteractWithBlock,
    PickUpBlock,
    UseItemOnBlock,
    PlaceItemOnBlock,
    TakeItemFromBlock,
    FillItemFromBlock,
    FillBlockFromItem,
    UseBlock,
    MountBlock,

    TargetEntity,
    PlaceEntity,
    InteractWithEntity,
    PickUpEntity,
    UseItemOnEntity,
    PlaceItemOnEntity,
    TakeItemFromEntity,
    FillItemFromEntity,
    FillEntityFromItem,
    MountEntity,

    MeleeWeapon,
    RangedWeapon,
    RangedWeaponCharging,
    RangedWeaponCharged,
    UsableTool,
    Tool,
    CorrectTool,
    IncorrectTool,
    Shield,

    /// Force a regular crosshair to be displayed.
    ForceRegularCrosshair,
    /// No interaction specified, use tags for evaluation.
    Empty,
    /// No interaction, do not fall back to tags.
    NoAction,
}

impl InteractionType {
    /// `InteractionType.getPrimaryInteractionType`.
    pub fn primary(self) -> Self {
        match self {
            Self::TargetBlock
            | Self::TargetEntity
            | Self::MeleeWeapon
            | Self::Tool
            | Self::CorrectTool
            | Self::IncorrectTool => self,
            Self::UsableTool => Self::Tool,
            Self::ForceRegularCrosshair => self,
            Self::PlaceBlock
            | Self::InteractWithBlock
            | Self::PickUpBlock
            | Self::UseItemOnBlock
            | Self::PlaceItemOnBlock
            | Self::TakeItemFromBlock
            | Self::FillItemFromBlock
            | Self::FillBlockFromItem
            | Self::UseBlock
            | Self::MountBlock
            | Self::PlaceEntity
            | Self::InteractWithEntity
            | Self::PickUpEntity
            | Self::UseItemOnEntity
            | Self::PlaceItemOnEntity
            | Self::TakeItemFromEntity
            | Self::FillItemFromEntity
            | Self::FillEntityFromItem
            | Self::MountEntity
            | Self::UseItem
            | Self::EquipItem
            | Self::ConsumeItem
            | Self::ThrowItem
            | Self::ChargeItem
            | Self::Spyglass
            | Self::PickUpItem
            | Self::RangedWeapon
            | Self::RangedWeaponCharging
            | Self::RangedWeaponCharged
            | Self::Shield
            | Self::Empty
            | Self::NoAction => Self::Empty,
        }
    }

    /// `InteractionType.getSecondaryInteractionType`.
    pub fn secondary(self) -> Self {
        match self {
            Self::PlaceBlock
            | Self::InteractWithBlock
            | Self::PickUpBlock
            | Self::UseItemOnBlock
            | Self::PlaceItemOnBlock
            | Self::TakeItemFromBlock
            | Self::FillItemFromBlock
            | Self::FillBlockFromItem
            | Self::UseBlock
            | Self::MountBlock
            | Self::PlaceEntity
            | Self::InteractWithEntity
            | Self::PickUpEntity
            | Self::UseItemOnEntity
            | Self::PlaceItemOnEntity
            | Self::TakeItemFromEntity
            | Self::FillItemFromEntity
            | Self::FillEntityFromItem
            | Self::MountEntity
            | Self::UseItem
            | Self::EquipItem
            | Self::ConsumeItem
            | Self::ThrowItem
            | Self::ChargeItem
            | Self::Spyglass
            | Self::PickUpItem
            | Self::RangedWeapon
            | Self::RangedWeaponCharged
            | Self::Shield => self,
            Self::UsableTool => Self::UseItemOnBlock,
            Self::RangedWeaponCharging | Self::ForceRegularCrosshair => self,
            Self::TargetBlock
            | Self::TargetEntity
            | Self::MeleeWeapon
            | Self::Tool
            | Self::CorrectTool
            | Self::IncorrectTool
            | Self::Empty
            | Self::NoAction => Self::Empty,
        }
    }
}

/// A pair of primary and secondary interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Crosshair {
    pub primary: InteractionType,
    pub secondary: InteractionType,
}

impl Default for InteractionType {
    fn default() -> Self {
        Self::Empty
    }
}

impl Crosshair {
    pub fn of(interaction: InteractionType) -> Self {
        Self {
            primary: interaction.primary(),
            secondary: interaction.secondary(),
        }
    }

    pub fn pair(primary: InteractionType, secondary: InteractionType) -> Self {
        Self { primary, secondary }
    }

    /// `Crosshair.combine`: the right hand fills what the left left empty, and a
    /// tool is refined by a correct/incorrect tool answer; a charging ranged
    /// weapon wins over another secondary interaction.
    pub fn combine(self, other: Option<Self>) -> Self {
        let Some(other) = other else {
            return self;
        };
        let mut primary = self.primary;
        let mut secondary = self.secondary;
        if self.primary == InteractionType::Empty {
            if other.primary != InteractionType::Empty {
                primary = other.primary;
            }
        } else if self.primary == InteractionType::Tool
            && matches!(
                other.primary,
                InteractionType::CorrectTool | InteractionType::IncorrectTool
            )
        {
            primary = other.primary;
        }
        if self.secondary == InteractionType::Empty {
            if other.secondary != InteractionType::Empty {
                secondary = other.secondary;
            }
        } else if matches!(
            other.secondary,
            InteractionType::RangedWeaponCharging | InteractionType::RangedWeaponCharged
        ) {
            secondary = other.secondary;
        }
        Self { primary, secondary }
    }

    pub fn combine_opt(one: Option<Self>, other: Option<Self>) -> Self {
        match one {
            Some(one) => one.combine(other),
            None => other.unwrap_or_default(),
        }
    }

    pub fn has_interaction(self) -> bool {
        self.primary != InteractionType::Empty || self.secondary != InteractionType::Empty
    }

    pub fn has_secondary(self) -> bool {
        self.secondary != InteractionType::Empty
    }
}
