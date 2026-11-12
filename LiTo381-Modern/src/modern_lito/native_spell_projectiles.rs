//! S23 class4 and eight-direction class5 ice1797/1798 at caster Hit entry.
//! Native 4A5ED0 dispatches by catalog class, retaining the wire type separately.
//! Flight coordinates freeze at launch. Class5 secondary anchors freeze at arrival.
//! Bounded lifetime and incarnation fences; no HP/MP mutation.
//!
//! Timing is the native movement callback (`NativeSpellClock`): first step
//! immediately, then one cell per 60ms callback rescheduled from the actual
//! dispatch; the flight source frame cycles once per movement. Arrival is the
//! movement callback that reaches the frozen endpoint, never the ideal
//! `path.impact_at()` wall time. `update` owns the clock; `draw_to` only reads
//! the last sample and hands over the cues entered since the previous draw.
use super::{
    compositor::Canvas,
    light_arrow_effect::{LightArrowAssets, LightArrowPath, NativeSpellClock, NativeSpellSample},
    native_combat::ProjectileCue,
    native_combat_animation::EnteredEvent,
    native_effect_timing::native_elapsed_ns,
    native_projectiles::Impact,
    native_world::World,
    sprite_actions::FrameEvent,
    world_map::{ViewProjection, WorldCell},
};
use std::{collections::{BTreeMap,BTreeSet}, path::Path, sync::atomic::AtomicBool};

#[path="native_eruption.rs"]
mod eruption;
pub fn supports_graphics(graphics:u16)->bool {matches!(graphics,167|1583|1799|1797|756|129)}

/// Cues wait here between `update` and the next `draw_to`. 64 flights cannot
/// exceed this in one frame; a caller that never draws must not grow it.
const MAX_PENDING_CUES: usize = 256;

#[derive(Clone)]
struct Flight {
    cue: ProjectileCue,
    start: u64,
    path: LightArrowPath,
    clock: NativeSpellClock,
    /// Last dispatched clock sample with its cues already handed over.
    sample: Option<NativeSpellSample>,
    finished: bool,
    arrived: bool,
    secondary: bool,
    impact_position: Option<WorldCell>,
}
fn valid(world: &World, cue: &ProjectileCue) -> bool {
    world
        .actors
        .get(&cue.attacker)
        .is_some_and(|a| a.incarnation == cue.attacker_incarnation)
        && (cue.target == 0
            || world
                .actors
                .get(&cue.target)
                .is_some_and(|a| Some(a.incarnation) == cue.target_incarnation))
}
#[derive(Default)]
pub struct Projectiles {
    eruption:eruption::Effects,
    generation: Option<u64>,
    assets: BTreeMap<u16,LightArrowAssets>,
    errors: BTreeMap<u16,String>,
    seen: BTreeSet<(u32, u64)>,
    flights: Vec<Flight>,
    launched: u64,
    arrived: u64,
    omitted: u64,
    events: Vec<EnteredEvent>,
    non_audio: Vec<serde_json::Value>,
    clock_errors: Vec<serde_json::Value>,
}
impl Projectiles {
    pub fn prepare(&mut self,root:&Path,world:&World,cancel:&AtomicBool) {
        self.prepare_with_catalog(root,world,cancel,&mut None);
    }
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,world:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {
        self.eruption.prepare(root,world,cancel,shared);
        let mut sprite_reader=None;
        for (skill,graphics) in [(4,167),(6,1797),(7,1799),(15,1583),(50,756)] {
            if !self.assets.contains_key(&graphics) && !self.errors.contains_key(&graphics)
                && (world.learned_skills.contains(&skill)||world.projectiles.iter().any(|c|
                    c.projectile.graphics==graphics)||self.flights.iter().any(|f|f.cue.projectile.graphics==graphics)) {
                let loaded=(|| {
                    let context=super::shared_action_catalog::SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                    LightArrowAssets::load_effect_profile_with_reader(context,u32::from(graphics),cancel,&mut sprite_reader)
                })();
                match loaded {
                    Ok(a)=>{self.assets.insert(graphics,a);},
                    Err(e)=>{self.errors.insert(graphics,e.to_string());},
                }
            }
        }
    }
    pub fn update<'a>(
        &mut self,
        world: &World,
        events: impl Iterator<Item = &'a EnteredEvent>,
        now: u64,
    ) -> Vec<Impact> {
        let events:Vec<_>=events.collect();
        if self.generation != Some(world.generation) {
            self.generation = Some(world.generation);
            self.seen.clear();
            self.flights.clear();
            self.events.clear();
            self.non_audio.clear();
            self.clock_errors.clear();
            self.launched = 0;
            self.arrived = 0;
            self.omitted = 0;
        }
        self.eruption.update(world,&events,now,self.flights.len());
        let live: BTreeSet<_> = world
            .projectiles
            .iter()
            .map(|c| (c.attacker, c.action_sequence))
            .collect();
        self.seen.retain(|key| live.contains(key)||self.flights.iter().any(|f|(f.cue.attacker,f.cue.action_sequence)==*key));
        let before=self.flights.len();
        self.flights.retain(|f|valid(world,&f.cue)&&!self.errors.contains_key(&f.cue.projectile.graphics));
        self.omitted+=(before-self.flights.len()) as u64;
        for event in events.into_iter().filter(|e| e.event == "Hit") {
            let key = (event.actor, event.sequence);
            let Some(cue) = world
                .projectiles
                .iter()
                .find(|c| (c.attacker, c.action_sequence) == key
                    && supports_graphics(c.projectile.graphics) && c.projectile.graphics!=129)
            else {
                continue;
            };
            if !self.seen.insert(key) {
                continue;
            }
            if self.flights.len()+self.eruption.active_count() >= 64
                || !valid(world, cue)
                || world.actors.get(&cue.attacker).is_none_or(|a| a.dead)
            {
                self.omitted += 1;
                continue;
            }
            let Some(assets) = self.assets.get(&cue.projectile.graphics) else {
                self.omitted += 1;
                continue;
            };
            let endpoint = if cue.target == 0 {
                cue.projectile.to
            } else {
                let a = &world.actors[&cue.target];
                [a.x, a.y]
            };
            let cell = |p: [u16; 2]| WorldCell {
                x: i32::from(p[0]),
                y: i32::from(p[1]),
            };
            // Native 42F300 constructs a secondary only for a resolved target;
            // class4 always constructs one at its flight endpoint.
            let secondary=cue.projectile.graphics!=1797 || cue.target!=0;
            let launched=LightArrowPath::new(cell(cue.projectile.from), cell(endpoint)).ok()
                .and_then(|path|assets.start_native_s23(&path,secondary).ok().map(|clock|(path,clock)));
            match launched {
                Some((path,clock)) => {
                    self.flights.push(Flight {
                        cue: cue.clone(),
                        start: now,
                        path,
                        clock,
                        sample: None,
                        finished: false,
                        arrived: false,
                        secondary,
                        impact_position: None,
                    });
                    self.launched += 1;
                }
                None => self.omitted += 1,
            }
        }
        let mut impacts = Vec::new();
        let Self{flights,assets,arrived,omitted,events:entered,non_audio,clock_errors,..}=self;
        for flight in flights.iter_mut() {
            if flight.finished {continue;}
            let Some(assets)=assets.get(&flight.cue.projectile.graphics) else {continue;};
            let graphics=flight.cue.projectile.graphics;
            let result=(||->anyhow::Result<()> {
                // Global integer-millisecond origin: floor both endpoints first.
                let mut sample=flight.clock.advance(native_elapsed_ns(now,flight.start))?;
                if sample.arrived_now {
                    flight.arrived=true;
                    *arrived+=1;
                    if graphics==1797 {
                        // Native42F300 resolves the target at arrival, then places
                        // its finite secondary there. The flight itself never homes.
                        flight.impact_position=world.actors.get(&flight.cue.target)
                            .map(|a|WorldCell{x:i32::from(a.x),y:i32::from(a.y)});
                    }
                    if graphics!=756 && flight.cue.damage_marker>0
                        && world.actors.get(&flight.cue.target).is_some_and(|a|!a.dead) {
                        impacts.push(Impact{target:flight.cue.target,sequence:flight.cue.action_sequence,projectile:flight.cue.projectile.id});
                    }
                }
                for cue in std::mem::take(&mut sample.cues) {
                    if let FrameEvent::Sound(sound)=cue.event {
                        anyhow::ensure!(assets.native_s23_events(cue.graphics,cue.source_frame)?.contains(&FrameEvent::Sound(sound)),"native S23 cue is not in its source frame");
                        anyhow::ensure!(entered.len()<MAX_PENDING_CUES,"native S23 pending cue budget");
                        let group=super::combat_animation::classify_native_cues(&[FrameEvent::Sound(sound)])?
                            .audio.ok_or_else(||anyhow::anyhow!("native S23 audio group absent"))?;
                        // The callback occurrence keeps a re-entered source1 cue
                        // distinct from the same cast's earlier movement.
                        let payload=super::native_audio_delivery::PreparedCue::from_callback_event(assets.sound_bank(),group,0,
                            super::native_audio_delivery::FiniteEventKey{sequence:flight.cue.action_sequence,graphics:cue.graphics,action:0,source_frame:cue.source_frame},cue.occurrence)?;
                        entered.push(EnteredEvent{prepared_audio:Some(payload),actor:flight.cue.attacker,sequence:flight.cue.action_sequence,
                            action:18,frame:cue.source_frame,event:format!("SpellSound({sound})"),sound:Some(sound)});
                    } else if non_audio.len()<MAX_PENDING_CUES {
                        // A secondary Hit is a presentation receipt only; the
                        // arrival above already released the one hurt.
                        non_audio.push(serde_json::json!({"sequence":flight.cue.action_sequence,"graphics":cue.graphics,"source_frame":cue.source_frame,"event":format!("{:?}",cue.event),"authority_callback":false}));
                    }
                }
                flight.finished=sample.finished;
                flight.sample=Some(sample);
                Ok(())
            })();
            if let Err(error)=result {
                flight.finished=true;
                *omitted+=1;
                if clock_errors.len()<64 {
                    clock_errors.push(serde_json::json!({"projectile":flight.cue.projectile.id,"error":format!("{error:#}")}));
                }
            }
        }
        impacts
    }
    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        now: u64,
    ) -> (serde_json::Value, Vec<EnteredEvent>) {
        self.draw_to(
            &mut super::native_world_effects::Target::Canvas(canvas),
            projection,
            now,
        )
    }
    pub fn draw_to<'s>(
        &'s mut self,
        target: &mut super::native_world_effects::Target<'_, 's>,
        projection: ViewProjection,
        now: u64,
    ) -> (serde_json::Value, Vec<EnteredEvent>) {
        let (eruption_receipt,eruption_events)=self.eruption.draw(target,projection);
        let mut rows = Vec::new();
        let mut events = eruption_events;
        // Each entered cue leaves exactly once, with the first draw after it.
        events.append(&mut self.events);
        let non_audio_cues=std::mem::take(&mut self.non_audio);
        let clock_errors=std::mem::take(&mut self.clock_errors);
        for f in &self.flights {
            let (Some(assets),Some(sample))=(self.assets.get(&f.cue.projectile.graphics),f.sample.as_ref()) else {continue;};
            let result = (|| -> anyhow::Result<_> {
                // The secondary is undirected; the flight uses the sampled bank.
                let secondary=sample.arrived&&f.secondary;
                let visual=assets.native_s23_frame(sample.graphics,if secondary{0}else{sample.heading},sample.source_frame,sample.finished)?;
                let drawn=!f.finished&&!sample.finished;
                // Class4 explodes at the frozen flight endpoint; class5 at the
                // target cell resolved by the arrival callback.
                let position=if sample.arrived {f.impact_position.unwrap_or(sample.position)}else{sample.position};
                if drawn {
                    target.submit(
                        visual.frame,
                        position,
                        projection,
                        super::native_world_effects::Mix::Screen,
                    )?;
                }
                Ok(
                    serde_json::json!({"sequence":f.cue.action_sequence,"projectile":f.cue.projectile.id,
                    "attacker":f.cue.attacker,"target":f.cue.target,"graphics":visual.graphics_id,
                    "flight_type":f.cue.projectile.flight_type,
                    "source_frame":sample.source_frame,"sprite":visual.sprite_name,"subframe":visual.subframe,
                    "drawn":drawn,"finished":f.finished||sample.finished,"arrived":sample.arrived,"heading":f.path.heading(),
                    "bank_heading":sample.heading,
                    "position":[position.x,position.y],"secondary":f.secondary,
                    "from":[f.path.from().x,f.path.from().y],"to":[f.path.target().x,f.path.target().y],
                    "elapsed_ns":now.saturating_sub(f.start),
                    "arrival_ns":sample.arrival_ns,"arrival_clock_unit":"native_ms_relative_ns",
                    "native_clock_elapsed_ms":native_elapsed_ns(now,f.start)/1_000_000,
                    "native_clock_origin_ms":f.start/1_000_000,
                    "movement_callbacks":sample.callbacks,"next_movement_ms":sample.next_movement_ms}),
                )
            })();
            rows.push(match result {Ok(r)=>r,Err(e)=>{
                serde_json::json!({"projectile":f.cue.projectile.id,"drawn":false,"error":format!("{e:#}")})
            }});
        }
        self.flights.retain(|f| !f.finished);
        (
            serde_json::json!({"generation":self.generation,"launched":self.launched,"arrivals":self.arrived,
            "eruption":eruption_receipt,"omitted":self.omitted,"active":self.flights.len(),"non_audio_cues":non_audio_cues,
            "clock_errors":clock_errors,"load_error":if self.errors.is_empty(){None}else{Some(&self.errors)},"flights":rows,
            "entered_events":events,"sources":self.assets.values().flat_map(|a|a.sprite_sources()).map(|s|
                serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})
            ).collect::<Vec<_>>(),"complete":false}),
            events,
        )
    }
}

/// Metadata only: admission, frozen path and clock cursor; never SPR/PCM/files.
#[derive(Clone)]
pub(crate) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,flights:Vec<Flight>,launched:u64,arrived:u64,omitted:u64,eruption:eruption::History}
impl History {
    pub(crate) fn retire(&mut self){self.flights.clear();self.eruption.retire();}
    pub(crate) fn merge(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}self.eruption.merge(&other.eruption);}
    pub(crate) fn oldest(&self)->Option<u64>{self.flights.iter().map(|f|f.cue.action_sequence).chain(self.eruption.oldest()).min()}
    /// Casts whose re-entrant callback audio keys must outlive the raw World
    /// ring: a flight still cycling may re-enter source1 after ring eviction.
    pub(crate) fn active_sequences(&self)->BTreeSet<u64>{self.flights.iter().filter(|f|!f.finished).map(|f|f.cue.action_sequence).collect()}
}
impl Projectiles {
    pub(crate) fn history(&self)->History{History{generation:self.generation,seen:self.seen.clone(),flights:self.flights.clone(),launched:self.launched,arrived:self.arrived,omitted:self.omitted,eruption:self.eruption.history()}}
    pub(crate) fn restore(&mut self,h:History){self.generation=h.generation;self.seen=h.seen;self.flights=h.flights;self.launched=h.launched;self.arrived=h.arrived;self.omitted=h.omitted;self.eruption.restore(h.eruption);}
}

#[cfg(test)]
mod native_s23_history_tests {
    use super::*;
    fn empty(generation:u64)->History {
        History{generation:Some(generation),seen:BTreeSet::new(),flights:Vec::new(),launched:0,arrived:0,omitted:0,eruption:Projectiles::default().history().eruption}
    }
    #[test]
    fn metadata_roundtrip_keeps_admission_and_counters() {
        let mut h=empty(7);h.seen.insert((0xf1234567,91));h.launched=5;h.arrived=3;h.omitted=2;
        let mut p=Projectiles::default();p.restore(h);let saved=p.history();
        assert_eq!(saved.generation,Some(7));assert!(saved.seen.contains(&(0xf1234567,91)));
        assert_eq!((saved.launched,saved.arrived,saved.omitted),(5,3,2));
        assert!(saved.flights.is_empty()&&saved.active_sequences().is_empty()&&saved.oldest().is_none());
    }
    #[test]
    fn admission_merges_only_within_the_same_world_generation_and_retire_keeps_seen() {
        let mut h=empty(7);h.seen.insert((1,10));
        let mut peer=empty(7);peer.seen.insert((2,20));h.merge(&peer);
        assert_eq!(h.seen,BTreeSet::from([(1,10),(2,20)]));
        let mut other=empty(8);other.seen.insert((3,30));h.merge(&other);
        assert!(!h.seen.contains(&(3,30)));
        h.retire();assert_eq!(h.seen.len(),2);assert!(h.flights.is_empty());
    }
    #[test]
    fn a_world_generation_change_drops_pending_cues_with_the_flights() {
        let mut p=Projectiles::default();
        p.generation=Some(1);
        p.non_audio.push(serde_json::json!({"event":"Hit"}));
        p.clock_errors.push(serde_json::json!({"error":"x"}));
        p.launched=3;
        let mut world=World::default();
        world.generation=2;
        assert!(p.update(&world,std::iter::empty(),0).is_empty());
        assert!(p.non_audio.is_empty()&&p.clock_errors.is_empty()&&p.events.is_empty());
        assert_eq!((p.generation,p.launched),(Some(2),0));
    }
}
