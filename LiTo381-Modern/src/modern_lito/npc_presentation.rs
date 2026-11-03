//! Explicit immutable preflight roster and worker-owned NPC presentation.
//! No archive I/O after construction; no player IDs, speculative HP, pathing,
//! collision updates or local corpse expiry. Production admission must supply
//! this plan BEFORE LOADED, not infer future resources from currently spawned NPCs.
use super::{
    npc_assets::{NpcAction, NpcAppearance, NpcAssets},
    npc_protocol::{self as wire, Actor, Direction, Reply, Scope},
    shared_action_catalog::SharedActionCatalog,
    world_scene::NpcDraw,
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    scope: Scope,
    appearances: Vec<NpcAppearance>,
    light_arrow: bool,
    disintegrate: bool,
    fire_storm: bool,
}
impl Plan {
    /// The formal client route must use the v18 server-provided requirements,
    /// including an empty LOADING roster. Never infer these from actors.
    pub fn from_reply(reply: &Reply) -> Result<Self> {
        reply.validate()?;
        let frame = reply
            .frame
            .as_ref()
            .context("NPC resource snapshot absent")?;

        let forms = frame
            .resources
            .as_ref()
            .context("NPC peer did not supply resource preflight")?;
        let mut plan = Self::new(
            frame.scope,
            forms
                .iter()
                .map(|g| NpcAppearance::new(*g))
                .collect::<Result<_>>()?,
        )?;
        // World resource preparation must also preload the shared light-arrow
        // caster/effect assets before LOADED. This pins capability, not pixels.
        plan.light_arrow = !frame.combat.npc_spells.is_empty();
        plan.disintegrate = !frame.combat.npc_disintegrations.is_empty();
        plan.fire_storm = !frame.combat.npc_fires.is_empty();
        Ok(plan)
    }
    /// Trusted encounter resource contract, not a packet spawn or permission.
    /// The existing 128 MiB decoded-asset limit still applies at load time.
    pub fn new(scope: Scope, mut appearances: Vec<NpcAppearance>) -> Result<Self> {
        ensure!(
            [scope.room_id, scope.match_id, scope.generation]
                .into_iter()
                .all(|n| n > 0 && n <= i64::MAX as u64),
            "invalid NPC plan scope"
        );
        ensure!(
            !appearances.is_empty() && appearances.len() <= 10,
            "NPC plan roster bound"
        );
        appearances.sort();
        ensure!(
            appearances.windows(2).all(|a| a[0] != a[1]),
            "duplicate NPC plan form"
        );
        Ok(Self {
            scope,
            appearances,
            light_arrow: false,
            disintegrate: false,
            fire_storm: false,
        })
    }
    pub(crate) fn validate(&self, reply: &Reply) -> Result<()> {
        reply.validate()?;
        let frame = reply
            .frame
            .as_ref()
            .context("NPC presentation snapshot absent")?;

        ensure!(
            self.fire_storm == !frame.combat.npc_fires.is_empty(),
            "NPC70 resource capability changed"
        );
        ensure!(
            self.disintegrate == !frame.combat.npc_disintegrations.is_empty(),
            "NPC77 resource capability changed"
        );
        ensure!(
            self.light_arrow == !frame.combat.npc_spells.is_empty(),
            "NPC spell resource capability changed"
        );
        ensure!(frame.scope == self.scope, "NPC plan scope mismatch");
        if frame.resources.is_some() {
            ensure!(
                &Self::from_reply(reply)? == self,
                "NPC preflight differs from server resource requirements"
            );
        }
        for actor in &frame.actors {
            ensure!(
                self.appearances
                    .binary_search(&NpcAppearance::new(actor.graphic)?)
                    .is_ok(),
                "NPC form absent from preflight plan"
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ConfirmedAttack {
    pub revision: i64,
    pub cell: Option<super::world_map::WorldCell>,
}
pub(crate) fn player_attack(frame: &wire::Frame, entity: u64) -> Option<ConfirmedAttack> {
    let identity = frame
        .combat
        .world
        .actors
        .iter()
        .find(|p| p.entity_id == entity)?
        .identity();
    let npc = frame
        .exchanges
        .iter()
        .find(|e| e.direction == Direction::PlayerToNpc && e.participant == identity)
        .map(|e| ConfirmedAttack {
            revision: e.revision,
            cell: frame
                .actor(e.npc)
                .filter(|a| a.life == e.npc_life)
                .map(|a| a.cell),
        });
    let quest = frame.combat.world.quest.as_ref().and_then(|q| {
        q.player_attack(entity).map(|e| ConfirmedAttack {
            revision: e.revision,
            cell: q.target(e.slot).map(|t| t.cell),
        })
    });
    let tower = frame.combat.world.time_attack.as_ref().and_then(|d| {
        d.exchanges
            .iter()
            .find(|e| e.entity_id == entity)
            .and_then(|e| {
                // Cooperative receipts retain raw engine revisions; PC poses use wire + 1.
                Some(ConfirmedAttack {
                    revision: e.revision.checked_add(1)?,
                    cell: Some(d.tower_cell),
                })
            })
    });
    [npc, quest, tower]
        .into_iter()
        .flatten()
        .max_by_key(|e| e.revision)
}

struct Visual {
    actor: Actor,
    action: NpcAction,
    started: Instant,
    marker: Option<(NpcAction, u128, usize)>,
    last_sample: Instant,
    silent: bool,
}
impl Visual {
    fn baseline(actor: &Actor, now: Instant) -> Self {
        // A late first snapshot is not evidence that an old attack just began.
        Self {
            actor: actor.clone(),
            action: if actor.alive() {
                NpcAction::Breath
            } else {
                NpcAction::Death
            },
            started: now,
            marker: None,
            last_sample: now,
            silent: true,
        }
    }
}
pub(crate) struct Presentation {
    pub(crate) assets: NpcAssets,
    plan: Arc<Plan>,
    previous: Option<Arc<Reply>>,
    visuals: BTreeMap<u64, Visual>,
}
impl Presentation {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        plan: Arc<Plan>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let assets = NpcAssets::load_with_catalog(catalog, &plan.appearances, cancel)?;
        Ok(Self {
            assets,
            plan,
            previous: None,
            visuals: BTreeMap::new(),
        })
    }
    pub(crate) fn update(
        &mut self,
        reply: Arc<Reply>,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(
            !cancel.load(Ordering::Acquire),
            "NPC presentation cancelled"
        );
        self.plan.validate(&reply)?;
        ensure!(
            wire::follows(self.previous.as_deref(), &reply),
            "NPC presentation rewound"
        );
        ensure!(
            self.visuals.values().all(|v| now >= v.last_sample),
            "NPC presentation clock rewound"
        );
        let f = reply.frame.as_ref().unwrap();
        let ids: BTreeSet<_> = f.actors.iter().map(|a| a.serial).collect();
        self.visuals.retain(|id, _| ids.contains(id));
        for actor in &f.actors {
            let v = self
                .visuals
                .entry(actor.serial)
                .or_insert_with(|| Visual::baseline(actor, now));
            if actor.life != v.actor.life {
                *v = Visual::baseline(actor, now);
                continue;
            }
            if actor.changed_revision > v.actor.changed_revision {
                let action = if !actor.alive() && v.actor.alive() {
                    Some(NpcAction::Death)
                } else if actor.action == wire::NpcAction::Hurt && actor.hp < v.actor.hp {
                    Some(NpcAction::Damage)
                } else if actor.action == wire::NpcAction::Attack
                    && self.previous.as_ref().is_some_and(|p| {
                        f.exchanges.iter().any(|e| {
                            e.direction == Direction::NpcToParticipant
                                && e.npc.serial == actor.serial
                                && e.npc_life == actor.life
                                && e.revision > p.frame.as_ref().unwrap().combat.world.revision
                        })
                    })
                {
                    Some(NpcAction::Attack)
                } else if actor.action == wire::NpcAction::Walk && actor.cell != v.actor.cell {
                    Some(NpcAction::Walk)
                } else if actor.action == wire::NpcAction::Idle && actor.action != v.actor.action {
                    Some(NpcAction::Breath)
                } else {
                    None
                };
                if let Some(action) = action {
                    v.action = action;
                    v.started = now;
                    v.marker = None;
                    v.silent = false;
                }
            }
            v.actor = actor.clone();
        }
        self.previous = Some(reply);
        Ok(())
    }
    /// Only the current source's cue can be delivered. Slow polling drops old
    /// audio instead of replaying an unbounded backlog. Hit events do nothing.
    pub(crate) fn sample(
        &mut self,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Vec<super::combat_animation::CrossedCue>> {
        let mut cues = Vec::new();
        for (visual_index, v) in self.visuals.values_mut().enumerate() {
            ensure!(
                !cancel.load(Ordering::Acquire) && now >= v.last_sample,
                "NPC sample cancelled/rewound"
            );
            let appearance = NpcAppearance::new(v.actor.graphic)?;
            let duration = self.assets.action(appearance, v.action)?.duration();
            let mut elapsed = now.saturating_duration_since(v.started);
            if !matches!(v.action, NpcAction::Death | NpcAction::Breath) && elapsed >= duration {
                v.action = NpcAction::Breath;
                v.started = v
                    .started
                    .checked_add(duration)
                    .context("NPC clock overflow")?;
                v.marker = None;
                elapsed = now.saturating_duration_since(v.started);
            }
            let action = self.assets.action(appearance, v.action)?;
            let (source, cycle) = if v.silent && v.action == NpcAction::Death {
                (0, 0)
            } else {
                action.sample(elapsed, v.actor.heading)?
            };
            let marker = (v.action, cycle, source);
            if !v.silent
                && v.marker != Some(marker)
                && now.duration_since(v.last_sample) <= Duration::from_millis(500)
            {
                // This is a missed-poll bound, NOT a maximum action age.
                // Regularly sampled long attacks may have valid later cues.
                if action.sound_group(source)?.is_some() {
                    // AudioSequencer groups contiguous cues by source_frame.
                    // One unique per-actor index prevents two NPCs merging.
                    for (event_index, event) in action.source_frames()[source]
                        .events
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            !matches!(
                                e,
                                super::sprite_actions::FrameEvent::Hit
                                    | super::sprite_actions::FrameEvent::Skip
                            )
                        })
                    {
                        cues.push(super::combat_animation::CrossedCue {
                            source_frame: visual_index,
                            event_index,
                            tick: 0,
                            event: event.clone(),
                        });
                    }
                }
            }
            v.marker = Some(marker);
            v.last_sample = now;
            // Baseline silence is retained for a historical corpse; idle may
            // emit on its NEXT source change, never first-snapshot old attacks.
            if v.action != NpcAction::Death {
                v.silent = false;
            }
        }
        Ok(cues)
    }
    pub(crate) fn draws(&self) -> Result<Vec<NpcDraw<'_>>> {
        self.visuals
            .values()
            .map(|v| {
                let (_, _, source) = v.marker.context("NPC not sampled")?;
                Ok(NpcDraw {
                    walk_from: None,
                    id: wire::Id {
                        scope: self.plan.scope,
                        serial: v.actor.serial,
                    },
                    actor: (&v.actor).into(),
                    source_frame: source,
                    sample: self.assets.frame(
                        NpcAppearance::new(v.actor.graphic)?,
                        v.action,
                        v.actor.heading,
                        source,
                    )?,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Presentation, Reply) {
        let r = wire::tests::read_reply();
        let a = NpcAppearance::ALL[0];
        let p = Presentation {
            assets: super::super::npc_assets::tests::presentation_fixture(a),
            plan: Arc::new(Plan::new(r.echo.world, vec![a]).unwrap()),
            previous: None,
            visuals: BTreeMap::new(),
        };
        (p, r)
    }
    fn advance(r: &Reply) -> Reply {
        let mut r = r.clone();
        let f = r.frame.as_mut().unwrap();
        f.combat.world.revision += 1;
        f.revision += 1;
        f.elapsed_millis += 1;
        r
    }
    #[test]
    fn historical_attack_is_idle_and_new_hurt_does_not_restart_on_poll() {
        let (mut p, r) = fixture();
        let now = Instant::now();
        let c = AtomicBool::new(false);
        p.update(Arc::new(r.clone()), now, &c).unwrap();
        assert!(p.sample(now, &c).unwrap().is_empty());
        assert!(p.visuals.values().all(|v| v.action == NpcAction::Breath));
        let mut hurt = advance(&r);
        let f = hurt.frame.as_mut().unwrap();
        f.actors[0].hp -= 1;
        f.actors[0].changed_revision = f.revision;
        let at = now + Duration::from_millis(10);
        p.update(Arc::new(hurt.clone()), at, &c).unwrap();
        p.sample(at, &c).unwrap();
        assert_eq!(p.visuals[&1].action, NpcAction::Damage);
        let started = p.visuals[&1].started;
        p.update(Arc::new(hurt), at + Duration::from_millis(130), &c)
            .unwrap();
        p.sample(at + Duration::from_millis(130), &c).unwrap();
        assert_eq!(p.visuals[&1].started, started);
        assert_eq!(p.draws().unwrap()[0].source_frame, 2);
        p.sample(at + Duration::from_secs(20), &c).unwrap();
        assert_eq!(p.visuals[&1].action, NpcAction::Breath);
        assert!(p.sample(now, &c).is_err());
    }
    #[test]
    fn corpse_stays_until_authority_retires_it_and_new_life_discards_death_clock() {
        let (mut p, r) = fixture();
        let now = Instant::now();
        let c = AtomicBool::new(false);
        p.update(Arc::new(r.clone()), now, &c).unwrap();
        p.sample(now, &c).unwrap();
        let mut dead = advance(&r);
        let f = dead.frame.as_mut().unwrap();
        let n = &mut f.actors[0];
        n.hp = 0;
        n.action = wire::NpcAction::Death;
        n.changed_revision = f.revision;
        n.corpse_until_millis = f.elapsed_millis + 100;
        n.next_move_millis = 0;
        n.next_attack_millis = 0;
        p.update(Arc::new(dead.clone()), now + Duration::from_millis(1), &c)
            .unwrap();
        p.sample(now + Duration::from_secs(5), &c).unwrap();
        assert_eq!(p.draws().unwrap()[0].source_frame, 0);
        assert_eq!(
            p.visuals.len(),
            2,
            "wall clock cannot expire an authority corpse"
        );
        let mut resurrected = advance(&dead);
        let f = resurrected.frame.as_mut().unwrap();
        let n = &mut f.actors[0];
        n.life += 1;
        n.hp = n.max_hp;
        n.action = wire::NpcAction::Idle;
        n.corpse_until_millis = 0;
        n.changed_revision = f.revision;
        p.update(Arc::new(resurrected), now + Duration::from_secs(6), &c)
            .unwrap();
        assert!(p
            .sample(now + Duration::from_secs(6), &c)
            .unwrap()
            .is_empty());
        assert_eq!(p.visuals[&1].actor.life, 2);
        assert_eq!(p.visuals[&1].action, NpcAction::Breath);
        // Separate retirement branch keeps historical player->NPC receipt but
        // removes the NPC-owned receipt with the retired NPC.
        let (mut p, _) = fixture();
        p.update(Arc::new(dead.clone()), now, &c).unwrap();
        let mut retired = advance(&dead);
        let f = retired.frame.as_mut().unwrap();
        f.elapsed_millis = f.actors[0].corpse_until_millis;
        f.actors.remove(0);
        f.exchanges
            .retain(|e| e.direction != Direction::NpcToParticipant || e.npc.serial != 1);
        p.update(Arc::new(retired), now, &c).unwrap();
        p.sample(now, &c).unwrap();
        assert_eq!(p.draws().unwrap().len(), 1);
        assert!(!p.visuals.contains_key(&1));
    }
    #[test]
    fn new_attack_receipt_emits_once_and_cancel_or_plan_mismatch_never_mutates_visuals() {
        let (mut p, r) = fixture();
        let now = Instant::now();
        let c = AtomicBool::new(false);
        p.update(Arc::new(r.clone()), now, &c).unwrap();
        p.sample(now, &c).unwrap();
        let mut attack = advance(&r);
        let f = attack.frame.as_mut().unwrap();
        f.actors[0].action = wire::NpcAction::Attack;
        f.actors[0].next_attack_millis += 700;
        f.actors[0].changed_revision = f.revision;
        f.combat.vitals[0].hp -= 28;
        f.combat.vitals[0].hurt_revision = f.combat.world.revision;
        let e = f
            .exchanges
            .iter_mut()
            .find(|e| e.direction == Direction::NpcToParticipant && e.npc.serial == 1)
            .unwrap();
        e.revision = f.combat.world.revision;
        e.outcome.remaining_hp -= 28;
        f.exchanges.sort_by_key(|e| e.revision);
        p.update(Arc::new(attack.clone()), now, &c).unwrap();
        assert_eq!(p.sample(now, &c).unwrap().len(), 1);
        assert!(p.sample(now, &c).unwrap().is_empty());
        for step in 1..5 {
            assert!(p
                .sample(now + Duration::from_millis(step * 125), &c)
                .unwrap()
                .is_empty());
        }
        assert_eq!(
            p.sample(now + Duration::from_millis(625), &c)
                .unwrap()
                .len(),
            1,
            "on-time late attack source is not a stale audio backlog"
        );
        assert!(p
            .sample(now + Duration::from_secs(10), &c)
            .unwrap()
            .is_empty());
        let before = p.visuals[&1].actor.clone();
        let mut wrong = advance(&attack);
        wrong.frame.as_mut().unwrap().actors[0].graphic = 1578;
        assert!(p
            .update(Arc::new(wrong), now + Duration::from_secs(11), &c)
            .is_err());
        assert!(p
            .update(
                Arc::new(advance(&attack)),
                now + Duration::from_secs(11),
                &AtomicBool::new(true)
            )
            .is_err());
        assert_eq!(p.visuals[&1].actor, before);
    }
}
