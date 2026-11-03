//! Three-tower, gate, team and terminal proof in canonical ENGINE space.
//! A config digest binds configuration, not local terrain reachability or SQL.
use super::{combat::require, data::*, invalid, scene::initial, ApplicationError};
use sha2::{Digest, Sha256};

pub(super) fn config_hash(s: &Siege) -> [u8; 32] {
    let mut b = Vec::new();
    let name = b"modern-siege-policy-v1";
    b.extend((name.len() as u16).to_be_bytes());
    b.extend(name);
    b.extend(s.mode.to_be_bytes());
    b.extend(s.logical_map.to_be_bytes());
    let cell = |b: &mut Vec<u8>, c: &Cell| {
        b.extend(c.x.to_be_bytes());
        b.extend(c.y.to_be_bytes());
    };
    for t in &s.towers.0 {
        b.push(t.slot as u8);
        cell(&mut b, &t.cell);
        b.extend(t.graphic.to_be_bytes());
        b.extend(t.maximum_hp.to_be_bytes());
    }
    let mut gate = s.gate.0.iter().collect::<Vec<_>>();
    gate.sort_by_key(|c| (c.x, c.y));
    b.extend((gate.len() as i32).to_be_bytes());
    for c in gate {
        cell(&mut b, c);
    }
    for camp in [&s.attack_camp.0, &s.defense_camp.0] {
        b.extend((camp.len() as i32).to_be_bytes());
        for c in camp {
            cell(&mut b, c);
        }
    }
    cell(&mut b, &s.portal_outside);
    cell(&mut b, &s.portal_inside);
    b.extend(s.duration_millis.to_be_bytes());
    b.extend(s.respawn_millis.to_be_bytes());
    Sha256::digest(b).into()
}
impl UnvalidatedData {
    pub(super) fn check_siege(&self) -> Result<(), ApplicationError> {
        let Some(s) = &self.siege else {
            return require(self.result.as_ref().is_none_or(|r| r.mode != 3));
        };
        require(
            s.mode == 3
                && s.mode == self.room.mode
                && s.logical_map == self.room.map
                && [547, 563, 573].contains(&s.logical_map)
                && s.towers.0.len() == 3
                && s.objectives.0.len() == 3
                && (1..=64).contains(&s.gate.0.len())
                && (1..=50).contains(&s.attack_camp.0.len())
                && (1..=50).contains(&s.defense_camp.0.len())
                && (50..=1_800_000).contains(&s.duration_millis)
                && s.duration_millis == i64::from(self.world.descriptor.active_timeout_millis)
                && (1..=60_000).contains(&s.respawn_millis)
                && (0..=1).contains(&s.attacker)
                && (1..=self.world.revision).contains(&s.revision)
                && (0..=s.revision).contains(&s.gate_revision)
                && (0..=s.revision).contains(&s.finish_revision)
                && (0..=s.duration_millis).contains(&s.finished_at_millis)
                && (-1..=1).contains(&s.winner)
                && (s.phase == SiegePhase::Waiting) == initial(self.world.phase)
                && (s.phase == SiegePhase::Finished) == (self.world.phase == Phase::Finished)
                && (s.phase == SiegePhase::Aborted) == (self.world.phase == Phase::Aborted),
        )?;
        let mut cells = std::collections::BTreeSet::new();
        for c in s
            .gate
            .0
            .iter()
            .chain(&s.attack_camp.0)
            .chain(&s.defense_camp.0)
            .chain([&s.portal_inside, &s.portal_outside])
            .chain(s.towers.0.iter().map(|t| &t.cell))
        {
            require(self.manifest.contains_cell(c) && cells.insert((c.x, c.y)))?;
        }
        let mut teams = [0usize; 2];
        for a in &self.actors.0 {
            teams[self.scene_member(a)?.team as usize] += 1;
            require(
                !s.towers.0.iter().any(|t| t.cell == a.position)
                    && (s.gate_revision > 0 || !s.gate.0.contains(&a.position)),
            )?;
        }
        require(teams.iter().all(|n| (15..=50).contains(n)))?;
        for (i, (t, o)) in s.towers.0.iter().zip(&s.objectives.0).enumerate() {
            require(
                t.slot as usize == i
                    && t.maximum_hp > 0
                    && [2665, 2667, 2669, 2671].contains(&t.graphic)
                    && *t == o.tower
                    && (0..=t.maximum_hp).contains(&o.hp)
                    && o.damage_loss == t.maximum_hp - o.hp
                    && o.hit_count >= 0
                    && o.hit_count <= o.revision
                    && (0..=s.revision).contains(&o.revision)
                    && (0..=o.revision).contains(&o.death_revision)
                    && (0..=self.actors.0.len() as i64).contains(&o.last_actor)
                    && (o.hit_count == 0) == (o.revision == 0)
                    && (o.hit_count == 0) == (o.last_actor == 0)
                    && (o.hp == 0) == (o.death_revision > 0)
                    && (o.hp != 0 || o.death_revision == o.revision)
                    && (o.hit_count != 0 || o.damage_loss == 0)
                    && (s.phase != SiegePhase::Waiting || o.hit_count == 0)
                    && (i != 2
                        || o.hit_count == 0
                        || s.gate_revision > 0 && o.revision > s.gate_revision),
            )?;
            if o.hit_count > 0 {
                require(self.scene_member(self.scene_actor(o.last_actor)?)?.team == s.attacker)?;
            }
        }
        let outer = s.objectives.0[0].hp == 0 && s.objectives.0[1].hp == 0;
        let inner = s.objectives.0[2].hp == 0;
        require(
            (s.gate_revision > 0) == outer
                && (!inner || outer)
                && (!outer
                    || s.gate_revision
                        == s.objectives.0[0]
                            .death_revision
                            .max(s.objectives.0[1].death_revision))
                && (!inner || s.finish_revision == s.objectives.0[2].death_revision)
                && (s.phase != SiegePhase::Outer || !outer)
                && (s.phase != SiegePhase::Inner || outer && !inner),
        )?;
        if s.phase != SiegePhase::Finished {
            return require(
                s.winner == -1
                    && s.finish_revision == 0
                    && s.finished_at_millis == 0
                    && !inner
                    && self.result.is_none(),
            );
        }
        require(
            s.finish_revision == s.revision
                && s.finish_revision == self.world.revision
                && s.winner >= 0
                && s.finished_at_millis <= self.world.active_elapsed_nanos / 1_000_000
                && (s.winner == s.attacker) == inner
                && if inner {
                    s.finished_at_millis < s.duration_millis
                } else {
                    s.finished_at_millis == s.duration_millis
                },
        )?;
        let r = self.result.as_ref().ok_or_else(invalid)?;
        let p = r.siege.as_ref().ok_or_else(invalid)?;
        require(
            r.mode == 3
                && p.config_sha256 == config_hash(s)
                && p.attacker == s.attacker
                && p.winner == s.winner
                && p.end
                    == if inner {
                        SiegeEnd::InnerDestroyed
                    } else {
                        SiegeEnd::Timeout
                    }
                && i64::from(p.duration_millis) == s.duration_millis
                && i64::from(p.respawn_millis) == s.respawn_millis
                && p.elapsed_millis == s.finished_at_millis
                && p.gate_revision == s.gate_revision
                && p.towers == s.objectives
                && s.gate_revision < self.world.revision
                && s.objectives.0.iter().all(|o| {
                    (o.tower.slot == SiegeSlot::Inner && inner) || o.revision < self.world.revision
                })
                && r.participants
                    .0
                    .iter()
                    .all(|p| (p.verdict == Verdict::Win) == (p.team == s.winner)),
        )
    }
}
