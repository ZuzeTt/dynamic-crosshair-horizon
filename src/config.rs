//! Settings, style selection and the small JSON file for the original mod's
//! free-form tweak lists.
//!
//! Everything the original stores in `dynamiccrosshair.json5` is exposed
//! through Horizon's Options > Mods page. Lists of extra items and blocks
//! cannot be expressed as settings widgets, so they live in
//! `config/dynamiccrosshair.json` (plain JSON):
//!
//! ```json
//! {
//!   "additional_tools": ["mymod:hammer", "#mymod:tools"],
//!   "additional_melee_weapons": [],
//!   "additional_ranged_weapons": [],
//!   "additional_throwables": [],
//!   "additional_usable_items": [],
//!   "additional_interactable_blocks": ["mymod:machine"]
//! }
//! ```

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use mod_api::{ClientHost, RegistrationContext, SettingSpec, serde_json};

/// Horizon's own settings page shows one row per registered setting.
pub const SETTING_PREFIX: &str = "dynamiccrosshair.option.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Always,
    IfTargeting,
    Disabled,
}

impl Policy {
    fn from_choice(index: u32) -> Self {
        match index {
            0 => Self::Always,
            1 => Self::IfTargeting,
            _ => Self::Disabled,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsablePolicy {
    Always,
    IfInteractable,
    Disabled,
}

impl UsablePolicy {
    fn from_choice(index: u32) -> Self {
        match index {
            0 => Self::Always,
            1 => Self::IfInteractable,
            _ => Self::Disabled,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPolicy {
    Always,
    IfTargeting,
    IfInteractable,
    Disabled,
}

impl BlockPolicy {
    fn from_choice(index: u32) -> Self {
        match index {
            0 => Self::Always,
            1 => Self::IfTargeting,
            2 => Self::IfInteractable,
            _ => Self::Disabled,
        }
    }
}

/// One crosshair part configuration: which sprite, what colour, and whether
/// it blends by inverting what is under it.
#[derive(Clone, Debug)]
pub struct StyleConfig {
    pub index: usize,
    pub override_color: bool,
    pub color: u32,
    pub blend: bool,
    pub coalesce: bool,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            index: 0,
            override_color: false,
            color: 0xFF_FFFF,
            blend: true,
            coalesce: true,
        }
    }
}

/// The style slots, in the original mod's `CrosshairStyles` order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleSlot {
    Regular = 0,
    OnBlock = 1,
    OnEntity = 2,
    HoldingTool = 3,
    HoldingMeleeWeapon = 4,
    HoldingRangedWeapon = 5,
    HoldingThrowable = 6,
    HoldingBlock = 7,
    Interact = 8,
    UseItem = 9,
    Shield = 10,
    ModCorrectTool = 11,
    ModIncorrectTool = 12,
}

pub const STYLE_SLOTS: [StyleSlot; 13] = [
    StyleSlot::Regular,
    StyleSlot::OnBlock,
    StyleSlot::OnEntity,
    StyleSlot::HoldingTool,
    StyleSlot::HoldingMeleeWeapon,
    StyleSlot::HoldingRangedWeapon,
    StyleSlot::HoldingThrowable,
    StyleSlot::HoldingBlock,
    StyleSlot::Interact,
    StyleSlot::UseItem,
    StyleSlot::Shield,
    StyleSlot::ModCorrectTool,
    StyleSlot::ModIncorrectTool,
];

impl StyleSlot {
    pub fn key(self) -> &'static str {
        match self {
            Self::Regular => "style_regular",
            Self::OnBlock => "style_on_block",
            Self::OnEntity => "style_on_entity",
            Self::HoldingTool => "style_holding_tool",
            Self::HoldingMeleeWeapon => "style_holding_melee_weapon",
            Self::HoldingRangedWeapon => "style_holding_ranged_weapon",
            Self::HoldingThrowable => "style_holding_throwable",
            Self::HoldingBlock => "style_holding_block",
            Self::Interact => "style_interact",
            Self::UseItem => "style_use_item",
            Self::Shield => "style_shield",
            Self::ModCorrectTool => "style_mod_correct_tool",
            Self::ModIncorrectTool => "style_mod_incorrect_tool",
        }
    }

    /// The sprite each slot uses out of the box, by builtin style path.
    fn default_style(self) -> &'static str {
        match self {
            Self::Regular => "crosshair",
            Self::OnBlock => "cross-open",
            Self::OnEntity => "cross-open-diagonal",
            Self::HoldingTool => "square",
            Self::HoldingMeleeWeapon => "cross-open",
            Self::HoldingRangedWeapon => "cross-open-diagonal",
            Self::HoldingThrowable => "circle-large",
            Self::HoldingBlock => "diamond",
            Self::Interact => "brackets",
            Self::UseItem => "brackets-round",
            Self::Shield => "brackets-bottom",
            Self::ModCorrectTool => "dot",
            Self::ModIncorrectTool => "cross-diagonal-small",
        }
    }

    /// Default colour override (`modCorrectTool` is green and
    /// `modIncorrectTool` red in the original).
    fn default_color(self) -> Option<u32> {
        match self {
            Self::ModCorrectTool => Some(0x00_FF00),
            Self::ModIncorrectTool => Some(0xFF_0000),
            _ => None,
        }
    }

    fn default_blend(self) -> bool {
        !matches!(self, Self::ModCorrectTool | Self::ModIncorrectTool)
    }
}

/// A style the player can pick: a builtin sprite, the vanilla sprite, or a
/// PNG from `config/dynamiccrosshair/crosshairs/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleEntry {
    /// The GUI sprite id handed to `DrawContext::sprite` / `GuiDraw::Sprite`.
    pub sprite: String,
    /// A translation key for the settings list (the raw name for custom PNGs).
    pub label: String,
    /// The file name stem for a custom style, if this is one.
    pub custom_name: Option<String>,
}

fn builtin_styles() -> Vec<StyleEntry> {
    let names = [
        "crosshair",
        "cross-open",
        "cross-open-diagonal",
        "circle",
        "circle-large",
        "square",
        "square-large",
        "diamond",
        "diamond-large",
        "caret",
        "dot",
        "cross-diagonal-small",
        "brackets",
        "brackets-bottom",
        "brackets-top",
        "brackets-round",
        "lines",
        "line-bottom",
    ];
    let mut out = Vec::with_capacity(names.len() + 8);
    out.push(StyleEntry {
        sprite: "minecraft:hud/crosshair".into(),
        label: format!("{SETTING_PREFIX}style.vanilla"),
        custom_name: None,
    });
    for name in names {
        out.push(StyleEntry {
            sprite: format!("dynamiccrosshair:gui/builtin/{name}"),
            label: format!("{SETTING_PREFIX}style.{name}"),
            custom_name: None,
        });
    }
    out
}

static STYLES: OnceLock<Mutex<Vec<StyleEntry>>> = OnceLock::new();

pub fn styles() -> &'static Mutex<Vec<StyleEntry>> {
    STYLES.get_or_init(|| Mutex::new(builtin_styles()))
}

/// Where the game keeps this mod's config files: `<instance>/config/dynamiccrosshair`.
pub fn config_dir(context: &RegistrationContext) -> Option<PathBuf> {
    let dir = context.instance_dir().ok()?;
    let base = match dir {
        mod_api::InstanceDirectory::Unix(bytes) => {
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStringExt;
                PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec()))
            }
            #[cfg(not(unix))]
            {
                PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
            }
        }
        mod_api::InstanceDirectory::Windows(units) => {
            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStringExt;
                PathBuf::from(std::ffi::OsString::from_wide(units))
            }
            #[cfg(not(windows))]
            {
                PathBuf::from(String::from_utf16_lossy(units))
            }
        }
        _ => return None,
    };
    Some(base.join("config").join("dynamiccrosshair"))
}

/// PNGs under `config/dynamiccrosshair/crosshairs/`, sorted by file name.
pub fn scan_custom_styles(dir: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let folder = dir.join("crosshairs");
    let Ok(read) = std::fs::read_dir(&folder) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // The name becomes part of an identifier and a translation key.
        let name: String = stem
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
                    c
                } else if c.is_ascii_uppercase() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();
        if name.is_empty() {
            continue;
        }
        if let Ok(png) = std::fs::read(&path) {
            found.push((name, png));
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found.dedup_by(|a, b| a.0 == b.0);
    found
}

/// Register a freshly discovered set of custom PNG styles in the style table.
pub fn install_custom_styles(found: &[(String, Vec<u8>)]) {
    let mut styles = styles().lock().unwrap();
    for (name, _) in found {
        if styles
            .iter()
            .any(|entry| entry.custom_name.as_deref() == Some(name))
        {
            continue;
        }
        styles.push(StyleEntry {
            sprite: format!("dynamiccrosshair:gui/custom/{name}"),
            label: name.clone(),
            custom_name: Some(name.clone()),
        });
    }
}

/// The lists the original keeps in its config screen as item/block pickers.
#[derive(Clone, Debug, Default)]
pub struct ExtraLists {
    pub tools: Vec<String>,
    pub melee_weapons: Vec<String>,
    pub ranged_weapons: Vec<String>,
    pub throwables: Vec<String>,
    pub usable_items: Vec<String>,
    pub interactable_blocks: Vec<String>,
}

static EXTRA: OnceLock<Mutex<ExtraLists>> = OnceLock::new();

pub fn extra_lists() -> &'static Mutex<ExtraLists> {
    EXTRA.get_or_init(|| Mutex::new(ExtraLists::default()))
}

fn strings(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Read `config/dynamiccrosshair.json`, ignoring a missing or malformed file.
pub fn read_extra_lists(dir: &std::path::Path) -> ExtraLists {
    // `dir` is `config/dynamiccrosshair`; the file sits beside it as
    // `config/dynamiccrosshair.json`, like the original's config store.
    let parent = dir.parent().unwrap_or(dir);
    let Ok(text) = std::fs::read_to_string(parent.join("dynamiccrosshair.json")) else {
        return ExtraLists::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return ExtraLists::default();
    };
    ExtraLists {
        tools: strings(value.get("additional_tools")),
        melee_weapons: strings(value.get("additional_melee_weapons")),
        ranged_weapons: strings(value.get("additional_ranged_weapons")),
        throwables: strings(value.get("additional_throwables")),
        usable_items: strings(value.get("additional_usable_items")),
        interactable_blocks: strings(value.get("additional_interactable_blocks")),
    }
}

/// The whole configuration, as read from Horizon's settings at runtime.
#[derive(Clone, Debug)]
pub struct Config {
    pub dynamic_crosshair: bool,
    pub third_person_crosshair: bool,
    pub hide_with_screen: bool,
    pub hide_with_map: bool,
    pub fix_centered_crosshair: bool,
    pub dynamic_crosshair_style: bool,
    #[allow(dead_code)]
    pub disable_debug_crosshair: bool,

    pub on_block: bool,
    pub on_interactable_block: bool,
    pub on_entity: bool,
    pub holding_tool: Policy,
    pub display_correct_tool: bool,
    pub holding_melee_weapon: bool,
    pub melee_weapon_on_entity: bool,
    pub melee_weapon_on_breakable_block: bool,
    pub holding_ranged_weapon: UsablePolicy,
    pub holding_throwable: UsablePolicy,
    pub holding_shield: bool,
    pub holding_block: BlockPolicy,
    pub holding_block_in_offhand: bool,
    pub holding_usable_item: UsablePolicy,
    pub force_holding_spyglass: bool,
    pub enable_tweaks: bool,

    pub global_override_color: bool,
    pub global_color: u32,
    pub global_blend: bool,

    pub style: [StyleConfig; 13],
    pub extra: ExtraLists,
}

impl Default for Config {
    fn default() -> Self {
        let mut style = std::array::from_fn(|_| StyleConfig::default());
        // The defaults from the original `Config` constructors.
        let defaults: [(StyleSlot, bool, bool); 13] = [
            (StyleSlot::Regular, true, true),
            (StyleSlot::OnBlock, true, true),
            (StyleSlot::OnEntity, true, true),
            (StyleSlot::HoldingTool, true, true),
            (StyleSlot::HoldingMeleeWeapon, true, true),
            (StyleSlot::HoldingRangedWeapon, true, false),
            (StyleSlot::HoldingThrowable, true, true),
            (StyleSlot::HoldingBlock, true, false),
            (StyleSlot::Interact, true, true),
            (StyleSlot::UseItem, true, true),
            (StyleSlot::Shield, true, true),
            (StyleSlot::ModCorrectTool, false, true),
            (StyleSlot::ModIncorrectTool, false, true),
        ];
        for (slot, blend, coalesce) in defaults {
            let entry = &mut style[slot as usize];
            entry.blend = blend;
            entry.coalesce = coalesce;
            if let Some(color) = slot.default_color() {
                entry.override_color = true;
                entry.color = color;
            }
        }
        Self {
            dynamic_crosshair: true,
            third_person_crosshair: false,
            hide_with_screen: true,
            hide_with_map: true,
            fix_centered_crosshair: false,
            dynamic_crosshair_style: true,
            disable_debug_crosshair: false,
            on_block: true,
            on_interactable_block: true,
            on_entity: true,
            holding_tool: Policy::Always,
            display_correct_tool: true,
            holding_melee_weapon: true,
            melee_weapon_on_entity: false,
            melee_weapon_on_breakable_block: false,
            holding_ranged_weapon: UsablePolicy::Always,
            holding_throwable: UsablePolicy::IfInteractable,
            holding_shield: true,
            holding_block: BlockPolicy::IfInteractable,
            holding_block_in_offhand: true,
            holding_usable_item: UsablePolicy::IfInteractable,
            force_holding_spyglass: false,
            enable_tweaks: true,
            global_override_color: false,
            global_color: 0xFF_FFFF,
            global_blend: true,
            style,
            extra: ExtraLists::default(),
        }
    }
}

impl Config {
    /// The style entry index for a slot's default.
    pub fn default_style_index(slot: StyleSlot) -> usize {
        let want = slot.default_style();
        styles()
            .lock()
            .unwrap()
            .iter()
            .position(|entry| {
                entry
                    .sprite
                    .strip_prefix("dynamiccrosshair:gui/builtin/")
                    .is_some_and(|path| path == want)
                    || (want == "crosshair" && entry.sprite == "minecraft:hud/crosshair")
            })
            .unwrap_or(0)
    }

    pub fn style_for(&self, slot: StyleSlot) -> &StyleConfig {
        &self.style[slot as usize]
    }

    pub fn resolved_style(&self, slot: StyleSlot) -> (String, u32, bool, bool) {
        let config = self.style_for(slot);
        let styles = styles().lock().unwrap();
        let sprite = styles
            .get(config.index)
            .map(|entry| entry.sprite.clone())
            .unwrap_or_else(|| "minecraft:hud/crosshair".into());
        let color = if config.override_color {
            config.color
        } else if self.global_override_color {
            self.global_color
        } else {
            0xFF_FFFF
        };
        (sprite, color, config.blend, config.coalesce)
    }
}

fn choice(host: &dyn ClientHost, key: &str, default: u32) -> u32 {
    host.setting_choice(key).unwrap_or(default)
}

fn toggle(host: &dyn ClientHost, key: &str, default: bool) -> bool {
    host.setting_bool(key).unwrap_or(default)
}

/// Read every setting back. Called after settings load and on every change.
pub fn read(host: &dyn ClientHost, _previous: &Config) -> Config {
    let mut config = Config {
        dynamic_crosshair: toggle(host, "dynamic_crosshair", true),
        third_person_crosshair: toggle(host, "third_person_crosshair", false),
        hide_with_screen: toggle(host, "hide_with_screen", true),
        hide_with_map: toggle(host, "hide_with_map", true),
        fix_centered_crosshair: toggle(host, "fix_centered_crosshair", false),
        dynamic_crosshair_style: toggle(host, "dynamic_crosshair_style", true),
        disable_debug_crosshair: toggle(host, "disable_debug_crosshair", false),
        on_block: toggle(host, "on_block", true),
        on_interactable_block: toggle(host, "on_interactable_block", true),
        on_entity: toggle(host, "on_entity", true),
        holding_tool: Policy::from_choice(choice(host, "holding_tool", 0)),
        display_correct_tool: toggle(host, "display_correct_tool", true),
        holding_melee_weapon: toggle(host, "holding_melee_weapon", true),
        melee_weapon_on_entity: toggle(host, "melee_weapon_on_entity", false),
        melee_weapon_on_breakable_block: toggle(host, "melee_weapon_on_breakable_block", false),
        holding_ranged_weapon: UsablePolicy::from_choice(choice(host, "holding_ranged_weapon", 0)),
        holding_throwable: UsablePolicy::from_choice(choice(host, "holding_throwable", 1)),
        holding_shield: toggle(host, "holding_shield", true),
        holding_block: BlockPolicy::from_choice(choice(host, "holding_block", 2)),
        holding_block_in_offhand: toggle(host, "holding_block_in_offhand", true),
        holding_usable_item: UsablePolicy::from_choice(choice(host, "holding_usable_item", 1)),
        force_holding_spyglass: toggle(host, "force_holding_spyglass", false),
        enable_tweaks: toggle(host, "enable_tweaks", true),
        global_override_color: toggle(host, "color_override", false),
        global_color: host.setting_color("color").unwrap_or(0xFF_FFFF),
        global_blend: toggle(host, "color_blend", true),
        style: std::array::from_fn(|_| StyleConfig::default()),
        extra: extra_lists().lock().unwrap().clone(),
    };
    for slot in STYLE_SLOTS {
        let key = slot.key();
        let default_index = Config::default_style_index(slot) as u32;
        let index = choice(host, key, default_index) as usize;
        let custom = slot.default_color().is_some();
        let entry = &mut config.style[slot as usize];
        entry.index = index;
        entry.override_color = toggle(host, &format!("{key}_override_color"), custom);
        entry.color = host
            .setting_color(&format!("{key}_color"))
            .unwrap_or_else(|| slot.default_color().unwrap_or(0xFF_FFFF));
        entry.blend = toggle(host, &format!("{key}_blend"), slot.default_blend());
        entry.coalesce = toggle(
            host,
            &format!("{key}_coalesce"),
            !matches!(
                slot,
                StyleSlot::HoldingRangedWeapon | StyleSlot::HoldingBlock
            ),
        );
    }
    config
}

/// Register every setting and remember the custom style list for choices.
pub fn register(registrar: &mut dyn mod_api::Registrar, context: &RegistrationContext) {
    let dir = config_dir(context);
    let found = dir.as_deref().map(scan_custom_styles).unwrap_or_default();
    install_custom_styles(&found);
    if let Some(dir) = dir.as_deref() {
        *extra_lists().lock().unwrap() = read_extra_lists(dir);
    }
    let style_labels: Vec<String> = styles()
        .lock()
        .unwrap()
        .iter()
        .map(|entry| entry.label.clone())
        .collect();

    let mut add = |setting: SettingSpec| {
        let _ = registrar.register_setting(setting);
    };

    add(SettingSpec::toggle(
        "dynamic_crosshair",
        format!("{SETTING_PREFIX}dynamic_crosshair"),
        true,
    ));
    add(SettingSpec::toggle(
        "third_person_crosshair",
        format!("{SETTING_PREFIX}third_person_crosshair"),
        false,
    ));
    add(SettingSpec::toggle(
        "hide_with_screen",
        format!("{SETTING_PREFIX}hide_with_screen"),
        true,
    ));
    add(SettingSpec::toggle(
        "hide_with_map",
        format!("{SETTING_PREFIX}hide_with_map"),
        true,
    ));
    add(SettingSpec::toggle(
        "fix_centered_crosshair",
        format!("{SETTING_PREFIX}fix_centered_crosshair"),
        false,
    ));
    add(SettingSpec::toggle(
        "dynamic_crosshair_style",
        format!("{SETTING_PREFIX}dynamic_crosshair_style"),
        true,
    ));
    add(SettingSpec::toggle(
        "disable_debug_crosshair",
        format!("{SETTING_PREFIX}disable_debug_crosshair"),
        false,
    ));

    add(SettingSpec::toggle(
        "on_block",
        format!("{SETTING_PREFIX}on_block"),
        true,
    ));
    add(SettingSpec::toggle(
        "on_interactable_block",
        format!("{SETTING_PREFIX}on_interactable_block"),
        true,
    ));
    add(SettingSpec::toggle(
        "on_entity",
        format!("{SETTING_PREFIX}on_entity"),
        true,
    ));

    let policy_options = [
        format!("{SETTING_PREFIX}policy.always"),
        format!("{SETTING_PREFIX}policy.if_targeting"),
        format!("{SETTING_PREFIX}policy.disabled"),
    ];
    let usable_options = [
        format!("{SETTING_PREFIX}policy.always"),
        format!("{SETTING_PREFIX}policy.if_interactable"),
        format!("{SETTING_PREFIX}policy.disabled"),
    ];
    let block_options = [
        format!("{SETTING_PREFIX}policy.always"),
        format!("{SETTING_PREFIX}policy.if_targeting"),
        format!("{SETTING_PREFIX}policy.if_interactable"),
        format!("{SETTING_PREFIX}policy.disabled"),
    ];
    add(SettingSpec::choice(
        "holding_tool",
        format!("{SETTING_PREFIX}holding_tool"),
        policy_options.clone(),
        0,
    ));
    add(SettingSpec::toggle(
        "display_correct_tool",
        format!("{SETTING_PREFIX}display_correct_tool"),
        true,
    ));
    add(SettingSpec::toggle(
        "holding_melee_weapon",
        format!("{SETTING_PREFIX}holding_melee_weapon"),
        true,
    ));
    add(SettingSpec::toggle(
        "melee_weapon_on_entity",
        format!("{SETTING_PREFIX}melee_weapon_on_entity"),
        false,
    ));
    add(SettingSpec::toggle(
        "melee_weapon_on_breakable_block",
        format!("{SETTING_PREFIX}melee_weapon_on_breakable_block"),
        false,
    ));
    add(SettingSpec::choice(
        "holding_ranged_weapon",
        format!("{SETTING_PREFIX}holding_ranged_weapon"),
        usable_options.clone(),
        0,
    ));
    add(SettingSpec::choice(
        "holding_throwable",
        format!("{SETTING_PREFIX}holding_throwable"),
        usable_options.clone(),
        1,
    ));
    add(SettingSpec::toggle(
        "holding_shield",
        format!("{SETTING_PREFIX}holding_shield"),
        true,
    ));
    add(SettingSpec::choice(
        "holding_block",
        format!("{SETTING_PREFIX}holding_block"),
        block_options,
        2,
    ));
    add(SettingSpec::toggle(
        "holding_block_in_offhand",
        format!("{SETTING_PREFIX}holding_block_in_offhand"),
        true,
    ));
    add(SettingSpec::choice(
        "holding_usable_item",
        format!("{SETTING_PREFIX}holding_usable_item"),
        usable_options,
        1,
    ));
    add(SettingSpec::toggle(
        "force_holding_spyglass",
        format!("{SETTING_PREFIX}force_holding_spyglass"),
        false,
    ));

    add(SettingSpec::toggle(
        "enable_tweaks",
        format!("{SETTING_PREFIX}enable_tweaks"),
        true,
    ));
    add(SettingSpec::toggle(
        "color_override",
        format!("{SETTING_PREFIX}color_override"),
        false,
    ));
    add(SettingSpec::color(
        "color",
        format!("{SETTING_PREFIX}color"),
        0xFF_FFFF,
    ));
    add(SettingSpec::toggle(
        "color_blend",
        format!("{SETTING_PREFIX}color_blend"),
        true,
    ));

    for slot in STYLE_SLOTS {
        let key = slot.key();
        add(SettingSpec::choice(
            key,
            format!("{SETTING_PREFIX}{key}"),
            style_labels.clone(),
            Config::default_style_index(slot),
        ));
        let default_color = slot.default_color();
        add(SettingSpec::toggle(
            format!("{key}_override_color"),
            format!("{SETTING_PREFIX}{key}_override_color"),
            default_color.is_some(),
        ));
        add(SettingSpec::color(
            format!("{key}_color"),
            format!("{SETTING_PREFIX}{key}_color"),
            default_color.unwrap_or(0xFF_FFFF),
        ));
        add(SettingSpec::toggle(
            format!("{key}_blend"),
            format!("{SETTING_PREFIX}{key}_blend"),
            slot.default_blend(),
        ));
        add(SettingSpec::toggle(
            format!("{key}_coalesce"),
            format!("{SETTING_PREFIX}{key}_coalesce"),
            !matches!(
                slot,
                StyleSlot::HoldingRangedWeapon | StyleSlot::HoldingBlock
            ),
        ));
    }
}
