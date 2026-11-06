//! Formal S23 type0 class4 and eight-direction class5 ice1797/1798 at caster Hit entry.
//! Flight coordinates freeze at launch. Class5 secondary anchors freeze at arrival.
//! Bounded lifetime and incarnation fences; no HP/MP mutation.
use super::{
    compositor::Canvas,
    light_arrow_effect::{LightArrowAssets, LightArrowPath, LightArrowPlayback},
    native_combat::ProjectileCue,
    native_combat_animation::EnteredEvent,
    native_projectiles::Impact,
    native_world::World,
    sprite_actions::FrameEvent,
    world_map::{ViewProjection, WorldCell},
};
use std::{collections::{BTreeMap,BTreeSet}, path::Path, sync::atomic::AtomicBool, time::Duration};

#[path="native_eruption.rs"]
mod eruption;
pub fn supports_graphics(graphics:u16)->bool {matches!(graphics,167|1583|1799|1797|756|129)}

#[derive(Clone)]
struct Flight {
    cue: ProjectileCue,
    start: u64,
    path: LightArrowPath,
    playback: LightArrowPlayback,
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
                    c.projectile.flight_type==0 && c.projectile.graphics==graphics)||self.flights.iter().any(|f|f.cue.projectile.graphics==graphics)) {
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
                .find(|c| (c.attacker, c.action_sequence) == key && c.projectile.flight_type == 0 && c.projectile.graphics!=129)
            else {
                continue;
            };
            if !self.seen.insert(key) {
                continue;
            }
            if !supports_graphics(cue.projectile.graphics)
                || self.flights.len()+self.eruption.active_count() >= 64
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
            match LightArrowPath::new(cell(cue.projectile.from), cell(endpoint)) {
                Ok(path) => {
                    let secondary=cue.projectile.graphics!=1797 || cue.target!=0;
                    let playback=assets.start_effect(&path);
                    let playback=if secondary {playback}else{playback.without_secondary()};
                    self.flights.push(Flight {
                        cue: cue.clone(),
                        start: now,
                        path,
                        playback,
                        arrived: false,
                        secondary,
                        impact_position: None,
                    });
                    self.launched += 1;
                }
                Err(_) => self.omitted += 1,
            }
        }
        let mut impacts = Vec::new();
        for flight in &mut self.flights {
            let elapsed = Duration::from_nanos(now.saturating_sub(flight.start));
            if !flight.arrived && elapsed >= flight.path.impact_at() {
                flight.arrived = true;
                self.arrived += 1;
                if flight.cue.projectile.graphics==1797 {
                    // Native42F300 resolves the target at arrival, then places
                    // its finite secondary there. The flight itself never homes.
                    flight.impact_position=world.actors.get(&flight.cue.target)
                        .map(|a|WorldCell{x:i32::from(a.x),y:i32::from(a.y)});
                }
                if flight.cue.projectile.graphics!=756 && flight.cue.damage_marker > 0
                    && world
                        .actors
                        .get(&flight.cue.target)
                        .is_some_and(|a| !a.dead)
                {
                    impacts.push(Impact {
                        target: flight.cue.target,
                        sequence: flight.cue.action_sequence,
                        projectile: flight.cue.projectile.id,
                    });
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
        let mut finished = BTreeSet::new();
        let mut non_audio_cues = Vec::new();
        for f in &mut self.flights {
            if let Some(assets) = self.assets.get(&f.cue.projectile.graphics) {
                let elapsed = Duration::from_nanos(now.saturating_sub(f.start));
                let result = (|| -> anyhow::Result<_> {
                    let sample = f.playback.advance(elapsed)?;
                    for cue in sample.crossed_cues {
                        if let FrameEvent::Sound(sound) = cue.event {
                            events.push(EnteredEvent { prepared_audio:None,
                                actor: f.cue.attacker,
                                sequence: f.cue.action_sequence,
                                action: 18,
                                frame: cue.source_frame,
                                event: format!("SpellSound({sound})"),
                                sound: Some(sound),
                            });
                        } else {
                            non_audio_cues.push(serde_json::json!({"sequence":f.cue.action_sequence,"source_frame":cue.source_frame,"event":format!("{:?}",cue.event),"authority_callback":false}));
                        }
                    }
                    let visual=if f.secondary {assets.effect_frame(&f.path,elapsed)?}else{assets.flight_frame(&f.path,elapsed)?};
                    let drawn = !sample.finished;
                    let position=if f.arrived {f.impact_position.unwrap_or_else(||f.path.position(elapsed))}else{f.path.position(elapsed)};
                    if drawn {
                        target.submit(
                            visual.frame,
                            position,
                            projection,
                            super::native_world_effects::Mix::Screen,
                        )?;
                    }
                    if sample.finished {
                        finished.insert((f.cue.attacker, f.cue.action_sequence));
                    }
                    Ok(
                        serde_json::json!({"sequence":f.cue.action_sequence,"projectile":f.cue.projectile.id,
                        "attacker":f.cue.attacker,"target":f.cue.target,"graphics":visual.graphics_id,
                        "source_frame":visual.source_frame,"sprite":visual.sprite_name,"subframe":visual.subframe,
                        "drawn":drawn,"finished":sample.finished,"arrived":f.arrived,"heading":f.path.heading(),
                        "position":[position.x,position.y],"secondary":f.secondary,
                        "from":[f.path.from().x,f.path.from().y],"to":[f.path.target().x,f.path.target().y],
                        "elapsed_ns":elapsed.as_nanos(),"arrival_ns":f.path.impact_at().as_nanos()}),
                    )
                })();
                rows.push(match result {Ok(r)=>r,Err(e)=>{
                    finished.insert((f.cue.attacker,f.cue.action_sequence));
                    serde_json::json!({"projectile":f.cue.projectile.id,"drawn":false,"error":format!("{e:#}")})
                }});
            }
        }
        self.flights
            .retain(|f| !finished.contains(&(f.cue.attacker, f.cue.action_sequence)));
        (
            serde_json::json!({"generation":self.generation,"launched":self.launched,"arrivals":self.arrived,
            "eruption":eruption_receipt,"omitted":self.omitted,"active":self.flights.len(),"non_audio_cues":non_audio_cues,"load_error":if self.errors.is_empty(){None}else{Some(&self.errors)},"flights":rows,
            "entered_events":events,"sources":self.assets.values().flat_map(|a|a.sprite_sources()).map(|s|
                serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})
            ).collect::<Vec<_>>(),"complete":false}),
            events,
        )
    }
}

#[derive(Clone)]
pub(crate) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,flights:Vec<Flight>,launched:u64,arrived:u64,omitted:u64,eruption:eruption::History}
impl History {
    pub(crate) fn retire(&mut self){self.flights.clear();self.eruption.retire();}
    pub(crate) fn merge(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}self.eruption.merge(&other.eruption);}
    pub(crate) fn oldest(&self)->Option<u64>{self.flights.iter().map(|f|f.cue.action_sequence).chain(self.eruption.oldest()).min()}
}
impl Projectiles {
    pub(crate) fn history(&self)->History{History{generation:self.generation,seen:self.seen.clone(),flights:self.flights.clone(),launched:self.launched,arrived:self.arrived,omitted:self.omitted,eruption:self.eruption.history()}}
    pub(crate) fn restore(&mut self,h:History){self.generation=h.generation;self.seen=h.seen;self.flights=h.flights;self.launched=h.launched;self.arrived=h.arrived;self.omitted=h.omitted;self.eruption.restore(h.eruption);}
}
