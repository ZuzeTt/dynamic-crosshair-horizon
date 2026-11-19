//! Drawing the crosshair parts through the HUD's GUI batch.

use std::borrow::Cow;

use mod_api::client::hud_layout::Blend;
use mod_api::client::hud_layout::PoseOp;
use mod_api::{DrawContext, GuiDraw};

use crate::style::Part;

pub const CROSSHAIR_SIZE: f32 = 15.0;

fn texture_key(sprite: &str) -> String {
    match sprite.strip_prefix("minecraft:") {
        Some(path) => format!("gui/sprites/{path}"),
        None => sprite.to_owned(),
    }
}

/// Draw the parts where vanilla draws its 15x15 sprite. With
/// `fix_centered`, a fractional translation puts them on the exact centre;
/// otherwise the integer top-left vanilla uses is kept.
///
/// The original inverts the pixels under an `enableBlend` style. Horizon's
/// mod GUI draw list has no invert blend for a mod namespace (`GuiDraw::Sprite`
/// prepends `gui/sprites/`, so it cannot name a mod sprite, and `Blit` only
/// draws texturing), so the sprite is drawn with straight alpha and its
/// configured colour.
pub fn draw(draw: &mut dyn DrawContext, parts: &[Part], fix_centered: bool) {
    if parts.is_empty() {
        return;
    }
    let [width, height] = draw.size();
    let x = ((width - CROSSHAIR_SIZE) / 2.0).floor();
    let y = ((height - CROSSHAIR_SIZE) / 2.0).floor();
    let mut draws = Vec::with_capacity(parts.len() + 2);
    if fix_centered {
        let dx = (width - CROSSHAIR_SIZE) / 2.0 - x;
        let dy = (height - CROSSHAIR_SIZE) / 2.0 - y;
        draws.push(GuiDraw::Push(vec![PoseOp::Translate(dx, dy)]));
    }
    for part in parts {
        draws.push(GuiDraw::Blit {
            texture: Cow::Owned(texture_key(&part.sprite)),
            blend: Blend::Textured,
            x: x as i32,
            y: y as i32,
            w: CROSSHAIR_SIZE as i32,
            h: CROSSHAIR_SIZE as i32,
            uv: [0.0, 0.0],
            src: [CROSSHAIR_SIZE as i32, CROSSHAIR_SIZE as i32],
            tex: [CROSSHAIR_SIZE as i32, CROSSHAIR_SIZE as i32],
            color: part.color,
        });
    }
    if fix_centered {
        draws.push(GuiDraw::Pop);
    }
    let _ = draw.gui_paint(&draws);
}

/// The attack indicator vanilla keeps beside the crosshair, rebuilt from the
/// sprites and the current attack strength when this mod has to replace the
/// whole crosshair layer.
pub fn draw_attack_indicator(draw: &mut dyn DrawContext, scale: f32, delay: f32, full: bool) {
    let [width, height] = draw.size();
    let x = (width / 2.0).floor() - 8.0;
    let y = (height / 2.0).floor() - 7.0 + 16.0;
    if full {
        let _ = draw.sprite(
            "gui/sprites/hud/crosshair_attack_indicator_full",
            [0.0, 0.0, 16.0, 16.0],
            [x, y, 16.0, 16.0],
            Some(0xFFFF_FFFF),
        );
    } else if scale < 1.0 {
        let _ = draw.sprite(
            "gui/sprites/hud/crosshair_attack_indicator_background",
            [0.0, 0.0, 16.0, 4.0],
            [x, y, 16.0, 4.0],
            Some(0xFFFF_FFFF),
        );
        let progress = (scale * 17.0).floor();
        let _ = draw.sprite(
            "gui/sprites/hud/crosshair_attack_indicator_progress",
            [0.0, 0.0, progress, 4.0],
            [x, y, progress, 4.0],
            Some(0xFFFF_FFFF),
        );
        let _ = delay;
    }
}
