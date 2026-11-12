//! Bounded server action playback and exactly-once frame-entry events.
use super::{
    native_combat::ActionCue,
    native_world::Actor,
    native_world_animation::{resolve, Clip},
    sprite_actions::{FrameEvent, SpriteActionCatalog},
};
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, VecDeque};
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
    /// Native 4E6B70: the body is in its running walk (`hold_for_motion`).
    walking: bool,
}
impl State {
    fn fresh(actor: &Actor, now: u64) -> Self {
        Self {
            incarnation: actor.incarnation,
            seen: 0,
            last: now,
            speed: actor.speed,
            brave: actor.brave,
            active: None,
            queue: VecDeque::new(),
            walking: false,
        }
    }
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
        let state=self.states.entry(actor.id).or_insert_with(||State::fresh(actor,now));
        if state.incarnation!=actor.incarnation {*state=State::fresh(actor,now);}
        state.seen=state.seen.max(sequence);
        state.queue.retain(|cue|cue.sequence>sequence);
        // An admitted death stays on its final pose without entering old events.
        if actor.dead && !state.active.as_ref().is_some_and(|active|active.cue.action==8) {
            state.active=Some(Active{cue:ActionCue{sequence,action:8,source:"action"},units:u128::MAX,emitted:usize::MAX});
        }
    }

    /// A presentation-only arrival (effect-frame Hit, 44ECCE..44ED59). The
    /// caller has already sampled this actor at `now`, so the next sample
    /// cannot spend pre-impact wall time on it.
    ///
    /// Native order: dead / 4E8EC0 (44/45) / no-hurt bodies / 4DF7C0 /
    /// `base+26` are ignored; a running walk (4E6B70) gets `4E63C0(2)` with
    /// its >3 threshold; any other body is interrupted by `4E45F0(2)`, which
    /// replaces the running action and keeps the pending commands.
    pub fn impact(&mut self, actor: &Actor, sequence: u64) -> bool {
        use super::native_hurt::{dispatch, queue_suppresses, HurtDispatch, HurtTarget};
        let Some(state) = self.states.get_mut(&actor.id) else {
            return false;
        };
        // Modern-retained guards: the guardian object clock, actor identity,
        // a pending authoritative death and the bounded modern queue.
        if super::native_obstacles::guardian(actor)
            || state.incarnation != actor.incarnation
            || state.queue.len() >= 16
            || state.queue.iter().any(|c| c.action == 8)
        {
            return false;
        }
        let running = state.active.as_ref().map(|a| a.cue.action);
        let target = HurtTarget {
            dead: actor.dead || running == Some(8),
            action_44_45: matches!(running, Some(44 | 45))
                || state.queue.iter().any(|c| matches!(c.action, 44 | 45)),
            graphics: actor.graphics,
            walking: state.walking,
        };
        let cue = ActionCue {
            sequence,
            action: 2,
            source: "projectile_arrival",
        };
        match dispatch(&target) {
            HurtDispatch::Ignore => false,
            HurtDispatch::Enqueue => {
                if queue_suppresses(state.queue.len(), state.queue.back().map(|c| c.action), 2) {
                    return false;
                }
                state.queue.push_back(cue);
                true
            }
            HurtDispatch::Interrupt => {
                // The running action's remaining frame events are not entered.
                state.active = None;
                state.queue.push_front(cue);
                true
            }
        }
    }
    /// A native running walk owns the body until its boundary. Impact may
    /// enqueue hurt meanwhile, but walk wall time must not age that action.
    pub(crate) fn hold_for_motion(&mut self,actor:&Actor,now:u64)->Result<()> {
        let state=self.states.entry(actor.id).or_insert_with(||State::fresh(actor,now));
        ensure!(now>=state.last,"combat clock reversed while walking");
        if state.incarnation!=actor.incarnation || actor.action_sequence<state.seen {
            state.incarnation=actor.incarnation;state.seen=0;state.active=None;state.queue.clear();
        }
        state.walking=true;
        state.last=now;state.speed=actor.speed;state.brave=actor.brave;Ok(())
    }
    pub fn retain(&mut self, ids: &std::collections::BTreeSet<u32>) {
        self.states.retain(|id, _| ids.contains(id));
    }
    pub fn sample(
        &mut self,
        catalog: &SpriteActionCatalog,
        actor: &Actor,
        now: u64,
    ) -> Result<(Option<ActionPose>, Vec<EnteredEvent>)> {
        use super::native_hurt::{hurt_exempt, queue_suppresses};
        let state = self.states.entry(actor.id).or_insert_with(|| State::fresh(actor, now));
        // The caller samples here only outside a held running walk.
        state.walking = false;
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
            // Native guard before every hurt insertion: these bodies never
            // take action 2 (4E461A/4E4655/4E466E, 4237D1..423808).
            if cue.action == 2 && hurt_exempt(actor.graphics) {
                continue;
            }
            // Native 4E63C0: with more than three pending commands, an
            // action 2 directly behind another action 2 is dropped. Attack and
            // spell cues are never merged, so their Hit/Sound events run once.
            if queue_suppresses(state.queue.len(), state.queue.back().map(|c| c.action), cue.action) {
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
                // 4E45F0 repeats the guard at action start: a polymorph after
                // insertion cannot let a queued hurt reach an exempt body.
                if cue.action == 2 && hurt_exempt(actor.graphics) {
                    continue;
                }
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
                for event in &clip.animation.frames[index].events {
                    ensure!(events.len() < 256, "combat frame event budget");
                    events.push(EnteredEvent { prepared_audio:None,
                        actor: actor.id,
                        sequence: active.cue.sequence,
                        action: active.cue.action,
                        frame: index,
                        event: format!("{event:?}"),
                        sound: if let FrameEvent::Sound(id) = event {
                            Some(*id)
                        } else {
                            None
                        },
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
#[cfg(test)]
#[path = "native_combat_animation_tests.rs"]
mod tests;
