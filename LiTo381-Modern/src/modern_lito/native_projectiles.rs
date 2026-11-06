//! Current class3 S23 cues -> original projectile assets at the actor's Hit entry.
//! Flight is presentation only; it never mutates HP, hit ratios or authority.
use super::{
    compositor::Canvas,
    native_combat_animation::EnteredEvent,
    native_world::World,
    ranged_effect::{RangedAssets, RangedPath},
    world_map::{ViewProjection, WorldCell},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::AtomicBool,
    time::Duration,
};

pub fn supports_graphics(graphics: u16) -> bool {
    // Native S23 dispatches by list.spr effect class, independently of the
    // wire type byte stored in the effect object. The qualified profiles
    // have class 3 and use the same path/frame callback.
    super::ranged_effect::PHYSICAL_GRAPHICS.contains(&u32::from(graphics))
}

struct Flight {
    graphics: u16,
    flight_type: u8,
    attacker: u32,
    attacker_incarnation: u64,
    target: u32,
    target_incarnation: Option<u64>,
    sequence: u64,
    damage_marker: u8,
    id: u32,
    start: u64,
    path: RangedPath,
    callbacks: usize,
    retain_on_arrival: u8,
    name: Vec<u8>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Impact {
    pub target: u32,
    pub sequence: u64,
    pub projectile: u32,
}
impl Flight {
    fn advance_audio(&mut self, assets: &RangedAssets, now: u64, events: &mut Vec<EnteredEvent>) {
        let elapsed = Duration::from_nanos(now.saturating_sub(self.start));
        for (index, sample) in self.path.callbacks(elapsed).skip(self.callbacks) {
            if let Ok(Some(cue)) = assets.audio(sample) {
                events.push(EnteredEvent {
                    actor: self.attacker,
                    sequence: self.sequence,
                    action: 0,
                    frame: index,
                    event: format!("ProjectileSound({})", self.graphics),
                    sound: cue.group().selected(cue.selection()),
                    prepared_audio: Some(cue),
                });
            }
            self.callbacks = index + 1;
        }
    }
    fn valid(&self, world: &World) -> bool {
        world
            .actors
            .get(&self.attacker)
            .is_some_and(|a| a.incarnation == self.attacker_incarnation)
            && (self.target == 0
                || world
                    .actors
                    .get(&self.target)
                    .is_some_and(|a| Some(a.incarnation) == self.target_incarnation))
    }
    fn impact(&self, world: &World, now: u64) -> Option<Impact> {
        let sample = self
            .path
            .sample(Duration::from_nanos(now.saturating_sub(self.start)));
        (sample.finished
            && sample.position == self.path.target()
            && self.damage_marker > 0
            && world.actors.get(&self.target).is_some_and(|a| !a.dead)
            && self.valid(world))
        .then_some(Impact {
            target: self.target,
            sequence: self.sequence,
            projectile: self.id,
        })
    }
}
#[derive(Default)]
pub struct Projectiles {
    generation: Option<u64>,
    seen: BTreeSet<(u32, u64)>,
    flights: Vec<Flight>,
    assets: BTreeMap<u16, RangedAssets>,
    load_errors: BTreeMap<u16, String>,
    launched: u64,
    omitted: u64,
    arrivals: u64,
    events: Vec<EnteredEvent>,
}
impl Projectiles {
    pub fn prepare(&mut self, root: &Path, cancel: &AtomicBool, world: &World) {
        for (stance, graphics) in [(20, 66), (62, 2989)] {
            if world.actors.values().any(|a| a.walk_action == stance)
                || world
                    .projectiles
                    .iter()
                    .any(|c| c.projectile.graphics == graphics && supports_graphics(graphics))
            {
                self.load(root, cancel, graphics);
            }
        }
        for cue in &world.projectiles {
            if supports_graphics(cue.projectile.graphics) {
                self.load(root, cancel, cue.projectile.graphics);
            }
        }
    }
    pub fn entered_events(&self) -> &[EnteredEvent] {
        &self.events
    }

    fn load(&mut self, root: &Path, cancel: &AtomicBool, graphics: u16) {
        if self.assets.contains_key(&graphics) || self.load_errors.contains_key(&graphics) {
            return;
        }
        match RangedAssets::load_graphics(root, u32::from(graphics), cancel) {
            Ok(assets) => {
                self.assets.insert(graphics, assets);
            }
            Err(error) => {
                self.load_errors.insert(graphics, error.to_string());
            }
        }
    }
    pub fn update<'a>(
        &mut self,
        world: &World,
        events: impl Iterator<Item = &'a EnteredEvent>,
        now: u64,
    ) -> Vec<Impact> {
        let mut impacts = Vec::new();
        self.events.clear();
        if self.generation != Some(world.generation) {
            self.generation = Some(world.generation);
            self.seen.clear();
            self.flights.clear();
            self.launched = 0;
            self.omitted = 0;
            self.arrivals = 0;
        }
        let valid: BTreeSet<_> = world
            .projectiles
            .iter()
            .map(|c| (c.attacker, c.action_sequence))
            .collect();
        self.seen.retain(|k| valid.contains(k));
        self.flights.retain_mut(|f| {
            if !f.valid(world) {
                return false;
            }
            if let Some(assets) = self.assets.get(&f.graphics) {
                f.advance_audio(assets, now, &mut self.events);
            }
            if let Some(impact) = f.impact(world, now) {
                impacts.push(impact);
            }
            f.valid(world)
                && !f
                    .path
                    .sample(Duration::from_nanos(now.saturating_sub(f.start)))
                    .finished
        });
        for e in events.filter(|e| e.event == "Hit") {
            let key = (e.actor, e.sequence);
            let Some(cue) = world
                .projectiles
                .iter()
                .find(|c| (c.attacker, c.action_sequence) == key)
            else {
                continue;
            };
            if !self.seen.insert(key) {
                continue;
            }
            if !supports_graphics(cue.projectile.graphics) {
                // Other catalog classes have their own presenters, for every wire type.
                if cue.projectile.flight_type != 0
                    && !super::native_spell_projectiles::supports_graphics(cue.projectile.graphics)
                    && !super::native_class1::supports_graphics(cue.projectile.graphics)
                    && !super::native_disintegrate::supports_graphics(cue.projectile.graphics)
                    && !super::native_cone_cold::supports_graphics(cue.projectile.graphics)
                {
                    self.omitted += 1;
                }
                continue;
            }
            if self.flights.len() >= 64
                || !world
                    .actors
                    .get(&cue.attacker)
                    .is_some_and(|a| a.incarnation == cue.attacker_incarnation)
            {
                self.omitted += 1;
                continue;
            }
            // Native target identity encodes hit/miss, independently of damage.
            let endpoint = if cue.target != 0 && cue.projectile.retain_on_arrival == 0 {
                let Some(a) = world
                    .actors
                    .get(&cue.target)
                    .filter(|a| Some(a.incarnation) == cue.target_incarnation)
                else {
                    self.omitted += 1;
                    continue;
                };
                [a.x, a.y]
            } else {
                cue.projectile.to
            };
            let cell = |p: [u16; 2]| WorldCell {
                x: i32::from(p[0]),
                y: i32::from(p[1]),
            };
            match RangedPath::new(cell(cue.projectile.from), cell(endpoint)) {
                Ok(path) => {
                    self.launched += 1;
                    let mut flight = Flight {
                        graphics: cue.projectile.graphics,
                        flight_type: cue.projectile.flight_type,
                        attacker: cue.attacker,
                        attacker_incarnation: cue.attacker_incarnation,
                        target: cue.target,
                        target_incarnation: cue.target_incarnation,
                        sequence: cue.action_sequence,
                        damage_marker: cue.damage_marker,
                        id: cue.projectile.id,
                        start: now,
                        path,
                        callbacks: 0,
                        retain_on_arrival: cue.projectile.retain_on_arrival,
                        name: cue.projectile.name.clone(),
                    };
                    if let Some(assets) = self.assets.get(&flight.graphics) {
                        flight.advance_audio(assets, now, &mut self.events);
                    }
                    if let Some(impact) = flight.impact(world, now) {
                        impacts.push(impact);
                    }
                    if !flight.path.sample(Duration::ZERO).finished {
                        self.flights.push(flight);
                    }
                }
                Err(_) => self.omitted += 1,
            }
        }
        self.arrivals += impacts.len() as u64;
        impacts
    }
    /// Called only from the bounded background scene worker (or offline tests).
    pub fn draw(
        &mut self,
        root: &Path,
        cancel: &AtomicBool,
        canvas: &mut Canvas,
        projection: ViewProjection,
        now: u64,
    ) -> serde_json::Value {
        self.draw_to(
            root,
            cancel,
            &mut super::native_world_effects::Target::Canvas(canvas),
            projection,
            now,
        )
    }
    pub fn draw_to<'s>(
        &'s mut self,
        root: &Path,
        cancel: &AtomicBool,
        target: &mut super::native_world_effects::Target<'_, 's>,
        projection: ViewProjection,
        now: u64,
    ) -> serde_json::Value {
        let needed: BTreeSet<_> = self.flights.iter().map(|flight| flight.graphics).collect();
        for graphics in needed {
            self.load(root, cancel, graphics);
        }
        let mut rows = Vec::new();
        for f in &self.flights {
            if let Some(assets) = self.assets.get(&f.graphics) {
                let elapsed = Duration::from_nanos(now.saturating_sub(f.start));
                let result = (|| -> anyhow::Result<serde_json::Value> {
                    let sample = assets.effect_frame(&f.path, elapsed)?;
                    let drawn = !sample.finished;
                    assets.submit(target, &f.path, projection, elapsed)?;
                    Ok(
                        serde_json::json!({"id":f.id,"attacker":f.attacker,"target":f.target,"graphics":sample.graphics_id,"flight_type":f.flight_type,"retain_on_arrival":f.retain_on_arrival,"name_bytes":f.name,"shadow":assets.shadow_source().map(|s|&s.name),"declared_secondary":assets.declared_secondary(),"secondary_dispatched":false,"from":[f.path.from().x,f.path.from().y],"to":[f.path.target().x,f.path.target().y],"position":[sample.position.x,sample.position.y],"heading":f.path.heading(),"sprite":sample.sprite_name,"subframe":sample.subframe,"source_frame":sample.source_frame,"drawn":drawn,"elapsed_ns":elapsed.as_nanos()}),
                    )
                })();
                rows.push(match result {
                    Ok(r) => r,
                    Err(e) => serde_json::json!({"id":f.id,"drawn":false,"error":e.to_string()}),
                });
            }
        }
        serde_json::json!({"generation":self.generation,"launched":self.launched,"omitted":self.omitted,"arrivals":self.arrivals,"active":self.flights.len(),"flights":rows,"entered_events":self.events,"load_error":self.load_errors.values().next(),"load_errors":self.load_errors,
            "shadow_sources":self.assets.values().filter_map(|a|a.shadow_source()).map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>(),
            "sources":(!self.assets.is_empty()).then(||self.assets.values().flat_map(|a|a.sprite_sources()).map(|s|serde_json::json!({"archive":format!("{:?}",s.archive),"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>()),
            "graphics_supported":super::ranged_effect::PHYSICAL_GRAPHICS,"native_effects_complete":false})
    }
}
