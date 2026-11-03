//! Current S23 cues -> original arrow66 resources at the actor's Hit entry.
//! Flight is presentation only; it never mutates HP, hit ratios or authority.
use super::{
    compositor::Canvas,
    native_combat_animation::EnteredEvent,
    native_world::World,
    ranged_effect::{RangedAssets, RangedPath},
    world_map::{ViewProjection, WorldCell},
};
use std::{collections::BTreeSet, path::Path, sync::atomic::AtomicBool, time::Duration};
struct Flight {
    attacker: u32,
    attacker_incarnation: u64,
    target: u32,
    target_incarnation: Option<u64>,
    sequence: u64,
    damage_marker: u8,
    id: u32,
    start: u64,
    path: RangedPath,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Impact {
    pub target: u32,
    pub sequence: u64,
    pub projectile: u32,
}
impl Flight {
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
    assets: Option<RangedAssets>,
    load_error: Option<String>,
    launched: u64,
    omitted: u64,
    arrivals: u64,
}
impl Projectiles {
    pub fn prepare(&mut self, root: &Path, cancel: &AtomicBool, world: &World) {
        if self.assets.is_none()
            && self.load_error.is_none()
            && (world.actors.values().any(|a| a.walk_action == 20)
                || world
                    .projectiles
                    .iter()
                    .any(|c| c.projectile.graphics == 66))
        {
            match RangedAssets::load(root, cancel) {
                Ok(a) => self.assets = Some(a),
                Err(e) => self.load_error = Some(e.to_string()),
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
        self.flights.retain(|f| {
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
            if cue.projectile.flight_type != 0x7f {
                continue;
            }
            if cue.projectile.graphics != 66
                || self.flights.len() >= 64
                || !world
                    .actors
                    .get(&cue.attacker)
                    .is_some_and(|a| a.incarnation == cue.attacker_incarnation)
            {
                self.omitted += 1;
                continue;
            }
            // Native target identity encodes hit/miss, independently of damage.
            let endpoint = if cue.target != 0 {
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
                    let flight = Flight {
                        attacker: cue.attacker,
                        attacker_incarnation: cue.attacker_incarnation,
                        target: cue.target,
                        target_incarnation: cue.target_incarnation,
                        sequence: cue.action_sequence,
                        damage_marker: cue.damage_marker,
                        id: cue.projectile.id,
                        start: now,
                        path,
                    };
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
        if !self.flights.is_empty() && self.assets.is_none() && self.load_error.is_none() {
            match RangedAssets::load(root, cancel) {
                Ok(a) => self.assets = Some(a),
                Err(e) => self.load_error = Some(e.to_string()),
            }
        }
        let mut rows = Vec::new();
        if let Some(assets) = &self.assets {
            for f in &self.flights {
                let elapsed = Duration::from_nanos(now.saturating_sub(f.start));
                let result = (|| -> anyhow::Result<serde_json::Value> {
                    let sample = assets.effect_frame(&f.path, elapsed)?;
                    let drawn = !sample.finished;
                    if drawn {
                        target.submit(
                            sample.frame,
                            sample.position,
                            projection,
                            super::native_world_effects::Mix::Over,
                        )?;
                    }
                    Ok(
                        serde_json::json!({"id":f.id,"attacker":f.attacker,"target":f.target,"graphics":sample.graphics_id,"from":[f.path.from().x,f.path.from().y],"to":[f.path.target().x,f.path.target().y],"position":[sample.position.x,sample.position.y],"heading":f.path.heading(),"sprite":sample.sprite_name,"subframe":sample.subframe,"source_frame":sample.source_frame,"drawn":drawn,"elapsed_ns":elapsed.as_nanos()}),
                    )
                })();
                rows.push(match result {
                    Ok(r) => r,
                    Err(e) => serde_json::json!({"id":f.id,"drawn":false,"error":e.to_string()}),
                });
            }
        }
        serde_json::json!({"generation":self.generation,"launched":self.launched,"omitted":self.omitted,"arrivals":self.arrivals,"active":self.flights.len(),"flights":rows,"load_error":self.load_error,
            "sources":self.assets.as_ref().map(|a|a.sprite_sources().map(|s|serde_json::json!({"archive":format!("{:?}",s.archive),"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>()),
            "graphics_supported":[66],"native_effects_complete":false})
    }
}
