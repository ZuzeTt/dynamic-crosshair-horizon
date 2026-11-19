//! Dynamic Crosshair for Horizon: hides or changes the crosshair depending
//! on what the player looks at and holds.
//!
//! The original Fabric mod is a large set of mixins over vanilla classes. This
//! port keeps its decision pipeline (`CrosshairHandler`, `CrosshairComponent`,
//! `VanillaApiImpl`) and expresses the mixin rules as registry-id and state
//! checks in `rules`. Horizon's HUD layer hook replaces the vanilla crosshair
//! sprite and draws the configured style parts with the same inverting blend.

mod config;
mod ctx;
mod interaction;
mod render;
mod rules;
mod style;

use std::sync::Mutex;

use mod_api::{
    ClientHost, ClientResourcesReloadedEvent, ClientSetupEvent, ClientTickPostEvent, Flow,
    GuiLayer, GuiLayerDrawEvent, GuiLayerPlacement, GuiLayerPostEvent, GuiLayerPreEvent, Hooks,
    Identifier, RegisterGuiLayersEvent, RegisterRuntimeSpritesEvent, RegistrationContext,
    RenderGuiPreEvent, RuntimeSprite, SettingChangedEvent, TextureAtlas,
};

use config::Config;
use ctx::Ctx;
use interaction::{Crosshair, InteractionType as I};
use style::Component;

const THIRD_PERSON_LAYER: &str = "dynamiccrosshair:third_person";

#[derive(Default)]
struct Computed {
    show: bool,
    component: Component,
    default_style: bool,
    replace_in_post: bool,
}

struct State {
    config: Config,
    config_loaded: bool,
    dirty: bool,
    computed: Computed,
    frame_crosshair_seen: bool,
    attack_ticks: f32,
    attack_delay: f32,
    attack_down: bool,
    target_living: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            config: Config::default(),
            config_loaded: false,
            dirty: true,
            computed: Computed::default(),
            frame_crosshair_seen: false,
            attack_ticks: 0.0,
            attack_delay: 5.0,
            attack_down: false,
            target_living: false,
        }
    }
}

struct Mod {
    state: Mutex<State>,
}

impl Default for Mod {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::default()),
        }
    }
}

fn build_dynamic_by_hand(ctx: &Ctx, config: &Config) -> Crosshair {
    let main = build_advanced_by_hand(ctx, config);
    if main.has_secondary() {
        return main;
    }
    let off = ctx.with_hand(mod_api::Hand::Off);
    let off_crosshair = build_advanced_by_hand(&off, config);
    Crosshair::combine_opt(Some(main), Some(off_crosshair))
}

fn build_advanced_by_hand(ctx: &Ctx, config: &Config) -> Crosshair {
    let mut crosshair = None;
    if ctx.entity().is_some() {
        crosshair = rules::build_from_entity(ctx, config);
    } else if ctx.block().is_some() && ctx.should_interact() && config.on_interactable_block {
        crosshair = rules::build_from_block(ctx, config);
    }
    let item = rules::build_from_item(ctx, config);
    Crosshair::combine_opt(crosshair, item)
}

fn build_crosshair(ctx: &Ctx, config: &Config) -> Crosshair {
    let dynamic = build_dynamic_by_hand(ctx, config);
    let target = match ctx.target {
        Some(mod_api::CrosshairTarget::Entity { .. }) if config.on_entity => {
            Some(Crosshair::of(I::TargetEntity))
        }
        Some(mod_api::CrosshairTarget::Block { state, .. }) => {
            if config.on_block {
                Some(Crosshair::of(I::TargetBlock))
            } else if config.on_interactable_block && rules::is_block_interactable(ctx, config) {
                Some(Crosshair::of(I::TargetBlock))
            } else {
                let _ = state;
                None
            }
        }
        _ => None,
    };
    Crosshair::combine_opt(Some(dynamic), target)
}

fn refresh_config(state: &mut State, host: &dyn ClientHost) {
    state.config = config::read(host, &state.config);
    state.config_loaded = true;
    state.dirty = false;
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, registrar: &mut dyn mod_api::Registrar, context: &RegistrationContext) {
        config::register(registrar, context);
    }

    fn register_gui_layers(
        &self,
        _context: &RegistrationContext,
        event: &mut RegisterGuiLayersEvent<'_>,
    ) {
        event.layers.push(GuiLayer {
            id: THIRD_PERSON_LAYER.into(),
            placement: GuiLayerPlacement::Above("minecraft:crosshair".into()),
        });
    }

    fn register_runtime_sprites(
        &self,
        context: &RegistrationContext,
        event: &mut RegisterRuntimeSpritesEvent<'_>,
    ) {
        let Some(dir) = config::config_dir(context) else {
            return;
        };
        let found = config::scan_custom_styles(&dir);
        config::install_custom_styles(&found);
        for (name, png) in found {
            let Ok(id) = Identifier::from_parts("dynamiccrosshair", &format!("gui/custom/{name}"))
            else {
                continue;
            };
            event.sprites.push(RuntimeSprite {
                atlas: TextureAtlas::Gui,
                id,
                png,
            });
        }
    }

    fn client_setup(&self, _host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        if let Ok(mut state) = self.state.lock() {
            state.dirty = true;
            state.config_loaded = false;
        }
    }

    fn client_resources_reloaded(
        &self,
        _host: &mut dyn ClientHost,
        _event: &mut ClientResourcesReloadedEvent,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state.dirty = true;
        }
    }

    fn setting_changed(&self, _host: &mut dyn ClientHost, _event: &mut SettingChangedEvent<'_>) {
        if let Ok(mut state) = self.state.lock() {
            state.dirty = true;
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let Some(player) = host.local_player() else {
            state.computed = Computed::default();
            return;
        };
        if state.dirty || !state.config_loaded {
            refresh_config(&mut state, &*host);
        }

        let ctx = Ctx::new(host, player);
        let config = state.config.clone();

        // The attack indicator is part of the layer this mod replaces. The
        // client does not expose the hand-strength ticker, so mirror it from
        // the attack mapping and the attack-speed attribute.
        let attack_down = host
            .mapping("key.attack")
            .ok()
            .flatten()
            .is_some_and(|mapping| mapping.down);
        let speed = ctx
            .living()
            .and_then(|living| living.attribute_value(host, "minecraft:attack_speed"))
            .unwrap_or(4.0)
            .max(0.01);
        let delay = (20.0 / speed) as f32;
        if attack_down {
            if !state.attack_down {
                state.attack_ticks = 0.0;
            } else {
                state.attack_ticks += 1.0;
            }
            if state.attack_ticks >= delay {
                state.attack_ticks = 0.0;
            }
        } else {
            state.attack_ticks = delay;
        }
        state.attack_down = attack_down;
        state.attack_delay = delay;
        state.target_living = ctx.entity().is_some_and(|entity| {
            ctx.entity_type_id(entity).is_some_and(|id| {
                !id.contains("item")
                    && !id.contains("arrow")
                    && !id.contains("minecart")
                    && !id.ends_with("_display")
                    && !id.ends_with("_frame")
                    && !id.ends_with("_painting")
                    && !id.ends_with("_orb")
                    && !id.ends_with("_crystal")
                    && !id.ends_with("_knot")
            })
        });

        let hidden = (config.hide_with_screen && host.observed_screen_id().is_some())
            || (config.hide_with_map
                && ctx
                    .main
                    .as_ref()
                    .is_some_and(|stack| ctx.stack_id(stack) == "minecraft:filled_map")
                && ctx.off.is_none());
        if hidden {
            state.computed = Computed {
                show: false,
                ..Computed::default()
            };
            return;
        }

        let mut crosshair = build_crosshair(&ctx, &config);
        let mut show = crosshair.has_interaction();
        if !show && !config.dynamic_crosshair {
            crosshair = Crosshair::of(I::ForceRegularCrosshair);
            show = true;
        }
        let component = if show {
            style::component(crosshair, &config)
        } else {
            Component::default()
        };
        state.computed = Computed {
            show,
            component,
            default_style: !config.dynamic_crosshair_style,
            replace_in_post: false,
        };
    }

    fn gui_layer_pre(&self, _host: &dyn ClientHost, event: &mut GuiLayerPreEvent<'_>) -> Flow {
        if event.layer != "minecraft:crosshair" {
            return Flow::Continue;
        }
        let Ok(mut state) = self.state.lock() else {
            return Flow::Continue;
        };
        state.frame_crosshair_seen = true;
        if !state.computed.show {
            return Flow::Cancel;
        }
        let vanilla_exact = if state.computed.default_style {
            style::default_is_vanilla(&state.config)
        } else {
            state.computed.component.vanilla_exact
        };
        if vanilla_exact {
            state.computed.replace_in_post = false;
            return Flow::Continue;
        }
        state.computed.replace_in_post = true;
        Flow::Cancel
    }

    fn gui_layer_post(&self, host: &dyn ClientHost, event: &mut GuiLayerPostEvent<'_>) {
        if event.layer != "minecraft:crosshair" {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if !state.computed.replace_in_post {
            return;
        }
        state.computed.replace_in_post = false;
        let fix = state.config.fix_centered_crosshair;
        if state.computed.default_style {
            let part = style::default_style(&state.config);
            render::draw(event.draw, &[part], fix);
        } else {
            let component = &state.computed.component;
            let mut parts = Vec::with_capacity(component.blend.len() + component.no_blend.len());
            parts.extend_from_slice(&component.blend);
            parts.extend_from_slice(&component.no_blend);
            render::draw(event.draw, &parts, fix);
        }
        draw_attack_indicator_if_needed(host, event.draw, &state);
    }

    fn gui_layer_draw(&self, host: &dyn ClientHost, event: &mut GuiLayerDrawEvent<'_>) {
        if event.layer != THIRD_PERSON_LAYER {
            return;
        }
        let Ok(state) = self.state.lock() else {
            return;
        };
        if !state.config.third_person_crosshair
            || host.camera_type() != mod_api::CameraType::ThirdPersonBack
            || state.frame_crosshair_seen
            || !state.computed.show
        {
            return;
        }
        let fix = state.config.fix_centered_crosshair;
        if state.computed.default_style {
            let part = style::default_style(&state.config);
            render::draw(event.draw, &[part], fix);
        } else {
            let component = &state.computed.component;
            let mut parts = Vec::with_capacity(component.blend.len() + component.no_blend.len());
            parts.extend_from_slice(&component.blend);
            parts.extend_from_slice(&component.no_blend);
            render::draw(event.draw, &parts, fix);
        }
    }

    fn render_gui_pre(&self, _host: &dyn ClientHost, _event: &mut RenderGuiPreEvent) -> Flow {
        if let Ok(mut state) = self.state.lock() {
            state.frame_crosshair_seen = false;
            state.computed.replace_in_post = false;
        }
        Flow::Continue
    }
}

fn draw_attack_indicator_if_needed(
    host: &dyn ClientHost,
    draw: &mut dyn mod_api::DrawContext,
    state: &State,
) {
    if host.gui_options().attack_indicator != 1 || state.attack_delay <= 0.0 {
        return;
    }
    let scale = (state.attack_ticks / state.attack_delay).clamp(0.0, 1.0);
    let full = scale >= 1.0 && state.attack_delay > 5.0 && state.target_living;
    if full || scale < 1.0 {
        render::draw_attack_indicator(draw, scale, state.attack_delay, full);
    }
}
