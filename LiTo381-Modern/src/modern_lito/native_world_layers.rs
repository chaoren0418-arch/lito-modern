//! Terrain and body fragments share source half-cell depth and pixel ownership.
//! Display interpolation is modern: choose the nearest whole-cell anchor and
//! translate its fragments by the remaining projected pixels. This is not a
//! reconstruction of the native timer's insertion history or roof state.
use super::{
    compositor::Canvas,
    native_multisector_scene::Scene,
    sprite::SpriteFrame,
    world_depth::NativeDrawCell,
    world_map::{PixelPoint, ViewProjection},
    world_raster::{Blend, DrawVisibility, Raster},
};
use anyhow::{ensure, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy)]
pub struct Body<'a> {
    pub id: u32,
    /// Projected whole-cell center, including current motion interpolation.
    pub center: PixelPoint,
    pub frame: &'a SpriteFrame,
}
pub struct Attachment<'a> {
    pub frame: &'a SpriteFrame,
    pub mix: super::native_actor_layers::Mix,
}
pub struct LayeredBody<'a> {
    pub body: Body<'a>,
    pub attachments: Vec<Attachment<'a>>,
    pub shadow: Option<&'a SpriteFrame>,
}
pub struct Output {
    pub canvas: Canvas,
    pub visibility: Vec<(u32, DrawVisibility)>,
    pub terrain_fragments: usize,
    /// Qualified floor and plain RLE object writes. A raw object preview makes
    /// provenance incomplete until its original RLE coverage is implemented.
    /// Background, camera conversion and unsupported flags remain separate.
    pub background_gate: Option<super::native_terrain_pixels::Gate>,
}
pub fn compose(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[Body<'_>],
    cancel: &AtomicBool,
) -> Result<Output> {
    let layered: Vec<_> = bodies
        .iter()
        .map(|body| LayeredBody {
            body: *body,
            attachments: Vec::new(),
            shadow: None,
        })
        .collect();
    compose_with_layers(scene, view, width, height, &layered, cancel)
}
pub fn compose_with_layers(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[LayeredBody<'_>],
    cancel: &AtomicBool,
) -> Result<Output> {
    compose_with_roofs(
        scene,
        view,
        width,
        height,
        bodies,
        &Default::default(),
        cancel,
    )
}
pub fn compose_with_roofs(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[LayeredBody<'_>],
    roofs: &super::native_world_roofs::Frame,
    cancel: &AtomicBool,
) -> Result<Output> {
    compose_with_effects(scene, view, width, height, bodies, roofs, &[], cancel)
}
pub fn compose_with_effects(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[LayeredBody<'_>],
    roofs: &super::native_world_roofs::Frame,
    effects: &[super::native_world_effects::Effect<'_>],
    cancel: &AtomicBool,
) -> Result<Output> {
    compose_with_states(
        scene,
        view,
        width,
        height,
        bodies,
        roofs,
        effects,
        &vec![Default::default(); bodies.len()],
        cancel,
    )
}
pub fn compose_with_states(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[LayeredBody<'_>],
    roofs: &super::native_world_roofs::Frame,
    effects: &[super::native_world_effects::Effect<'_>],
    states: &[super::native_actor_states::Style],
    cancel: &AtomicBool,
) -> Result<Output> {
    let presentations: Vec<_> = states.iter().map(|&state|
        super::native_actor_states::Presentation { state, body_half: false }
    ).collect();
    compose_with_presentations(scene, view, width, height, bodies, roofs, effects, &presentations, cancel)
}
pub fn compose_with_presentations(
    scene: &Scene,
    view: ViewProjection,
    width: u32,
    height: u32,
    bodies: &[LayeredBody<'_>],
    roofs: &super::native_world_roofs::Frame,
    effects: &[super::native_world_effects::Effect<'_>],
    presentations: &[super::native_actor_states::Presentation],
    cancel: &AtomicBool,
) -> Result<Output> {
    ensure!(
        roofs.matches_definition(scene.roofs()),
        "roof frame belongs to different geometry"
    );
    ensure!(
        !cancel.load(Ordering::Acquire),
        "world composition cancelled"
    );
    ensure!(
        bodies.len() <= 128 && width > 0 && height > 0 && width <= 1024 && height <= 1024,
        "world composition budget"
    );
    ensure!(presentations.len() == bodies.len(), "actor state roster mismatch");
    ensure!(
        presentations.iter().all(
            |s| !matches!(s.state, super::native_actor_states::Style::Green(n) if !(8..=14).contains(&n))
        ),
        "actor green boost outside qualified table range"
    );
    let mut ids = std::collections::BTreeSet::new();
    ensure!(
        bodies
            .iter()
            .all(|b| b.body.id != 0 && ids.insert(b.body.id)),
        "duplicate/zero body identity"
    );
    let mut canvas = Canvas::new(width, height)?;
    let mut raster = Raster::new(view, &canvas);
    let terrain_fragments = scene.enqueue(&mut raster, view, height, roofs, cancel)?;
    for (index, layers) in bodies.iter().enumerate() {
        use super::native_actor_states::Style;
        let style = presentations[index].state;
        let body_style = presentations[index].body_style();
        if style == Style::Hidden {
            continue;
        }
        let body = &layers.body;
        ensure!(layers.attachments.len() <= 8, "actor attachment budget");
        ensure!(
            !cancel.load(Ordering::Acquire),
            "world composition cancelled"
        );
        let cell = view
            .pixel_to_cell(body.center)
            .context("body display anchor")?;
        let center = view.cell_to_pixel(cell).context("body projected anchor")?;
        let shift = PixelPoint {
            x: body
                .center
                .x
                .checked_sub(center.x)
                .context("body residual X")?,
            y: body
                .center
                .y
                .checked_sub(center.y)
                .context("body residual Y")?,
        };
        let origin = NativeDrawCell::from_world(cell).context("body half-cell anchor")?;
        raster.sprite_shifted(
            origin,
            body.frame,
            u16::try_from(index + 1)?,
            if body_style == Style::Normal {
                Blend::Over
            } else {
                Blend::Actor(body_style)
            },
            shift,
        )?;
        for attachment in &layers.attachments {
            // 4E7000: only the half state reaches an attachment (actor word
            // 0x100, or the attribute-100 rewrite); green and dim do not.
            let blend = match attachment.mix.submitted(style == Style::Half) {
                super::native_actor_layers::Mix::Over => Blend::Over,
                super::native_actor_layers::Mix::Half => Blend::Actor(Style::Half),
                super::native_actor_layers::Mix::Screen => Blend::Screen,
                super::native_actor_layers::Mix::Multiply => Blend::Multiply,
            };
            raster.sprite_shifted(
                origin,
                attachment.frame,
                u16::try_from(index + 1)?,
                blend,
                shift,
            )?;
        }
        if let Some(shadow) = layers.shadow {
            raster.sprite_shifted(origin, shadow, 0, Blend::Shadow, shift)?;
        }
    }
    enqueue_effects(&mut raster,effects,cancel)?;
    let mut background_gate=if scene.assets().verified_plan().map_id==303 {Some(super::native_terrain_pixels::Gate::new(width,height)?)}else{None};
    let visibility = raster
        .paint_with_background_gate(&mut canvas, cancel, bodies.len(),background_gate.as_mut())?
        .into_iter()
        .zip(bodies)
        .map(|(mask, body)| (body.body.id, mask))
        .collect();
    Ok(Output {
        canvas,
        visibility,
        terrain_fragments,
        background_gate,
    })
}

pub(crate) fn enqueue_effects<'a>(raster:&mut Raster<'a>,effects:&[super::native_world_effects::Effect<'a>],cancel:&AtomicBool)->Result<()> {
    ensure!(effects.len() <= 512, "effect submission budget");
    for effect in effects {
        ensure!(
            !cancel.load(Ordering::Acquire),
            "effect composition cancelled"
        );
        let origin =
            NativeDrawCell::from_world(effect.position).context("effect half-cell anchor")?;
        let blend = match effect.mix {
            super::native_world_effects::Mix::Over => Blend::EffectOver,
            super::native_world_effects::Mix::Screen => Blend::Screen,
            super::native_world_effects::Mix::Shadow => Blend::Shadow,
            super::native_world_effects::Mix::Multiply => Blend::Multiply,
        };
        raster.sprite(origin, effect.frame, 0, blend)?;
    }
    Ok(())
}
