//! Qualified overlays with separate lazy skill, potion, and Life Stream banks.
#[cfg(test)]
#[path = "native_t021_render_tests.rs"]
mod t021_tests;
use super::{
    compositor::Canvas,
    native_combat_animation::EnteredEvent,
    native_effect_timing::{Clip, Playback},
    native_skills::Effect,
    native_world::World,
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::{ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog},
    tower_assets::{HonorAssetError, SharedEffectBank},
    world_map::{ViewProjection, WorldCell},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

fn qualify(
    catalog: &SpriteActionCatalog,
    graphics: &[u32],
) -> super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let mut animations = Vec::new();
    for &gfx in graphics {
        let row = catalog
            .sprites
            .get(&gfx)
            .ok_or(HonorAssetError::Unsupported("native effect absent"))?;
        let action = catalog.resolve_action(gfx, ActionKey::new(0))?;
        let a = action.animation;
        if row.graphics_id != gfx
            || row.header_issue.is_some()
            || row.actions.len() != 1
            || row
                .attributes
                .iter()
                .any(|v| v.key != ActionKey::new(104) || v.body_bytes.trim_ascii() != b"8")
            || a.directionality != Directionality::Undirected
            || a.frames_per_second != 24
            || a.frames.len() < 2
            || a.frames
                .iter()
                .any(|f| f.events.iter().any(|e| !matches!(e, FrameEvent::Sound(_))))
        {
            return Err(HonorAssetError::Unsupported(
                "unqualified native effect controls",
            ));
        }
        animations.push((gfx, 0, a.clone()));
    }
    Ok(animations)
}
/// Exact original #169 class-0 teleport. Source0 is the terminal dwell;
/// source1 starts playback and carries sound119.
pub fn qualify_travel(c: &SpriteActionCatalog) -> super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let rows = qualify(c, &[169])?;
    let row = &c.sprites[&169];
    let a = &rows[0].2;
    if row.image_count != Some(1) || row.attributes.len() != 1
        || a.frames.len() != 7 || a.frames.iter().enumerate().any(|(n,f)|
            f.image_id != 0 || f.subframe != if n == 0 {6} else {n as u32 - 1}
            || f.duration_ticks != 1
            || f.events != if n == 1 {vec![FrameEvent::Sound(119)]} else {vec![]}) {
        return Err(HonorAssetError::Unsupported("teleport original definition changed"));
    }
    Ok(rows)
}
pub fn qualify_skills(
    c: &SpriteActionCatalog,
) -> super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let mut animations = qualify(c, &[221, 744, 745, 749])?;
    // Original #2510 explicitly binds the single149 bitmap. Only action0 is
    // the S37 finite effect; action3 is not selected as a spell-duration loop.
    let row=c.sprites.get(&2510).ok_or(HonorAssetError::Unsupported("light absent"))?;
    let a=c.resolve_action(2510,ActionKey::new(0))?.animation;
    let breath=c.resolve_action(2510,ActionKey::new(3))?.animation;
    if row.graphics_id!=149 || row.image_count!=Some(1) || row.header_issue.is_some()
        || row.actions.len()!=2 || !row.attributes.is_empty()
        || [a,breath].iter().any(|a|a.directionality!=Directionality::Undirected || a.frames_per_second!=24)
        || a.frames.len()!=1 || a.frames[0].image_id!=0 || a.frames[0].subframe!=0
        || a.frames[0].duration_ticks!=4 || !a.frames[0].events.is_empty()
        || breath.frames.len()!=2 || breath.frames.iter().enumerate().any(|(n,f)|
            f.image_id!=0 || f.subframe!=0 || f.duration_ticks!=if n==0 {4}else{144} || !f.events.is_empty()) {
        return Err(HonorAssetError::Unsupported("light original definition changed"));
    }
    animations.push((2510,0,a.clone()));
    // These three original definitions each own action0 and bind image744.
    // Preserve their wire IDs and individual sounds; this is not a recursive
    // fallback or a translation of the separate TW11723..11725 namespace.
    for (gfx, sound) in [(829, 114), (830, 115), (831, 126)] {
        let row = c.sprites.get(&gfx)
            .ok_or(HonorAssetError::Unsupported("higher healing absent"))?;
        let resolved = c.resolve_action(gfx, ActionKey::new(0))?;
        let a = resolved.animation;
        if row.graphics_id != 744 || row.header_issue.is_some()
            || row.image_count != Some(1) || row.actions.len() != 1
            || resolved.definition_sprite_id != gfx
            || row.attributes.len() != 1
            || row.attributes[0].key != ActionKey::new(104)
            || row.attributes[0].body_bytes.trim_ascii() != b"8"
            || a.directionality != Directionality::Undirected
            || a.frames_per_second != 24 || a.frames.len() != 21
            || a.frames.iter().enumerate().any(|(n, f)|
                f.image_id != 0 || f.subframe != if n == 0 {20} else {n as u32 - 1}
                || f.duration_ticks != 3
                || f.events != if n == 1 {vec![FrameEvent::Sound(sound)]} else {vec![]})
        {
            return Err(HonorAssetError::Unsupported("higher healing original definition changed"));
        }
        animations.push((gfx, 0, a.clone()));
    }
    animations.extend(qualify_summon_circle(c)?);
    // Original #2165 is class 0 with an explicit type(0) declaration.
    // Admit only this exact control/frame definition, without widening the
    // generic class-0 qualification for other effects.
    let row = c.sprites.get(&2165)
        .ok_or(HonorAssetError::Unsupported("holy weapon absent"))?;
    let a = c.resolve_action(2165, ActionKey::new(0))?.animation;
    if row.graphics_id != 2165 || row.header_issue.is_some()
        || row.image_count != Some(1) || row.actions.len() != 1
        || row.attributes.len() != 2
        || [102, 104].iter().any(|key|
            row.attributes.iter().filter(|v| v.key == ActionKey::new(*key)).count() != 1)
        || row.attributes.iter().any(|v| !(
            (v.key == ActionKey::new(102) && v.body_bytes.trim_ascii() == b"0")
            || (v.key == ActionKey::new(104) && v.body_bytes.trim_ascii() == b"8")))
        || a.directionality != Directionality::Undirected
        || a.frames_per_second != 24 || a.frames.len() != 13
        || a.frames.iter().enumerate().any(|(n, f)|
            f.image_id != 0 || f.subframe != if n == 0 {12} else {n as u32 - 1}
            || f.duration_ticks != 4
            || f.events != if n == 1 {vec![FrameEvent::Sound(804)]} else {vec![]})
    {
        return Err(HonorAssetError::Unsupported("holy weapon original definition changed"));
    }
    animations.push((2165, 0, a.clone()));
    let row = c.sprites.get(&871)
        .ok_or(HonorAssetError::Unsupported("cure poison absent"))?;
    let a = c.resolve_action(871, ActionKey::new(0))?.animation;
    if row.graphics_id != 192 || row.header_issue.is_some()
        || row.image_count != Some(1) || row.actions.len() != 1
        || row.attributes.len() != 1
        || row.attributes[0].key != ActionKey::new(104)
        || row.attributes[0].body_bytes.trim_ascii() != b"8"
        || a.directionality != Directionality::Undirected
        || a.frames_per_second != 24 || a.frames.len() != 6
        || a.frames.iter().enumerate().any(|(n, f)|
            f.image_id != 0 || f.subframe != if n == 0 {5} else {n as u32 - 1}
            || f.duration_ticks != if n == 0 {3} else {1}
            || f.events != if n == 1 {vec![FrameEvent::Sound(141)]} else {vec![]})
    {
        return Err(HonorAssetError::Unsupported("cure poison original definition changed"));
    }
    animations.push((871, 0, a.clone()));
    animations.extend(qualify_turn_undead(c)?);
    Ok(animations)
}
pub fn qualify_summon_circle(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
 let row=c.sprites.get(&761).ok_or(HonorAssetError::Unsupported("summon circle absent"))?;let a=c.resolve_action(761,ActionKey::new(0))?.animation;
 if row.graphics_id!=761||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=1
  ||row.attributes[0].key!=ActionKey::new(104)||row.attributes[0].body_bytes.trim_ascii()!=b"8"||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=10
  ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{9}else{n as u32-1}||f.duration_ticks!=3||f.events!=if n==1{vec![FrameEvent::Sound(131)]}else{vec![]}) {return Err(HonorAssetError::Unsupported("summon circle original definition changed"));}
 Ok(vec![(761,0,a.clone())])
}
pub fn qualify_turn_undead(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&754).ok_or(HonorAssetError::Unsupported("turn undead absent"))?;
    let a=c.resolve_action(754,ActionKey::new(0))?.animation;
    let subs=[20,0,0,1,2,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19];
    let ticks=[0,0,1,2,2,1,1,1,1,1,6,1,1,1,1,1,2,2,2,3,3];
    if row.graphics_id!=754||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=1
        ||row.attributes[0].key!=ActionKey::new(104)||row.attributes[0].body_bytes.trim_ascii()!=b"8"
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=21
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=subs[n]||f.duration_ticks!=ticks[n]
            ||f.events!=if n==1{vec![FrameEvent::Sound(102)]}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("turn undead original definition changed"));
    }
    Ok(vec![(754,0,a.clone())])
}
fn qualify_items(
    c: &SpriteActionCatalog,
) -> super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    qualify(c, &[189, 194, 197])
}
// Life Stream action3 is an explicit modern presentation choice. The legacy
// S53/action0 path does not play this action. Never infer a server lease from it.
fn qualify_reagent(
    c: &SpriteActionCatalog,
) -> super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let row = c
        .sprites
        .get(&2231)
        .ok_or(HonorAssetError::Unsupported("life stream absent"))?;
    let a = c.resolve_action(2231, ActionKey::new(3))?.animation;
    if row.graphics_id != 2231
        || row.header_issue.is_some()
        || row.image_count != Some(1)
        || row.actions.len() != 1
        || row.attributes.len() != 2
        || row.attributes.iter().any(|v| {
            !((v.key == ActionKey::new(102) && v.body_bytes.trim_ascii() == b"0")
                || (v.key == ActionKey::new(104) && v.body_bytes.trim_ascii() == b"8"))
        })
        || a.directionality != Directionality::Undirected
        || a.frames_per_second != 24
        || a.frames.len() != 19
        || a.frames.iter().enumerate().any(|(n, f)| {
            f.image_id != 0
                || f.subframe != if n == 0 { 18 } else { n as u32 - 1 }
                || f.duration_ticks != 4
                || f.events
                    != if n == 1 {
                        vec![FrameEvent::Sound(841)]
                    } else {
                        vec![]
                    }
        })
    {
        return Err(HonorAssetError::Unsupported(
            "life stream original definition changed",
        ));
    }
    Ok(vec![(2231, 3, a.clone())])
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Profile {
    #[default]
    Skills,
    Items,
    Reagent,
    Travel,
}
impl Profile {
    fn graphics(self) -> &'static [u16] {
        match self {
            Self::Skills => &[221, 744, 745, 749, 754, 761, 829, 830, 831, 871, 2165, 2510],
            Self::Items => &[189, 194, 197],
            Self::Reagent => &[2231],
            Self::Travel => &[169],
        }
    }
    fn action(self) -> u8 {
        if self == Self::Reagent {
            3
        } else {
            0
        }
    }
}
#[derive(Clone)]
struct Active {
    cue: Effect,
    start: u64,
    playback: Playback,
}
/// Accepted finite callback state, with no sprite/PCM resource ownership.
#[derive(Clone)]
pub(crate) struct History {
    generation:Option<u64>,seen:BTreeSet<u64>,active:Vec<Active>,started:u64,omitted:u64,
}
#[derive(Default)]
pub struct Effects {
    profile: Profile,
    generation: Option<u64>,
    seen: BTreeSet<u64>,
    active: Vec<Active>,
    bank: Option<SharedEffectBank>,
    tw_heal_bank: Option<SharedEffectBank>,
    selector_bank: Option<SharedEffectBank>,
    selector_error: Option<String>,
    additional_bank: Option<SharedEffectBank>,
    additional_error: Option<String>,
    dark_elf_bank: Option<SharedEffectBank>,
    dark_elf_error: Option<String>,
    tw_heal_error: Option<String>,
    clips: BTreeMap<u16, Clip>,
    error: Option<String>,
    started: u64,
    omitted: u64,
}
impl Effects {
    pub(crate) fn history(&self)->History {
        History{generation:self.generation,seen:self.seen.clone(),active:self.active.clone(),started:self.started,omitted:self.omitted}
    }
    pub(crate) fn restore_history(&mut self,history:History) {
        self.generation=history.generation;self.seen=history.seen;self.active=history.active;
        self.started=history.started;self.omitted=history.omitted;
    }
    pub fn travel() -> Self { Self { profile: Profile::Travel, ..Self::default() } }
    pub fn reagent() -> Self {
        Self {
            profile: Profile::Reagent,
            ..Self::default()
        }
    }
    pub fn items() -> Self {
        Self {
            profile: Profile::Items,
            ..Self::default()
        }
    }
    pub fn prepare(&mut self,root:&Path,world:&World,cancel:&AtomicBool) {
        self.prepare_with_catalog(root,world,cancel,&mut None);
    }
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,world:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {
        let graphics = self.profile.graphics();
        if self.bank.is_none()
            && self.error.is_none()
            && ((self.profile == Profile::Skills
                && [1, 2, 3, 8, 9, 11, 13, 18].iter().any(|id| world.learned_skills.contains(id)))
                || world.effects.iter().chain(self.active.iter().map(|a|&a.cue)).any(|e| graphics.contains(&e.graphics)))
        {
            let loaded = (|| -> anyhow::Result<_> {
                let catalog = SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank = SharedEffectBank::load_with_graphics(
                    &catalog,
                    cancel,
                    match self.profile {
                        Profile::Skills => qualify_skills,
                        Profile::Items => qualify_items,
                        Profile::Reagent => qualify_reagent,
                        Profile::Travel => qualify_travel,
                    },
                    if self.profile == Profile::Skills { &[(871, 192), (829, 744), (830, 744), (831, 744), (2510,149)] } else { &[] },
                )?;
                let mut clips = BTreeMap::new();
                for &gfx in graphics {
                    clips.insert(
                        gfx,
                        if self.profile == Profile::Reagent {
                            Clip::new_explicit_action(Arc::clone(bank.animation(u32::from(gfx), self.profile.action())?))?
                        } else {
                            Clip::new(Arc::clone(bank.animation(u32::from(gfx), self.profile.action())?))?
                        },
                    );
                }
                Ok((bank, clips))
            })();
            match loaded {
                Ok((bank, clips)) => {
                    self.bank = Some(bank);
                    self.clips.extend(clips);
                }
                Err(e) => self.error = Some(format!("{e:#}")),
            }
        }
        // Preserve the old startup path. The nine-selector bank is demand-loaded
        // only when a real S37 names one of its exact qualified graphics IDs.
        if self.profile==Profile::Skills && self.selector_bank.is_none() && self.selector_error.is_none()
            && world.effects.iter().chain(self.active.iter().map(|a|&a.cue)).any(|e|super::native_selector_effects::GRAPHICS.contains(&e.graphics)) {
            let loaded=(||->anyhow::Result<_>{
                let context=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_reviewed_selectors(&context,cancel)?;
                let mut clips=BTreeMap::new();
                for gfx in super::native_selector_effects::GRAPHICS {
                    clips.insert(gfx,Clip::new(Arc::clone(bank.animation(u32::from(gfx),0)?))?);
                }
                Ok((bank,clips))
            })();
            match loaded {Ok((bank,clips))=>{self.clips.extend(clips);self.selector_bank=Some(bank);},
                Err(e)=>self.selector_error=Some(format!("{e:#}")),}
        }
        // Only exact additional finite definitions are admitted. Load after a
        // real cue names one; original startup work and all bounds stay intact.
        if self.profile==Profile::Skills && self.additional_bank.is_none() && self.additional_error.is_none()
            && world.effects.iter().chain(self.active.iter().map(|a|&a.cue))
                .any(|e|super::native_additional_finite_effects::GRAPHICS.contains(&e.graphics)) {
            let loaded=(||->anyhow::Result<_>{
                let context=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_reviewed_additional_finite(context,cancel)?;
                let mut clips=BTreeMap::new();
                for &gfx in super::native_additional_finite_effects::GRAPHICS {
                    clips.insert(gfx,Clip::new(Arc::clone(bank.animation(u32::from(gfx),0)?))?);
                }
                Ok((bank,clips))
            })();
            match loaded {
                Ok((bank,clips))=>{self.clips.extend(clips);self.additional_bank=Some(bank);},
                Err(e)=>self.additional_error=Some(format!("{e:#}")),
            }
        }
        // Exact T021 definitions, including shadow armor's inherited clothing slots.
        if self.profile == Profile::Skills && self.dark_elf_bank.is_none() && self.dark_elf_error.is_none()
            && world.effects.iter().chain(self.active.iter().map(|a| &a.cue))
                .any(|e| super::native_dark_elf_effects::GRAPHICS.contains(&e.graphics)) {
            let loaded = (|| -> anyhow::Result<_> {
                let context = SharedActionCatalog::reuse_for_model(root, cancel, shared)?;
                let bank = SharedEffectBank::load_reviewed_dark_elf(context, cancel)?;
                let mut clips = BTreeMap::new();
                for gfx in super::native_dark_elf_effects::GRAPHICS {
                    clips.insert(gfx, Clip::new(Arc::clone(bank.animation(u32::from(gfx), 0)?))?);
                }
                Ok((bank, clips))
            })();
            match loaded {
                Ok((bank, clips)) => { self.clips.extend(clips); self.dark_elf_bank = Some(bank); }
                Err(e) => self.dark_elf_error = Some(format!("{e:#}")),
            }
        }
        // S37's explicit11722..11725 selects the reviewed TW data namespace. It does
        // not rewrite a wire graphics ID or replace the legacy catalog.
        if self.profile == Profile::Skills && self.tw_heal_bank.is_none()
            && self.tw_heal_error.is_none() && world.effects.iter().chain(self.active.iter().map(|a|&a.cue)).any(|e|matches!(e.graphics,11722..=11725)) {
            let loaded = (|| -> anyhow::Result<_> {
                let context = super::tw_action_catalog::SharedTwActionCatalog::load_selected_from_launcher(
                    root, &[11722,11723,11724,11725,11726].into(), cancel)?;
                let bank = SharedEffectBank::load_reviewed_tw_healing_family(root, &context, cancel)?;
                let mut clips=BTreeMap::new();
                for gfx in 11722u16..=11725 {clips.insert(gfx,Clip::new(Arc::clone(bank.animation(u32::from(gfx),0)?))?);}
                Ok((bank,clips))
            })();
            match loaded {
                Ok((bank,clips)) => {self.clips.extend(clips);self.tw_heal_bank=Some(bank);}
                Err(e) => self.tw_heal_error = Some(format!("{e:#}")),
            }
        }
    }
    pub fn draw(
        &mut self,
        world: &World,
        canvas: &mut Canvas,
        projection: ViewProjection,
        now: u64,
    ) -> (serde_json::Value, Vec<EnteredEvent>) {
        self.draw_to(
            world,
            &mut super::native_world_effects::Target::Canvas(canvas),
            projection,
            now,
        )
    }
    pub fn draw_to<'s>(
        &'s mut self,
        world: &World,
        target: &mut super::native_world_effects::Target<'_, 's>,
        projection: ViewProjection,
        now: u64,
    ) -> (serde_json::Value, Vec<EnteredEvent>) {
        if self.generation != Some(world.generation) {
            self.generation = Some(world.generation);
            self.seen.clear();
            self.active.clear();
            self.started = 0;
            self.omitted = 0;
        }
        let valid: BTreeSet<_> = world.effects.iter().map(|e| e.sequence).collect();
        self.seen.retain(|seq| valid.contains(seq));
        // S37 4ADDA0 copies the actor's coordinates into class0 and passes no
        // target list to 44EDE0. Its lifetime is independent after receipt.
        // The target field is provenance, not a following/cancellation owner.
        for cue in &world.effects {
            if cue.graphics == 168
                || (self.profile != Profile::Skills
                    && !self.profile.graphics().contains(&cue.graphics))
                || (self.profile == Profile::Skills
                    && matches!(cue.graphics, 169 | 189 | 194 | 197 | 2231))
            {
                continue;
            }
            if !self.seen.insert(cue.sequence) {
                continue;
            }
            if self.active.len() >= 64 || (self.profile == Profile::Reagent && cue.target.is_some())
            {
                self.omitted += 1;
                continue;
            }
            let Some(clip) = self.clips.get(&cue.graphics) else {
                self.omitted += 1;
                continue;
            };
            self.active.push(Active {
                cue: cue.clone(),
                start: now,
                playback: clip.start(),
            });
            self.started += 1;
        }
        let mut rows = Vec::new();
        let mut events = Vec::new();
        let mut finished = BTreeSet::new();
        {
            for active in &mut self.active {
                let cue = &active.cue;
                let result = (|| -> anyhow::Result<_> {
                    let bank = if super::native_dark_elf_effects::GRAPHICS.contains(&cue.graphics) { self.dark_elf_bank.as_ref() } else if matches!(cue.graphics,11722..=11725) { self.tw_heal_bank.as_ref() } else if self.profile==Profile::Skills && super::native_additional_finite_effects::GRAPHICS.contains(&cue.graphics) {self.additional_bank.as_ref()} else if self.profile==Profile::Skills && super::native_selector_effects::GRAPHICS.contains(&cue.graphics) {self.selector_bank.as_ref()} else { self.bank.as_ref() }
                        .ok_or_else(|| anyhow::anyhow!("effect bank absent"))?;
                    let elapsed = now.saturating_sub(active.start);
                    // Native timers quantize the common clock, not each
                    // effect's fractional birth instant. Keep raw elapsed for
                    // diagnostics and the explicit action-3 presentation.
                    let clock_elapsed_ms = (now / 1_000_000).saturating_sub(active.start / 1_000_000);
                    let sample = active.playback.advance(if self.profile == Profile::Reagent {
                        elapsed
                    } else { clock_elapsed_ms * 1_000_000 })?;
                    for crossed in sample.crossed_cues {
                        if crossed.event_index!=0 {continue;}
                        let animation=bank.animation(u32::from(cue.graphics),self.profile.action())?;
                        let frame=&animation.frames[crossed.source_frame];
                        let classified=super::combat_animation::classify_native_cues(&frame.events)?;
                        anyhow::ensure!(classified.non_audio.is_empty(),"finite non-audio cue unsupported");
                        if let Some(group)=classified.audio {
                            // All source-reviewed finite banks have a single audio candidate.
                            // Weighted producers must supply their reviewed random-word source.
                            anyhow::ensure!(group.candidates().len()==1,"finite weighted RNG provider unavailable");
                            let sound=group.selected(0);
                            let label=if group.mode==super::combat_animation::NativeAudioMode::SuppressWhileSameIdPlaying {
                                format!("EffectSound({})",sound.unwrap_or(0))
                            } else {format!("EffectAudioOverlap({})",sound.unwrap_or(0))};
                            let prepared=super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,
                                super::native_audio_delivery::FiniteEventKey{sequence:cue.sequence,graphics:cue.graphics,action:self.profile.action(),source_frame:crossed.source_frame})?;
                            events.push(EnteredEvent {actor:cue.target.map(|t|t.0).unwrap_or(0),sequence:cue.sequence,
                                action:self.profile.action(),frame:crossed.source_frame,event:label,sound,
                                prepared_audio:Some(prepared)});
                        }
                    }
                    if sample.finished {
                        finished.insert(cue.sequence);
                    }
                    let frame = bank.frame(
                        u32::from(cue.graphics),
                        self.profile.action(),
                        sample.source_frame,
                        sample.finished,
                    )?;
                    // Blank slots keep the parent dwell and audio events, but no pixels.
                    let mut drawn=!sample.finished && !frame.frame.rgba.is_empty();
                    let mut layers=None;
                    let position=WorldCell{x:i32::from(cue.position[0]),y:i32::from(cue.position[1])};
                    if cue.graphics == 2943 {
                        let clothing = bank.frame(2954, 0, sample.source_frame, sample.finished)?;
                        let clothing_drawn = !sample.finished && !clothing.frame.rgba.is_empty();
                        let mut batch = Vec::new();
                        if drawn { batch.push((frame.frame, super::native_world_effects::Mix::Screen)); }
                        if clothing_drawn { batch.push((clothing.frame, super::native_world_effects::Mix::Multiply)); }
                        if !batch.is_empty() { target.submit_batch(&batch, position, projection)?; }
                        layers = Some(serde_json::json!([
                            {"graphics":2943,"sprite":frame.sprite_name,"subframe":frame.subframe,"drawn":drawn},
                            {"graphics":2954,"sprite":clothing.sprite_name,"subframe":clothing.subframe,"drawn":clothing_drawn}
                        ]));
                        drawn |= clothing_drawn;
                    } else if cue.graphics==11725 {
                        let clothing=bank.frame(11726,0,sample.source_frame,sample.finished)?;
                        let clothing_drawn=!sample.finished && !clothing.frame.rgba.is_empty();
                        let mut batch=Vec::new();
                        if drawn {batch.push((frame.frame,super::native_world_effects::Mix::Screen));}
                        if clothing_drawn {batch.push((clothing.frame,super::native_world_effects::Mix::Screen));}
                        // Preflight the complete group, so a budget/fragment failure
                        // cannot submit only its body or only its clothes105 layer.
                        if !batch.is_empty(){target.submit_batch(&batch,position,projection)?;}
                        layers=Some(serde_json::json!([
                            {"graphics":11725,"sprite":frame.sprite_name,"subframe":frame.subframe,"drawn":drawn},
                            {"graphics":11726,"sprite":clothing.sprite_name,"subframe":clothing.subframe,"drawn":clothing_drawn}
                        ]));
                        drawn|=clothing_drawn;
                    } else if drawn {
                        target.submit(frame.frame,position,projection,match cue.graphics {2510=>super::native_world_effects::Mix::Over,2175|2228=>super::native_world_effects::Mix::Multiply,_=>super::native_world_effects::Mix::Screen})?;
                    }
                    let mut row=serde_json::json!({"sequence":cue.sequence,"graphics":cue.graphics,"target":cue.target,"position":cue.position,"action":self.profile.action(),"source_frame":frame.source_frame,"sprite":frame.sprite_name,"subframe":frame.subframe,"drawn":drawn,"finished":sample.finished,"elapsed_ns":elapsed,"native_clock_elapsed_ms":if self.profile==Profile::Reagent {None}else{Some(clock_elapsed_ms)},"next_callback_ms":active.playback.next_callback_millis(),"callback_count":active.playback.callback_count()});
                    if let Some(layers)=layers {row["layers"]=layers;}
                    Ok(row)
                })();
                rows.push(match result { Ok(row)=>row,Err(e)=>{finished.insert(cue.sequence);self.omitted+=1;serde_json::json!({"sequence":cue.sequence,"error":format!("{e:#}"),"drawn":false})} });
            }
        }
        self.active.retain(|a| !finished.contains(&a.cue.sequence));
        let mut receipt = serde_json::json!({"presentation":if self.profile==Profile::Reagent {"modern_explicit_action3_once"} else {"qualified_native_class0"},"started":self.started,"omitted":self.omitted,"active":self.active.len(),"effects":rows,"load_error":self.error.as_ref().or(self.tw_heal_error.as_ref()).or(self.additional_error.as_ref()).or(self.dark_elf_error.as_ref()),"sources":self.bank.as_ref().or(self.tw_heal_bank.as_ref()).map(|_|self.bank.iter().chain(self.tw_heal_bank.iter()).flat_map(|b|b.sources()).map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>()),"entered_events":events,"complete":false});
        if let Some(bank) = &self.dark_elf_bank {
            receipt["dark_elf_bank"] = serde_json::json!({"loaded":true,"load_error":self.dark_elf_error,
                "sources":bank.sources().map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,
                    "sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>()});
        } else if self.dark_elf_error.is_some() {
            receipt["dark_elf_bank"] = serde_json::json!({"loaded":false,"load_error":self.dark_elf_error});
        }
        if let Some(bank)=&self.selector_bank {
            receipt["selector_bank"]=serde_json::json!({"lazy":true,"loaded":true,
                "encoded_bytes":bank.encoded_bytes(),"retained_RGBA_bytes":bank.retained_rgba_bytes(),
                "sound_file_bytes":bank.sound_bank().file_bytes(),"sound_PCM_bytes":bank.sound_bank().pcm_bytes(),
                "load_error":self.selector_error});
        } else if self.selector_error.is_some() {
            receipt["selector_bank"]=serde_json::json!({"lazy":true,"loaded":false,"load_error":self.selector_error});
        }
        if let Some(bank)=&self.additional_bank {
            receipt["additional_bank"]=serde_json::json!({"lazy":true,"loaded":true,
                "graphics":super::native_additional_finite_effects::GRAPHICS,
                "encoded_bytes":bank.encoded_bytes(),"retained_RGBA_bytes":bank.retained_rgba_bytes(),
                "sound_file_bytes":bank.sound_bank().file_bytes(),"sound_PCM_bytes":bank.sound_bank().pcm_bytes(),
                "sources":bank.sources().map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,
                    "sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>(),
                "load_error":self.additional_error});
        } else if self.additional_error.is_some() {
            receipt["additional_bank"]=serde_json::json!({"lazy":true,"loaded":false,"load_error":self.additional_error});
        }
        if let Some(bank) = &self.tw_heal_bank {
            receipt["tw_catalog"] = serde_json::json!({"namespace":"launcher/TW13081901.pak","supported_wire_graphics":[11722,11723,11724,11725],
                "decoded_sha256":bank.catalog_sha256().iter().map(|b|format!("{b:02x}")).collect::<String>(),"load_error":self.tw_heal_error});
        } else if self.tw_heal_error.is_some() {
            receipt["tw_catalog"] = serde_json::json!({"namespace":"launcher/TW13081901.pak","supported_wire_graphics":[11722,11723,11724,11725],"load_error":self.tw_heal_error});
        }
        (receipt, events)
    }
}
