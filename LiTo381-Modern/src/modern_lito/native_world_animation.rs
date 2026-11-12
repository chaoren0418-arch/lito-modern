//! Resource-clock presentation. World coordinates remain network/prediction state.
use super::{
    native_world::Actor,
    sprite_actions::{ActionDefinition, ActionKey, Animation, Directionality, SpriteActionCatalog},
};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, hash::BuildHasher};
pub struct Clip {
    pub graphics: u32,
    pub animation: Animation,
    total: u64,
}
impl Clip {
    pub fn new(graphics: u32, animation: Animation) -> Result<Self> {
        ensure!(
            !animation.frames.is_empty() && animation.frames.len() <= 16384,
            "animation frame budget"
        );
        ensure!(
            (1..=1000).contains(&animation.frames_per_second),
            "animation FPS"
        );
        ensure!(
            !animation.has_uninterpreted_events(),
            "animation has uninterpreted events"
        );
        let total = animation.total_ticks();
        ensure!(total > 0, "zero animation duration");
        ensure!(
            u128::from(total) * 1_000_000_000 / u128::from(animation.frames_per_second)
                < u128::from(u64::MAX),
            "animation duration overflow"
        );
        Ok(Self {
            graphics,
            animation,
            total,
        })
    }
    pub fn duration_ns(&self) -> u64 {
        ((u128::from(self.total) * 1_000_000_000)
            .div_ceil(u128::from(self.animation.frames_per_second))) as u64
    }
    pub fn sample(&self, elapsed_ns: u64, repeat: bool) -> usize {
        let units =
            u128::from(elapsed_ns) * u128::from(self.animation.frames_per_second) / 1_000_000_000;
        let tick = if repeat {
            units % u128::from(self.total)
        } else {
            units.min(u128::from(self.total - 1))
        } as u64;
        let mut end = 0;
        for (index, frame) in self.animation.frames.iter().enumerate() {
            end += u64::from(frame.duration_ticks);
            if tick < end {
                return index;
            }
        }
        unreachable!("validated positive duration")
    }
}
pub fn resolve(catalog: &SpriteActionCatalog, id: u32, action: u8) -> Result<Clip> {
    // Formal guardian healthy action is zero. Original type11 obelisks have
    // no walk/breath action: their intact source image is stateone, frame zero.
    if action==0 && matches!(id,2665|2667|2669|2671) {
        let clip=resolve(catalog,id,32)?;
        ensure!(clip.animation.frames.len()==1 && clip.animation.frames[0].image_id==0
            && clip.animation.frames[0].subframe==0,"guardian intact source frame");
        return Ok(clip);
    }
    let requested = catalog
        .sprites
        .get(&id)
        .context("sprite definition absent")?;
    ensure!(requested.header_issue.is_none(), "unverified sprite header");
    let graphics = requested.graphics_id;
    let mut current = id;
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..32 {
        ensure!(seen.insert(current), "sprite reference cycle");
        let row = catalog
            .sprites
            .get(&current)
            .context("sprite reference absent")?;
        ensure!(
            row.header_issue.is_none() && row.graphics_id == graphics,
            "sprite alias graphics mismatch"
        );
        match row
            .actions
            .get(&ActionKey::new(u32::from(action)))
            .with_context(|| format!("sprite {id} action {action} absent"))?
        {
            ActionDefinition::Animation(animation) => {
                return Clip::new(graphics, animation.clone())
            }
            ActionDefinition::Reference { sprite_id, .. } => current = *sprite_id,
            ActionDefinition::Quarantined { .. } => anyhow::bail!("unverified animation syntax"),
        }
    }
    anyhow::bail!("sprite reference depth")
}
#[derive(Debug, serde::Serialize)]
pub struct Pose {
    pub action_complete: bool,
    pub entered_events: Vec<super::native_combat_animation::EnteredEvent>,
    pub graphics: u32,
    pub image: u32,
    pub subframe: u32,
    pub source_frame: usize,
    pub action: u8,
    pub phase: &'static str,
    pub position: [f64; 2],
    pub events: Vec<String>,
    pub static_prop: bool,
}
#[derive(Clone)]
struct Track {
    signature: (u64, u64, u16, u16, u16, u8),
    last: u64,
    virtual_units: u128,
    speed: u8,
    brave: u8,
    from: Option<[u16; 2]>,
    object_emitted: usize,
}
#[derive(Clone,Default)]
pub(crate) struct Admission { actors:BTreeMap<u32,(u64,u64,u64)> }
impl Admission {
    pub(crate) fn admit(&mut self,world:&super::native_world::World) {
        self.actors.retain(|id,_|world.actors.contains_key(id));
        for actor in world.actors.values() {self.actors.insert(actor.id,(actor.incarnation,actor.action_sequence,actor.motion_revision));}
    }
}
#[derive(Clone)]
pub(crate) struct History {combat:super::native_combat_animation::History,tracks:BTreeMap<u32,Track>,last_ns:Option<u64>,retired:Admission}
#[derive(Default)]
pub struct Animator {
    combat: super::native_combat_animation::Playback,
    clips: BTreeMap<(u32, u8), Clip>,
    tracks: BTreeMap<u32, Track>,
    last_ns: Option<u64>,
    retired:Admission,
}
impl Animator {
    pub(crate) fn history(&self)->History {History{combat:self.combat.history(),tracks:self.tracks.clone(),last_ns:self.last_ns,retired:self.retired.clone()}}
    pub(crate) fn restore(&mut self,history:History){self.combat.restore(history.combat);self.tracks=history.tracks;self.last_ns=history.last_ns;self.retired=history.retired;self.clips.clear();}
    pub(crate) fn retire_admitted(&mut self,admission:Admission,world:&super::native_world::World,now:u64){
        for actor in world.actors.values(){
            if let Some(&(incarnation,sequence,motion))=admission.actors.get(&actor.id).filter(|v|v.0==actor.incarnation){
                self.combat.admit(actor,sequence,now);
                if super::native_obstacles::guardian(actor) && !self.tracks.contains_key(&actor.id) && motion==actor.motion_revision {
                    self.tracks.insert(actor.id,Track{signature:(incarnation,motion,actor.graphics,actor.x,actor.y,actor.walk_action),last:now,virtual_units:0,speed:actor.speed,brave:actor.brave,from:actor.motion_from,object_emitted:usize::MAX});
                }
            }
        }
        self.retired=admission;
    }

    pub fn impact(&mut self, actor: &Actor, sequence: u64) -> bool {
        self.combat.impact(actor, sequence)
    }
    /// Whether the published playback has started every action up to `barrier`.
    pub(crate) fn presented(&self, actor: &Actor, barrier: u64) -> bool {
        self.combat.presented(actor, barrier)
    }
    pub fn begin(&mut self, now: u64, ids: impl Iterator<Item = u32>) -> Result<()> {
        ensure!(
            self.last_ns.is_none_or(|old| now >= old),
            "animation time went backwards"
        );
        self.last_ns = Some(now);
        let ids: std::collections::BTreeSet<_> = ids.collect();
        self.tracks.retain(|id, _| ids.contains(id));
        self.combat.retain(&ids);
        Ok(())
    }
    fn clip(&mut self, catalog: &SpriteActionCatalog, id: u32, action: u8) -> Result<&Clip> {
        if !self.clips.contains_key(&(id, action)) {
            ensure!(self.clips.len() < 256, "animation clip budget");
            self.clips
                .insert((id, action), resolve(catalog, id, action)?);
        }
        Ok(&self.clips[&(id, action)])
    }
    /// Keep received action admission behind an explicitly owned source walk.
    /// When it ends, the ordinary combat player must not restart the same walk
    /// from elapsed zero just because its images were supplied by callbacks.
    pub(crate) fn source_motion(&mut self, catalog:&SpriteActionCatalog, actor:&Actor,
        now:u64, complete:bool)->Result<()> {
        let duration=self.clip(catalog,u32::from(actor.graphics),actor.walk_action)?.duration_ns();
        let signature=(actor.incarnation,actor.motion_revision,actor.graphics,actor.x,actor.y,actor.walk_action);
        if complete && self.tracks.get(&actor.id).is_some_and(|t|t.signature==signature
            && t.virtual_units>=u128::from(duration)*126) {return Ok(());}
        self.tracks.insert(actor.id,Track {
            signature,
            last:now,virtual_units:if complete {u128::from(duration)*126} else {0},
            speed:actor.speed,brave:actor.brave,from:actor.motion_from,object_emitted:0,
        });
        if !complete {self.combat.hold_for_motion(actor,now)?;}
        Ok(())
    }
    pub fn pose(&mut self, catalog: &SpriteActionCatalog, actor: &Actor, now: u64) -> Result<Pose> {
        let id = u32::from(actor.graphics);
        let row = catalog
            .sprites
            .get(&id)
            .context("sprite definition absent")?;
        // Explicit one-image type9 ground item, type14 board, or type12 static NPC.
        // The renderer also verifies that the source SPR has exactly one frame.
        let ground_item = row
            .attributes
            .iter()
            .any(|a| a.key == ActionKey::new(102) && a.body_bytes.trim_ascii() == b"9");
        if row.header_issue.is_none()
            && row.image_count == Some(1)
            && row.actions.is_empty()
            && row.attributes.iter().any(|a| {
                a.key == ActionKey::new(102) && matches!(a.body_bytes.trim_ascii(), b"9" | b"12" | b"14")
            })
        {
            ensure!(actor.walk_action == 0 && actor.motion_from.is_none(), "static prop movement/action");
            return Ok(Pose {
                action_complete: true,
                graphics: row.graphics_id,
                image: 0,
                subframe: 0,
                source_frame: 0,
                action: 0,
                phase: if ground_item {
                    "ground_item"
                } else {
                    "static_prop"
                },
                position: [f64::from(actor.x), f64::from(actor.y)],
                events: vec![],
                static_prop: true,
                entered_events: vec![],
            });
        }
        let guardian = super::native_obstacles::guardian(actor);
        let cube = super::native_cube::is_graphics(id);
        if cube { super::native_cube::qualify(catalog, id)?; }
        let signature = (
            actor.incarnation,
            actor.motion_revision,
            actor.graphics,
            actor.x,
            actor.y,
            actor.walk_action,
        );
        let stance = actor.walk_action;
        let base_walk = if actor.motion_from.is_some() {
            self.clip(catalog, id, stance)?.duration_ns()
        } else {
            0
        };
        let track = self.tracks.entry(actor.id).or_insert(Track {
            signature,
            last: now,
            virtual_units: 0,
            speed: actor.speed,
            brave: actor.brave,
            from: actor.motion_from,
            object_emitted: 0,
        });
        if track.signature != signature {
            *track = Track {
                signature,
                last: now,
                virtual_units: 0,
                speed: actor.speed,
                brave: actor.brave,
                from: actor.motion_from,
                object_emitted: 0,
            };
        }
        // 126 is divisible by every denominator of the verified rate factors.
        // Keep fractional nanoseconds between samples rather than truncating each tick.
        let mut wall_units = u128::from(
            now.checked_sub(track.last)
                .context("actor clock went backwards")?,
        ) * 126;
        let base_units = u128::from(base_walk) * 126;
        if track.from.is_some() && track.virtual_units < base_units {
            let (n, d) = super::native_speed::factor(track.speed, track.brave, stance)?;
            let needed =
                ((base_units - track.virtual_units) * u128::from(n)).div_ceil(u128::from(d));
            if wall_units >= needed {
                track.virtual_units = base_units;
                wall_units -= needed;
            } else {
                track.virtual_units += wall_units * u128::from(d) / u128::from(n);
                wall_units = 0;
            }
        }
        let idle_action = if guardian {stance} else if matches!(
            stance,
            0 | 4 | 11 | 20 | 24 | 40 | 46 | 50 | 54 | 58 | 62 | 83
        ) {
            stance + 3
        } else {
            stance
        };
        let (n, d) = super::native_speed::factor(track.speed, track.brave, idle_action)?;
        track.virtual_units = track
            .virtual_units
            .checked_add(wall_units * u128::from(d) / u128::from(n))
            .context("animation elapsed overflow")?;
        track.last = now;
        track.speed = actor.speed;
        track.brave = actor.brave;
        let elapsed =
            u64::try_from(track.virtual_units / 126).context("animation elapsed overflow")?;
        let from = track.from;
        let object_emitted = track.object_emitted;
        let moving = from.is_some();
        let walk_duration = base_walk;
        // Original44E6B70 recognizes a running walk and44ED2B enqueues
        // hurt through4E63C0. Do not snap to the destination for hurt and then
        // resume an older partial walk. The motion/camera clock remains live.
        let (combat_pose, mut entered_events)=if guardian {
            (None,Vec::new())
        } else if moving && elapsed<walk_duration && !actor.dead {
            self.combat.hold_for_motion(actor,now)?;(None,Vec::new())
        } else {self.combat.sample(catalog,actor,now)?};
        if let Some(&(_,floor,_))=self.retired.actors.get(&actor.id).filter(|v|v.0==actor.incarnation) {entered_events.retain(|event|event.sequence>floor);}
        let (action, phase, time, repeat) = if let Some(combat) = combat_pose {
            (combat.action, combat.phase, combat.elapsed_ns, false)
        } else if moving && elapsed < walk_duration {
            (stance, "walk", elapsed, false)
        } else if cube {
            (3, "cube", elapsed, true)
        } else if guardian {
            (stance, "object_action", elapsed, false)
        } else if matches!(
            stance,
            0 | 4 | 11 | 20 | 24 | 40 | 46 | 50 | 54 | 58 | 62 | 83
        ) {
            (
                stance + 3,
                "idle",
                elapsed.saturating_sub(walk_duration),
                true,
            )
        } else {
            (stance, "object_action", elapsed, false)
        };
        let retired_motion=self.retired.actors.get(&actor.id).filter(|v|v.0==actor.incarnation).map(|v|v.2);
        let clip = self.clip(catalog, id, action)?;
        if phase == "idle" {
            let label = String::from_utf8_lossy(&clip.animation.label_bytes).to_ascii_lowercase();
            ensure!(
                label.starts_with("breath") || label.starts_with("idle"),
                "unverified idle action family"
            );
        }
        let source_frame = clip.sample(time, repeat);
        let frame = &clip.animation.frames[source_frame];
        let cube_loop = cube && phase == "cube";
        let object_end = if cube_loop {
            let cycles = u128::from(time) * u128::from(clip.animation.frames_per_second)
                / 1_000_000_000 / u128::from(clip.total);
            usize::try_from(cycles)?.checked_mul(clip.animation.frames.len())
                .and_then(|count| count.checked_add(source_frame + 1))
                .context("cube frame counter overflow")?
        } else { source_frame + 1 };
        if guardian || cube_loop {
            // A delayed render emits at most the latest cube cycle. The
            // source events remain presentation-only, including its Hit marks.
            let first = if cube_loop { object_emitted.max(object_end.saturating_sub(clip.animation.frames.len())) }
                else { object_emitted };
            for ordinal in first..object_end {
                let index = ordinal % clip.animation.frames.len();
                let frame_events = &clip.animation.frames[index].events;
                let selection = std::collections::hash_map::RandomState::new()
                    .hash_one((actor.id, actor.motion_revision, ordinal)) as u32;
                for (event_index, event) in frame_events.iter().enumerate() {
                    ensure!(entered_events.len()<256,"object frame event budget");
                    entered_events.push(super::native_combat_animation::EnteredEvent { prepared_audio:None,
                        actor:actor.id,sequence:actor.motion_revision,action,frame:index,
                        event:format!("{event:?}"),sound:super::native_combat_animation::frame_sound(frame_events,event_index,selection),
                    });
                }
            }
        }
        if guardian {if let Some(motion)=retired_motion {entered_events.retain(|event|event.sequence>motion);}}
        let direction = if clip.animation.directionality == Directionality::EightWay {
            u32::from(actor.heading)
        } else {
            0
        };
        let position = if phase == "walk" {
            let p = from.unwrap();
            let ratio = (elapsed as f64 / walk_duration as f64).clamp(0., 1.);
            [
                f64::from(p[0]) + (f64::from(actor.x) - f64::from(p[0])) * ratio,
                f64::from(p[1]) + (f64::from(actor.y) - f64::from(p[1])) * ratio,
            ]
        } else {
            [f64::from(actor.x), f64::from(actor.y)]
        };
        let pose = Pose {
            action_complete: !repeat && time >= clip.duration_ns(),
            graphics: clip.graphics,
            image: frame
                .image_id
                .checked_add(direction)
                .context("image overflow")?,
            subframe: frame.subframe,
            source_frame,
            action,
            phase,
            position,
            events: frame.events.iter().map(|e| format!("{e:?}")).collect(),
            static_prop: false,
            entered_events,
        };
        if guardian || cube_loop {self.tracks.get_mut(&actor.id).unwrap().object_emitted=object_end;}
        Ok(pose)
    }
}
