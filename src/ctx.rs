//! One evaluation's view of the game: the local player, both hands, and the
//! block or entity under the crosshair.

use mod_api::{
    BlockKey, BlockMetadata, BlockState, ClientHost, CrosshairTarget, EntityRef, EntityTypeKey,
    FluidKey, Hand, HostBlockStates, ItemKey, ItemStack, LivingRef, PlayerRef,
};

/// What the crosshair's ray meets, in a form the rules can use.
#[derive(Clone, Copy, Debug)]
pub struct BlockHit {
    pub pos: [i32; 3],
    pub state: BlockState,
    pub face: [i32; 3],
}

pub struct Ctx<'a> {
    pub host: &'a dyn ClientHost,
    pub player: PlayerRef,
    pub target: Option<CrosshairTarget>,
    pub main: Option<ItemStack>,
    pub off: Option<ItemStack>,
    pub hand: Hand,
    living: Option<LivingRef>,
    level: Option<mod_api::LevelRef>,
}

impl<'a> Ctx<'a> {
    pub fn new(host: &'a dyn ClientHost, player: PlayerRef) -> Self {
        let living = host.living(player.entity());
        let level = player.level(host);
        let main = living.and_then(|living| living.hand_item(host, Hand::Main));
        let off = living.and_then(|living| living.hand_item(host, Hand::Off));
        Self {
            host,
            player,
            target: host.crosshair_target(),
            main,
            off,
            hand: Hand::Main,
            living,
            level,
        }
    }

    pub fn with_hand(&self, hand: Hand) -> Self {
        Self {
            hand,
            ..self.clone()
        }
    }

    pub fn hand_stack(&self) -> Option<&ItemStack> {
        match self.hand {
            Hand::Main => self.main.as_ref(),
            Hand::Off => self.off.as_ref(),
        }
    }

    pub fn is_main_hand(&self) -> bool {
        self.hand == Hand::Main
    }

    pub fn is_off_hand(&self) -> bool {
        self.hand == Hand::Off
    }

    pub fn block(&self) -> Option<BlockHit> {
        match self.target? {
            CrosshairTarget::Block { pos, state, hit } => Some(BlockHit {
                pos: [pos.x, pos.y, pos.z],
                state,
                face: hit.face,
            }),
            CrosshairTarget::Entity { .. } => None,
            _ => None,
        }
    }

    pub fn entity(&self) -> Option<EntityRef> {
        match self.target? {
            CrosshairTarget::Entity { entity, .. } => Some(entity),
            CrosshairTarget::Block { .. } => None,
            _ => None,
        }
    }

    pub fn is_targeting(&self) -> bool {
        self.target.is_some()
    }

    pub fn living(&self) -> Option<LivingRef> {
        self.living
    }

    pub fn level(&self) -> Option<mod_api::LevelRef> {
        self.level
    }

    pub fn player_entity(&self) -> EntityRef {
        self.player.entity()
    }

    pub fn is_sneaking(&self) -> bool {
        self.player
            .is_secondary_use_active(self.host)
            .unwrap_or(false)
    }

    /// `CrosshairContext.shouldInteract`: not crouching, or both hands empty.
    pub fn should_interact(&self) -> bool {
        !self.is_sneaking() || (self.main.is_none() && self.off.is_none())
    }

    pub fn is_creative(&self) -> bool {
        self.player.is_creative(self.host).unwrap_or(false)
    }

    pub fn is_spectator(&self) -> bool {
        self.player.is_spectator(self.host).unwrap_or(false)
    }

    pub fn is_cooling_down(&self) -> bool {
        let Some(stack) = self.hand_stack() else {
            return false;
        };
        let group = self.host.item_cooldown_group(stack);
        self.player
            .has_item_cooldown(self.host, &group)
            .unwrap_or(false)
    }

    pub fn can_eat(&self, always: bool) -> bool {
        self.player.can_eat(self.host, always).unwrap_or(false)
    }

    pub fn is_gliding(&self) -> bool {
        self.living
            .and_then(|living| living.is_gliding(self.host))
            .unwrap_or(false)
    }

    pub fn on_ground(&self) -> bool {
        self.player_entity().on_ground(self.host).unwrap_or(false)
    }

    pub fn block_id(&self, state: BlockState) -> String {
        state.block().id(self.host)
    }

    pub fn block_is(&self, state: BlockState, id: &str) -> bool {
        self.block_id(state) == id
    }

    pub fn block_property(&self, state: BlockState, name: &str) -> Option<String> {
        self.host
            .block_states()
            .property((&state).into(), name)
            .ok()
            .flatten()
            .map(str::to_owned)
    }

    pub fn block_int_property(&self, state: BlockState, name: &str) -> Option<i32> {
        self.block_property(state, name)?.parse().ok()
    }

    pub fn block_bool_property(&self, state: BlockState, name: &str) -> bool {
        self.block_property(state, name).as_deref() == Some("true")
    }

    pub fn block_block(&self, state: BlockState) -> BlockKey {
        state.block()
    }

    pub fn block_in_tag(&self, state: BlockState, tag: &str) -> bool {
        self.host.block_in_tag(state.block(), tag).unwrap_or(false)
    }

    pub fn item_id(&self, item: ItemKey) -> String {
        item.id(self.host)
    }

    pub fn item_is(&self, item: ItemKey, id: &str) -> bool {
        self.item_id(item) == id
    }

    pub fn stack_id(&self, stack: &ItemStack) -> String {
        self.item_id(stack.item())
    }

    pub fn stack_is(&self, stack: &ItemStack, id: &str) -> bool {
        self.stack_id(stack) == id
    }

    pub fn item_in_tag(&self, item: ItemKey, tag: &str) -> bool {
        let id = item.id(self.host);
        let Some(entry) = self.host.registry_entry("minecraft:item", &id) else {
            return false;
        };
        self.host
            .registry_entry_in_tag(&entry, tag)
            .unwrap_or(false)
    }

    pub fn stack_in_tag(&self, stack: &ItemStack, tag: &str) -> bool {
        self.item_in_tag(stack.item(), tag)
    }

    pub fn item_has_component(&self, item: ItemKey, component: &str) -> bool {
        self.host.item_default_component(item, component).is_some()
    }

    pub fn stack_has_component(&self, stack: &ItemStack, component: &str) -> bool {
        stack.components().is_some_and(|patch| {
            patch
                .as_nbt()
                .keys()
                .any(|key| key.to_string_lossy() == component)
        }) || self.item_has_component(stack.item(), component)
    }

    pub fn stack_component(&self, stack: &ItemStack, component: &str) -> Option<mod_api::Tag> {
        stack
            .components()
            .and_then(|patch| {
                patch
                    .as_nbt()
                    .iter()
                    .find(|(key, _)| key.to_string_lossy() == component)
                    .map(|(_, value)| value.clone())
            })
            .or_else(|| self.host.item_default_component(stack.item(), component))
    }

    pub fn stack_component_is_empty_list(&self, stack: &ItemStack, component: &str) -> bool {
        match self.stack_component(stack, component) {
            None => true,
            Some(mod_api::Tag::List(list)) => list.is_empty(),
            Some(mod_api::Tag::Compound(map)) => map.is_empty(),
            Some(_) => false,
        }
    }

    pub fn entity_type_id(&self, entity: EntityRef) -> Option<String> {
        entity.kind(self.host).map(|key| key.id(self.host))
    }

    pub fn entity_is(&self, entity: EntityRef, id: &str) -> bool {
        self.entity_type_id(entity).as_deref() == Some(id)
    }

    pub fn entity_type_in_tag(&self, entity: EntityRef, tag: &str) -> bool {
        self.entity_type_id(entity)
            .is_some_and(|id| self.host.entity_type_in_tag(&id, tag))
    }

    pub fn entity_type_key(&self, entity: EntityRef) -> Option<EntityTypeKey> {
        entity.kind(self.host)
    }

    pub fn fluid_state_at(&self, pos: [i32; 3]) -> Option<mod_api::FluidState> {
        if let Some(level) = self.level {
            if let Some(state) = level.fluid_state(self.host, pos) {
                if !state.is_empty() {
                    return Some(state);
                }
            }
        }
        let state = self.block_state_at(pos)?;
        self.host.state_fluid_state(state)
    }

    pub fn fluid_is(&self, fluid: FluidKey, id: &str) -> bool {
        fluid.id(self.host) == id
    }

    pub fn fluid_in_tag(&self, fluid: FluidKey, tag: &str) -> bool {
        let id = fluid.id(self.host);
        self.host.fluid_in_tag(&id, tag).unwrap_or(false)
    }

    pub fn block_destroy_speed(&self, state: BlockState) -> Option<f32> {
        self.host.block_states().destroy_speed(state)
    }

    pub fn block_state_at(&self, pos: [i32; 3]) -> Option<BlockState> {
        self.level?.block_state(self.host, pos)
    }

    /// How fast the held tool mines `state`, before player modifiers.
    pub fn break_speed(&self, _state: BlockState, stack: Option<&ItemStack>) -> Option<f32> {
        let pos = self.block()?.pos;
        self.level?.block_break_speed(self.host, pos, stack)
    }

    pub fn entity_eye_position(&self, entity: EntityRef) -> Option<[f64; 3]> {
        entity.eye_position(self.host)
    }

    pub fn entity_rotation(&self, entity: EntityRef) -> Option<[f32; 2]> {
        entity.rotation(self.host)
    }

    pub fn entity_bounding_box(&self, entity: EntityRef) -> Option<[[f64; 3]; 2]> {
        entity.bounding_box(self.host)
    }

    pub fn entity_passengers(&self, entity: EntityRef) -> usize {
        entity
            .passengers(self.host)
            .map(|list| list.len())
            .unwrap_or(0)
    }

    pub fn entity_health(&self, entity: EntityRef) -> Option<(f32, f32)> {
        let living = self.host.living(entity)?;
        Some((living.health(self.host)?, living.max_health(self.host)?))
    }

    pub fn entity_is_baby(&self, entity: EntityRef) -> bool {
        self.host
            .living(entity)
            .and_then(|living| living.baby(self.host))
            .unwrap_or(false)
    }

    pub fn entity_is_tamed(&self, entity: EntityRef) -> bool {
        self.host
            .living(entity)
            .and_then(|living| living.is_tamed(self.host))
            .unwrap_or(false)
    }

    pub fn entity_equipment(
        &self,
        entity: EntityRef,
        slot: mod_api::EquipmentSlot,
    ) -> Option<ItemStack> {
        let living = self.host.living(entity)?;
        living.equipment(self.host, slot)
    }

    pub fn player_equipment(&self, slot: mod_api::EquipmentSlot) -> Option<ItemStack> {
        self.living
            .and_then(|living| living.equipment(self.host, slot))
    }

    pub fn entity_leash_holder(&self, entity: EntityRef) -> Option<mod_api::LeashHolder> {
        entity.leash_holder(self.host)
    }

    /// A ray from the player's eyes along the current view, checking fluid
    /// sources before the first block. `source_only` mirrors vanilla's
    /// `FluidHandling.SOURCE_ONLY`.
    pub fn raycast_fluid(&self, source_only: bool, distance: f64) -> Option<BlockHit> {
        let level = self.level?;
        let eye = self.entity_eye_position(self.player.entity())?;
        let [yaw, pitch] = self.entity_rotation(self.player.entity())?;
        let yaw = f64::from(yaw).to_radians();
        let pitch = f64::from(pitch).to_radians();
        let dir = [
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ];
        let end = [
            eye[0] + dir[0] * distance,
            eye[1] + dir[1] * distance,
            eye[2] + dir[2] * distance,
        ];
        let block_hit = level.clip(
            self.host,
            eye,
            end,
            mod_api::ClipBlock::Collider,
            mod_api::ClipFluid::None,
        );
        let block_distance = block_hit
            .filter(|hit| !hit.miss)
            .map(|hit| {
                let dx = f64::from(hit.block[0]) + 0.5 - eye[0];
                let dy = f64::from(hit.block[1]) + 0.5 - eye[1];
                let dz = f64::from(hit.block[2]) + 0.5 - eye[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .unwrap_or(distance);
        let limit = block_distance.min(distance);
        let steps = (limit / 0.05).ceil() as i32;
        let mut last = [i32::MIN; 3];
        for index in 0..=steps {
            let t = limit * f64::from(index) / f64::from(steps.max(1));
            let pos = [
                (eye[0] + dir[0] * t).floor() as i32,
                (eye[1] + dir[1] * t).floor() as i32,
                (eye[2] + dir[2] * t).floor() as i32,
            ];
            if pos == last {
                continue;
            }
            last = pos;
            let Some(state) = self.fluid_state_at(pos) else {
                continue;
            };
            if state.is_empty() || (source_only && !state.is_source()) {
                continue;
            }
            let Some(state_block) = self.block_state_at(pos) else {
                continue;
            };
            return Some(BlockHit {
                pos,
                state: state_block,
                face: [0, 1, 0],
            });
        }
        block_hit.and_then(|hit| {
            if hit.miss {
                return None;
            }
            let face = match hit.face {
                mod_api::BlockFace::Down => [0, -1, 0],
                mod_api::BlockFace::Up => [0, 1, 0],
                mod_api::BlockFace::North => [0, 0, -1],
                mod_api::BlockFace::South => [0, 0, 1],
                mod_api::BlockFace::West => [-1, 0, 0],
                mod_api::BlockFace::East => [1, 0, 0],
                _ => [0, 1, 0],
            };
            Some(BlockHit {
                pos: hit.block,
                state: self.block_state_at(hit.block)?,
                face,
            })
        })
    }
}

impl Clone for Ctx<'_> {
    fn clone(&self) -> Self {
        Self {
            host: self.host,
            player: self.player,
            target: self.target,
            main: self.main.clone(),
            off: self.off.clone(),
            hand: self.hand,
            living: self.living,
            level: self.level,
        }
    }
}
