//! Shared-only, base-action presentation for the reviewed logical form5641.
//!
//! Shared Text/list.spr 523c61ad...b9f127 gives5641 its OWN action table and
//! FPS scopes (walk/attack48; idle/damage/death24), but graphics bank240.
//! Bank240's existing bounded pixel/shadow/flags8 loader may be reused ONLY
//! after proving identical source-frame/layer routes. Its24FPS clock and NPC
//! identity are NOT reused. Only the reviewed missing-family reset is supported;
//! no spell, random extra attack, equipment permission, scene admission,
//! authority or network command is inferred.
//!
//! Native evidence: sub_4E0490 independently stores logical form at actor+14h
//! and action family at+18h. Sprite attributes101/105 route the body image and
//! subframe to shadow241/attachment242 (see npc_assets' layer evidence).
//! This component does not remove battle_scene's full-VIP presentation gate.

use super::{
    combat_animation::NativeAudioGroup,
    combat_audio::SoundBank,
    compositor::Canvas,
    fixed_walk::QualifiedFixedWalk,
    npc_assets::{NpcAction, NpcAppearance, NpcAssets, NpcLayer, QualifiedNpcAction},
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::{ActionFrame, ActionKey, Animation, FrameAsset, SpriteActionCatalog},
    teleport_effect::SpriteProvenance,
    world_map::PixelPoint,
    world_scene::BaseAction,
};
use anyhow::{ensure, Context, Result};
#[path = "polymorph_cast.rs"]
pub(crate) mod cast;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

const ACTIONS: [BaseAction; 5] = [
    BaseAction::Walk,
    BaseAction::Attack,
    BaseAction::Damage,
    BaseAction::Idle,
    BaseAction::Death,
];

/// Logical form identity, not a graphics-bank ID or an equipment permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PolymorphForm(u32);
impl PolymorphForm {
    /// The wide wire value is checked, never truncated to a native WORD/u32.
    /// Zero means no override; callers must use the confirmed base appearance.
    pub fn from_wire(graphics: i64) -> Result<Option<Self>> {
        match graphics {
            0 => Ok(None),
            5641 => Ok(Some(Self(5641))),
            _ => anyhow::bail!("unqualified shared polymorph logical ID {graphics}"),
        }
    }
    pub fn logical_sprite_id(self) -> u32 {
        self.0
    }
    pub fn graphics_id(self) -> u32 {
        240
    }
    fn bank(self) -> Result<NpcAppearance> {
        NpcAppearance::new(self.graphics_id())
    }

    /// Native4E5C00/4E5CA7 checks the requested family, then actor+18h;
    /// 4E5CC5 resets that family to0 when its action is absent. This is NOT
    /// a missing-SPR fallback.5641 has only the reviewed basic family; a new,
    /// malformed or aliased armed definition is not evidence for this route.
    pub fn qualify_weapon(
        self,
        catalog: &SpriteActionCatalog,
        weapon_type: u8,
    ) -> Result<PolymorphStance> {
        let requested_family = super::combat_animation::weapon_stance_base(weapon_type)?;
        for action in ACTIONS {
            QualifiedPolymorphAction::from_catalog(catalog, self, action)?;
        }
        let logical = &catalog.sprites[&self.0];
        if requested_family != 0 {
            ensure!(
                !(requested_family..=requested_family + 3)
                    .any(|id| logical.actions.contains_key(&ActionKey::new(id))),
                "polymorph armed family changed; no implicit downgrade"
            );
        }
        Ok(PolymorphStance {
            form: self,
            weapon_type,
            requested_family,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolymorphStance {
    form: PolymorphForm,
    weapon_type: u8,
    requested_family: u32,
}
impl PolymorphStance {
    pub fn form(self) -> PolymorphForm {
        self.form
    }
    pub fn weapon_type(self) -> u8 {
        self.weapon_type
    }
    pub fn requested_family(self) -> u32 {
        self.requested_family
    }
    pub fn effective_family(self) -> u32 {
        0
    }
}

fn bank_action(action: BaseAction) -> NpcAction {
    match action {
        BaseAction::Idle => NpcAction::Breath,
        BaseAction::Walk => NpcAction::Walk,
        BaseAction::Attack => NpcAction::Attack,
        BaseAction::Damage => NpcAction::Damage,
        BaseAction::Death => NpcAction::Death,
    }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "polymorph asset load cancelled"
    );
    Ok(())
}

#[derive(Clone)]
pub struct QualifiedPolymorphAction {
    form: PolymorphForm,
    action: BaseAction,
    animation: Animation,
    walk: Option<QualifiedFixedWalk>,
    durations: Vec<Duration>,
    duration: Duration,
    groups: Vec<Option<NativeAudioGroup>>,
}
impl QualifiedPolymorphAction {
    pub fn from_catalog(
        catalog: &SpriteActionCatalog,
        form: PolymorphForm,
        action: BaseAction,
    ) -> Result<Self> {
        let logical = catalog
            .sprites
            .get(&form.0)
            .context("polymorph logical definition absent")?;
        ensure!(
            logical.id == form.0
                && logical.graphics_id == form.graphics_id()
                && logical.image_count == Some(64)
                && logical.header_issue.is_none()
                && !logical.actions.keys().any(|key| key.id >= 100),
            "unreviewed polymorph header/control"
        );
        let expected = [
            (101, b"241".as_slice()),
            (102, b"10".as_slice()),
            (105, b"1 242".as_slice()),
        ];
        ensure!(
            logical.attributes.len() == expected.len() + 2,
            "unreviewed polymorph attributes"
        );
        // The parser retains both110 declarations as well as applying their
        // scoped rates to actions. They must not be mistaken for extra layers.
        let rates: Vec<_> = logical
            .attributes
            .iter()
            .filter(|a| a.key == ActionKey::new(110))
            .map(|a| a.body_bytes.as_slice())
            .collect();
        ensure!(
            rates == [b"48".as_slice(), b"24".as_slice()],
            "unreviewed polymorph FPS scopes"
        );
        for (id, bytes) in expected {
            let mut values = logical
                .attributes
                .iter()
                .filter(|a| a.key == ActionKey::new(id));
            ensure!(
                values.next().is_some_and(|a| a.body_bytes == bytes) && values.next().is_none(),
                "unreviewed polymorph layer attribute"
            );
        }
        // This proof owns the bank's native layer/cue rules, NOT the form's FPS.
        let bank = QualifiedNpcAction::from_catalog(catalog, form.bank()?, bank_action(action))?;
        let resolved = catalog.resolve_action(form.0, bank_action(action).key())?;
        ensure!(
            resolved.definition_sprite_id == form.0,
            "polymorph action alias is unqualified"
        );
        let animation = resolved.animation;
        let fps = if matches!(action, BaseAction::Walk | BaseAction::Attack) {
            48
        } else {
            24
        };
        ensure!(
            animation.frames_per_second == fps
                && animation.directionality == super::sprite_actions::Directionality::EightWay
                && animation.label_bytes.len() <= 256 * 1024
                && animation.frames == bank.source_frames(),
            "polymorph action does not match reviewed pixel/cue routes and FPS"
        );
        let walk = if action == BaseAction::Walk {
            let w = QualifiedFixedWalk::from_catalog(catalog, form.0)?;
            ensure!(
                w.provenance().definition_sprite_id == form.0,
                "polymorph walk alias"
            );
            Some(w)
        } else {
            None
        };
        let mut durations = Vec::with_capacity(animation.frames.len());
        let mut duration = Duration::ZERO;
        let mut groups = Vec::with_capacity(animation.frames.len());
        for (source, frame) in animation.frames.iter().enumerate() {
            let ms = frame
                .duration_ticks
                .checked_mul(1000)
                .context("polymorph duration overflow")?
                / fps;
            ensure!(
                (1..=u32::from(u16::MAX)).contains(&ms),
                "polymorph duration outside native WORD"
            );
            let d = Duration::from_millis(u64::from(ms));
            duration = duration
                .checked_add(d)
                .context("polymorph action duration overflow")?;
            durations.push(d);
            groups.push(bank.sound_group(source)?.cloned());
        }
        Ok(Self {
            form,
            action,
            animation: animation.clone(),
            walk,
            durations,
            duration,
            groups,
        })
    }
    pub fn form(&self) -> PolymorphForm {
        self.form
    }
    pub fn action(&self) -> BaseAction {
        self.action
    }
    pub fn frames_per_second(&self) -> u32 {
        self.animation.frames_per_second
    }
    pub fn source_frames(&self) -> &[ActionFrame] {
        &self.animation.frames
    }
    pub fn duration(&self) -> Duration {
        self.duration
    }
    pub fn source_duration(&self, source: usize) -> Result<Duration> {
        self.durations
            .get(source)
            .copied()
            .context("polymorph duration source outside action")
    }
    /// Frame-entry cues use the same native WORD milliseconds as sample(),
    /// not cumulative tick/FPS division (which has different rounding).
    pub(crate) fn audio_phases(&self) -> Result<Vec<(Duration, NativeAudioGroup)>> {
        ensure!(
            matches!(
                self.action,
                BaseAction::Attack | BaseAction::Damage | BaseAction::Death
            ),
            "polymorph one-shot audio action unqualified"
        );
        let mut phase = Duration::ZERO;
        let mut cues = Vec::new();
        for source in (1..self.durations.len()).chain(std::iter::once(0)) {
            if let Some(group) = self.sound_group(source)? {
                cues.push((phase, group.clone()));
            }
            phase = phase
                .checked_add(self.source_duration(source)?)
                .context("polymorph audio phase overflow")?;
        }
        ensure!(phase == self.duration, "polymorph audio duration mismatch");
        Ok(cues)
    }
    pub fn walk(&self) -> Option<&QualifiedFixedWalk> {
        self.walk.as_ref()
    }
    pub fn sound_group(&self, source: usize) -> Result<Option<&NativeAudioGroup>> {
        Ok(self
            .groups
            .get(source)
            .context("polymorph cue source outside action")?
            .as_ref())
    }
    /// Native1..N-1,0 order and per-source WORD milliseconds. Idle repeats;
    /// all other actions hold terminal0. Callers own cue de-duplication and
    /// authoritative pace changes; sampling never emits audio/hits or moves a cell.
    pub fn sample(&self, elapsed: Duration, heading: u8) -> Result<(usize, u128)> {
        ensure!(heading < 8, "polymorph heading outside0..7");
        if let Some(walk) = &self.walk {
            return Ok((
                walk.start_unpaced_step(heading)?
                    .advance(elapsed, &AtomicBool::new(false))?
                    .source_frame,
                0,
            ));
        }
        let repeating = self.action == BaseAction::Idle;
        if !repeating && elapsed >= self.duration {
            return Ok((0, 0));
        }
        let cycle = if repeating {
            elapsed.as_nanos() / self.duration.as_nanos()
        } else {
            0
        };
        let phase = elapsed.as_nanos() % self.duration.as_nanos();
        let mut end = 0u128;
        for source in (1..self.durations.len()).chain(std::iter::once(0)) {
            end += self.durations[source].as_nanos();
            if phase < end {
                return Ok((source, cycle));
            }
        }
        anyhow::bail!("polymorph phase outside qualified duration")
    }
    pub fn body_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        Ok(self
            .animation
            .frame_asset(self.form.graphics_id(), source, heading)?)
    }
}

/// Owned pixels plus the logical form's independent animation certificates.
/// The reused bank is a private pixel container, never a replacement actor.
pub struct PolymorphAssets {
    form: PolymorphForm,
    actions: BTreeMap<BaseAction, QualifiedPolymorphAction>,
    pixels: NpcAssets,
}
impl PolymorphAssets {
    pub fn load(root: &Path, form: PolymorphForm, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        Self::load_with_catalog(&SharedActionCatalog::load(root, cancel)?, form, cancel)
    }
    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        form: PolymorphForm,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let mut actions = BTreeMap::new();
        // Validate every route before any SPR/WAV read. No partial result on failure.
        for action in ACTIONS {
            actions.insert(
                action,
                QualifiedPolymorphAction::from_catalog(context.catalog(), form, action)?,
            );
        }
        active(cancel)?;
        let pixels = NpcAssets::load_with_catalog(context, &[form.bank()?], cancel)?;
        let assets = Self {
            form,
            actions,
            pixels,
        };
        for action in ACTIONS {
            for source in 0..assets.action(action)?.source_frames().len() {
                for heading in 0..8 {
                    assets.frame(action, heading, source)?;
                }
            }
        }
        active(cancel)?;
        Ok(assets)
    }
    pub fn form(&self) -> PolymorphForm {
        self.form
    }
    pub fn action(&self, action: BaseAction) -> Result<&QualifiedPolymorphAction> {
        self.actions
            .get(&action)
            .context("polymorph action not preloaded")
    }
    pub fn frame(
        &self,
        action: BaseAction,
        heading: u8,
        source: usize,
    ) -> Result<PolymorphFrame<'_>> {
        let asset = self.action(action)?.body_asset(source, heading)?;
        let frame = self
            .pixels
            .frame(self.form.bank()?, bank_action(action), heading, source)?;
        ensure!(
            frame.body.asset == asset
                && frame.shadow.asset
                    == FrameAsset {
                        graphics_id: 241,
                        ..asset
                    }
                && frame.attachment.as_ref().is_some_and(|layer| layer.asset
                    == FrameAsset {
                        graphics_id: 242,
                        ..asset
                    }),
            "polymorph pixel layer route changed"
        );
        Ok(PolymorphFrame {
            form: self.form,
            action,
            heading,
            source,
            body: frame.body,
            shadow: frame.shadow,
            attachment: frame.attachment.context("polymorph attachment absent")?,
        })
    }
    /// Flat reference compositor; does not grant native actor occlusion/order.
    /// All frame/anchor checks happen before destination pixels are touched.
    pub fn draw_frame(
        &self,
        canvas: &mut Canvas,
        action: BaseAction,
        heading: u8,
        source: usize,
        anchor: PixelPoint,
    ) -> Result<()> {
        self.frame(action, heading, source)?;
        self.pixels.draw_frame(
            canvas,
            self.form.bank()?,
            bank_action(action),
            heading,
            source,
            anchor,
        )
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.pixels.sprite_sources()
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        self.pixels.sound_bank()
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.pixels.catalog_sha256()
    }
    pub fn encoded_bytes(&self) -> usize {
        self.pixels.encoded_bytes()
    }
    pub fn rgba_bytes(&self) -> usize {
        self.pixels.rgba_bytes()
    }
}
pub struct PolymorphFrame<'a> {
    pub form: PolymorphForm,
    pub action: BaseAction,
    pub heading: u8,
    pub source: usize,
    pub body: NpcLayer<'a>,
    pub shadow: NpcLayer<'a>,
    pub attachment: NpcLayer<'a>,
}

#[cfg(test)]
#[path = "polymorph_assets_tests.rs"]
mod tests;
