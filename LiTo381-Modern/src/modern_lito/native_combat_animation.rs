//! Bounded server action playback and exactly-once frame-entry events.
use super::{
    native_combat::ActionCue,
    native_world::Actor,
    native_world_animation::{resolve, Clip},
    sprite_actions::{FrameEvent, SpriteActionCatalog},
};
use anyhow::{ensure, Context, Result};
use std::{collections::{BTreeMap, VecDeque}, hash::BuildHasher};
#[derive(Clone, Debug, serde::Serialize)]
pub struct EnteredEvent {
    pub actor: u32,
    pub sequence: u64,
    pub action: u8,
    pub frame: usize,
    pub event: String,
    pub sound: Option<u32>,
    #[serde(skip)]
    pub prepared_audio: Option<super::native_audio_delivery::PreparedCue>,
}

/// The catalog's historical `Effect` name denotes the '<' overlapping WAV
/// cue. Select one weighted candidate through the overlapping voice path.
/// The caller supplies presentation randomness. Hit remains a visual trigger.
pub(crate) fn frame_sound(events: &[FrameEvent], index: usize, selection: u32) -> Option<u32> {
    match events.get(index)? {
        FrameEvent::Sound(id) => Some(*id),
        FrameEvent::Effect(_) => {
            let group = super::combat_animation::classify_native_cues(events).ok()?.audio?;
            (index == 0).then(|| group.selected(selection)).flatten()
        }
        _ => None,
    }
}
pub struct ActionPose {
    pub action: u8,
    pub elapsed_ns: u64,
    pub phase: &'static str,
}
#[derive(Clone)]
struct Active {
    cue: ActionCue,
    units: u128,
    emitted: usize,
}
#[derive(Clone)]
struct State {
    incarnation: u64,
    seen: u64,
    last: u64,
    speed: u8,
    brave: u8,
    active: Option<Active>,
    queue: VecDeque<ActionCue>,
}
#[derive(Clone,Default)]
pub(crate) struct History { states:BTreeMap<u32,State> }
#[derive(Default)]
pub struct Playback {
    states: BTreeMap<u32, State>,
    clips: BTreeMap<(u32, u8), Clip>,
}
impl Playback {
    pub(crate) fn history(&self)->History {History{states:self.states.clone()}}
    pub(crate) fn restore(&mut self,history:History){self.states=history.states;self.clips.clear();}
    pub(crate) fn admit(&mut self,actor:&Actor,sequence:u64,now:u64) {
        let state=self.states.entry(actor.id).or_insert(State{incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,active:None,queue:VecDeque::new()});
        if state.incarnation!=actor.incarnation {*state=State{incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,active:None,queue:VecDeque::new()};}
        state.seen=state.seen.max(sequence);
        state.queue.retain(|cue|cue.sequence>sequence);
        // An admitted death stays on its final pose without entering old events.
        if actor.dead && !state.active.as_ref().is_some_and(|active|active.cue.action==8) {
            state.active=Some(Active{cue:ActionCue{sequence,action:8,source:"action"},units:u128::MAX,emitted:usize::MAX});
        }
    }

    /// A presentation-only arrival. The caller has already sampled this actor
    /// at `now`, so the next sample cannot spend pre-impact wall time on it.
    pub fn impact(&mut self, actor: &Actor, sequence: u64) -> bool {
        let Some(state) = self.states.get_mut(&actor.id) else {
            return false;
        };
        if actor.dead || super::native_obstacles::guardian(actor)
            || state.incarnation != actor.incarnation
            || state.queue.len() >= 16
            || state.queue.back().is_some_and(|cue| cue.action == 2)
            || state
                .active
                .as_ref()
                .is_some_and(|a| matches!(a.cue.action, 8 | 44 | 45))
            || state.queue.iter().any(|c| matches!(c.action, 8 | 44 | 45))
        {
            return false;
        }
        state.queue.push_back(ActionCue {
            sequence,
            action: 2,
            source: "projectile_arrival",
        });
        true
    }
    /// A native running walk owns the body until its boundary. Impact may
    /// enqueue hurt meanwhile, but walk wall time must not age that action.
    pub(crate) fn hold_for_motion(&mut self,actor:&Actor,now:u64)->Result<()> {
        let state=self.states.entry(actor.id).or_insert(State{
            incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,
            active:None,queue:VecDeque::new(),
        });
        ensure!(now>=state.last,"combat clock reversed while walking");
        if state.incarnation!=actor.incarnation || actor.action_sequence<state.seen {
            state.incarnation=actor.incarnation;state.seen=0;state.active=None;state.queue.clear();
        }
        state.last=now;state.speed=actor.speed;state.brave=actor.brave;Ok(())
    }
    pub fn retain(&mut self, ids: &std::collections::BTreeSet<u32>) {
        self.states.retain(|id, _| ids.contains(id));
    }
    /// True once every received cue up to `barrier` has left the received
    /// history for playback: it is active, finished, or was coalesced. A walk
    /// hold or a backlog keeps later HP/MP presentation behind the action.
    pub(crate) fn presented(&self, actor: &Actor, barrier: u64) -> bool {
        let state = self.states.get(&actor.id).filter(|s| s.incarnation == actor.incarnation);
        let seen = state.map_or(0, |s| s.seen);
        let unseen = actor.actions.iter().any(|c| c.sequence <= barrier && c.sequence > seen);
        let queued = state.is_some_and(|s| s.queue.iter().any(|c| c.sequence <= barrier));
        !unseen && !queued
    }
    pub fn sample(
        &mut self,
        catalog: &SpriteActionCatalog,
        actor: &Actor,
        now: u64,
    ) -> Result<(Option<ActionPose>, Vec<EnteredEvent>)> {
        let state = self.states.entry(actor.id).or_insert(State {
            incarnation: actor.incarnation,
            seen: 0,
            last: now,
            speed: actor.speed,
            brave: actor.brave,
            active: None,
            queue: VecDeque::new(),
        });
        // A fresh authoritative S_PUT_OBJECT reuses the ID but retires its old
        // action history (including a held death pose).
        if actor.incarnation != state.incarnation || actor.action_sequence < state.seen {
            state.incarnation = actor.incarnation;
            state.seen = 0;
            state.active = None;
            state.queue.clear();
            state.last = now;
        }
        let mut wall = u128::from(
            now.checked_sub(state.last)
                .context("combat clock reversed")?,
        ) * 126;
        // A newly observed command starts now, not at a guessed packet arrival time.
        let had_active = state.active.is_some();
        for cue in actor.actions.iter().filter(|c| c.sequence > state.seen) {
            if cue.action == 8 {
                state.active = None;
                state.queue.clear();
                wall = 0;
            }
            // Native 4E63C0 suppresses consecutive queued action-2 commands.
            // Keep one pending hurt pose: a crowd must not build an unbounded
            // replay of old damage. Never merge across an attack/spell cue,
            // whose frame-entry Hit/Sound events still have to run once.
            if cue.action == 2 && state.queue.back().is_some_and(|c| c.action == 2) {
                continue;
            }
            ensure!(state.queue.len() < 16, "combat action backlog");
            state.queue.push_back(cue.clone());
        }
        state.seen = actor.action_sequence;
        if !had_active {
            wall = 0;
        }
        let mut events = Vec::new();
        loop {
            if state.active.is_none() {
                let Some(cue) = state.queue.pop_front() else {
                    break;
                };
                state.active = Some(Active {
                    cue,
                    units: 0,
                    emitted: 0,
                });
            }
            let active = state.active.as_mut().unwrap();
            let key = (u32::from(actor.graphics), active.cue.action);
            if !self.clips.contains_key(&key) {
                ensure!(self.clips.len() < 256, "combat clip budget");
                self.clips.insert(key, resolve(catalog, key.0, key.1)?);
            }
            let clip = &self.clips[&key];
            let duration = u128::from(clip.duration_ns()) * 126;
            let (n, d) = super::native_speed::factor(state.speed, state.brave, active.cue.action)?;
            let required =
                ((duration.saturating_sub(active.units)) * u128::from(n)).div_ceil(u128::from(d));
            if wall >= required {
                active.units = duration;
                wall -= required;
            } else {
                active.units += wall * u128::from(d) / u128::from(n);
                wall = 0;
            }
            let elapsed = u64::try_from(active.units / 126)?;
            let last = if active.units >= duration {
                clip.animation.frames.len() - 1
            } else {
                clip.sample(elapsed, false)
            };
            for index in active.emitted..=last {
                let frame_events = &clip.animation.frames[index].events;
                let selection = std::collections::hash_map::RandomState::new()
                    .hash_one((actor.id, active.cue.sequence, index)) as u32;
                for (event_index, event) in frame_events.iter().enumerate() {
                    ensure!(events.len() < 256, "combat frame event budget");
                    events.push(EnteredEvent { prepared_audio:None,
                        actor: actor.id,
                        sequence: active.cue.sequence,
                        action: active.cue.action,
                        frame: index,
                        event: format!("{event:?}"),
                        sound: frame_sound(frame_events, event_index, selection),
                    });
                }
            }
            active.emitted = last + 1;
            if active.units < duration || active.cue.action == 8 {
                break;
            }
            state.active = None;
        }
        state.last = now;
        state.speed = actor.speed;
        state.brave = actor.brave;
        Ok((
            state.active.as_ref().map(|a| ActionPose {
                action: a.cue.action,
                elapsed_ns: (a.units / 126) as u64,
                phase: if a.cue.action == 8 {
                    "death"
                } else if a.cue.action == 2 {
                    "damage"
                } else if a.cue.source == "attack" {
                    "attack"
                } else {
                    "action"
                },
            }),
            events,
        ))
    }
}
