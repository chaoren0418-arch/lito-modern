//! S53 gfx168: native location constructor starts class0 immediately.
//! Source1..10 then terminal0 including its dwell; no invented server-lease loop.
use super::{
    compositor::Canvas,
    native_combat_animation::EnteredEvent,
    native_skills::Effect,
    native_world::World,
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::{ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog},
    tower_assets::{HonorAssetError, SharedEffectBank},
    world_map::{ViewProjection, WorldCell},
};
use std::{collections::BTreeSet, path::Path, sync::atomic::AtomicBool};

const TICKS: [u64; 11] = [2, 1, 1, 2, 3, 3, 3, 3, 3, 3, 2];
/// The final source0 is visible for two ticks, unlike body one-shots.
pub fn sample(elapsed_ns: u64) -> (usize, bool) {
    super::native_effect_timing::sample_ticks(&TICKS, elapsed_ns)
}
fn qualify(
    catalog: &SpriteActionCatalog,
) -> super::tower_assets::Result<Vec<(u32, u8, Animation)>> {
    let row = catalog
        .sprites
        .get(&168)
        .ok_or(HonorAssetError::Unsupported("fire wall absent"))?;
    let a = catalog.resolve_action(168, ActionKey::new(0))?.animation;
    if row.graphics_id != 168
        || row.header_issue.is_some()
        || row.actions.len() != 2
        || !row.actions.contains_key(&ActionKey::new(3))
        || row.attributes.len() != 2
        || row.attributes.iter().any(|a| {
            !((a.key == ActionKey::new(102) && a.body_bytes.trim_ascii() == b"0")
                || (a.key == ActionKey::new(104) && a.body_bytes.trim_ascii() == b"8"))
        })
        || a.directionality != Directionality::Undirected
        || a.frames_per_second != 24
        || a.frames.len() != 11
        || a.frames.iter().enumerate().any(|(i, f)| {
            f.image_id != 0
                || f.subframe != if i == 0 { 10 } else { i as u32 - 1 }
                || u64::from(f.duration_ticks) != TICKS[i]
                || f.events
                    != if i == 1 {
                        vec![FrameEvent::Sound(111)]
                    } else {
                        vec![]
                    }
        })
    {
        return Err(HonorAssetError::Unsupported(
            "fire wall original action changed",
        ));
    }
    Ok(vec![(168, 0, a.clone())])
}
struct Active {
    cue: Effect,
    start: u64,
}
#[derive(Default)]
pub struct Effects {
    generation: Option<u64>,
    seen: BTreeSet<u64>,
    active: Vec<Active>,
    bank: Option<SharedEffectBank>,
    error: Option<String>,
    started: u64,
    omitted: u64,
}
impl Effects {
    pub fn prepare(&mut self,root:&Path,world:&World,cancel:&AtomicBool) {
        self.prepare_with_catalog(root,world,cancel,&mut None);
    }
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,world:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {
        if self.bank.is_none()
            && self.error.is_none()
            && (world.learned_skills.contains(&58)
                || world.effects.iter().any(|e| e.graphics == 168))
        {
            let loaded = (|| -> anyhow::Result<_> {
                let catalog = SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                Ok(SharedEffectBank::load_with_catalog(
                    &catalog, cancel, qualify,
                )?)
            })();
            match loaded {
                Ok(b) => self.bank = Some(b),
                Err(e) => self.error = Some(format!("{e:#}")),
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
        self.seen
            .retain(|seq| world.effects.iter().any(|e| e.sequence == *seq));
        let mut events = Vec::new();
        for cue in world.effects.iter().filter(|e| e.graphics == 168) {
            if !self.seen.insert(cue.sequence) {
                continue;
            }
            if cue.target.is_some() || self.bank.is_none() || self.active.len() >= 64 {
                self.omitted += 1;
                continue;
            }
            self.active.push(Active {
                cue: cue.clone(),
                start: now,
            });
            self.started += 1;
            events.push(EnteredEvent { prepared_audio:None,
                actor: 0,
                sequence: cue.sequence,
                action: 0,
                frame: 1,
                event: "GroundSound(111)".into(),
                sound: Some(111),
            });
        }
        let mut rows = Vec::new();
        let mut finished = BTreeSet::new();
        if let Some(bank) = &self.bank {
            for active in &self.active {
                let cue = &active.cue;
                let elapsed = now.saturating_sub(active.start);
                let (source, done) = sample(elapsed);
                if done {
                    finished.insert(cue.sequence);
                }
                let result = (|| -> anyhow::Result<_> {
                    let f = bank.frame(168, 0, source, done)?;
                    if !done {
                        target.submit(
                            f.frame,
                            WorldCell {
                                x: i32::from(cue.position[0]),
                                y: i32::from(cue.position[1]),
                            },
                            projection,
                            super::native_world_effects::Mix::Screen,
                        )?;
                    }
                    Ok(
                        serde_json::json!({"sequence":cue.sequence,"graphics":168,"position":cue.position,
                        "source_frame":source,"sprite":f.sprite_name,"subframe":f.subframe,"elapsed_ns":elapsed,
                        "drawn":!done,"finished":done}),
                    )
                })();
                rows.push(match result {Ok(r)=>r,Err(e)=>{finished.insert(cue.sequence);self.omitted+=1;
                    serde_json::json!({"sequence":cue.sequence,"error":format!("{e:#}"),"drawn":false})}});
            }
        }
        self.active.retain(|a| !finished.contains(&a.cue.sequence));
        (
            serde_json::json!({"started":self.started,"active":self.active.len(),"omitted":self.omitted,"load_error":self.error,
            "effects":rows,"entered_events":events,"sources":self.bank.as_ref().map(|b| b.sources().map(|s|
                serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>()),
            "server_lease_rendered":false,"complete":false}),
            events,
        )
    }
}
