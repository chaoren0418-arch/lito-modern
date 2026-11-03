//! Canonical pixels, hit targets and labels. No legacy Frame conversion, wire
//! revision translation, network operation, asset read or optimistic command ACK.
use super::{
    application_protocol::SelectionContext,
    battle_follow::Observation,
    battle_payload::data as d,
    battle_scene::{ObjectiveTarget, Surface},
    compositor::Viewport,
    world_map::{PixelPoint, WorldCell},
};
use anyhow::{ensure, Result};
#[path = "battle_honor_hud.rs"]
mod honor_hud;
#[path = "battle_race_hud.rs"]
mod race_hud;
#[path = "battle_time_attack_hud.rs"]
mod time_attack_hud;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub context: SelectionContext,
    pub channel: u8,
    pub world: [i64; 3],
    pub entity: i64,
    pub life: i64,
    pub revision: i64,
}
impl std::fmt::Debug for Stamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PaintedBattleStamp[redacted]")
    }
}
impl Stamp {
    pub fn of(o: &Observation) -> Self {
        let p = o.personal().player();
        let w = &o.data().world;
        Self {
            context: SelectionContext {
                key: super::application_protocol::CharacterKey {
                    source_id: p.source as u32,
                    object_id: p.object as u64,
                },
                epoch: p.epoch as u64,
            },
            channel: p.channel as u8,
            world: [w.identity.room, w.identity.match_id, w.identity.generation],
            entity: o.data().personal.entity,
            life: self_actor(o).map_or(0, life),
            revision: w.revision,
        }
    }
    pub fn current(self, o: &Observation) -> bool {
        let current = Self::of(o);
        self.context == current.context
            && self.channel == current.channel
            && self.world == current.world
            && self.entity == current.entity
            && self.life == current.life
            && self.revision <= current.revision
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Player { entity: i64, life: i64 },
    Npc { serial: i64, life: i64 },
    Honor { entity: i64 },
    Quest { slot: i32, born: i64 },
    TimeTower,
    Siege { slot: u8 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Local(Local),
    Navigate(WorldCell),
    Attack(Target),
    LightArrow(Target),
    Disintegrate(Target),
    Heal(Target),
    Haste(Target),
    Slow(Target),
    Portal,
    Cancel,
}
/// Non-target gameplay still belongs to the life/selection the user saw.
/// Values remain intents: dispatch retains the existing command/cadence checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Local {
    Step(u8),
    Teleport,
    FireStorm,
    GreaterHaste,
    UseItem(i64),
    SelectAmmo(i64),
}
impl Local {
    pub fn from_action(action: super::world_window::Action) -> Option<Self> {
        use super::world_window::Action as A;
        Some(match action {
            A::Step(h) => Self::Step(h),
            A::Teleport => Self::Teleport,
            A::FireStorm => Self::FireStorm,
            A::GreaterHaste => Self::GreaterHaste,
            A::UseItem(id) => Self::UseItem(id),
            A::SelectAmmo(id) => Self::SelectAmmo(id),
            _ => return None,
        })
    }
    pub fn action(self) -> super::world_window::Action {
        use super::world_window::Action as A;
        match self {
            Self::Step(h) => A::Step(h),
            Self::Teleport => A::Teleport,
            Self::FireStorm => A::FireStorm,
            Self::GreaterHaste => A::GreaterHaste,
            Self::UseItem(id) => A::UseItem(id),
            Self::SelectAmmo(id) => A::SelectAmmo(id),
        }
    }
    pub fn valid(self) -> bool {
        match self {
            Self::Step(h) => h < 8,
            Self::UseItem(id) => super::item_protocol::UI_ITEMS.contains(&id),
            Self::SelectAmmo(id) => super::ammo_protocol::UI_ITEMS.contains(&id),
            Self::Teleport | Self::FireStorm | Self::GreaterHaste => true,
        }
    }
}
pub(crate) fn stamp_local_action(
    action: super::world_window::Action,
    painted: Option<&Observation>,
) -> super::world_window::Action {
    match (Local::from_action(action), painted) {
        (Some(local), Some(o)) => super::world_window::Action::Canonical(Intent {
            stamp: Stamp::of(o),
            kind: Kind::Local(local),
        }),
        // The legacy window has no canonical picture. The normal dispatcher
        // must reject this unstamped path when CAP_BATTLE_DATA was negotiated.
        _ => action,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    pub stamp: Stamp,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Skill {
    #[default]
    Melee,
    LightArrow,
    Heal,
    Disintegrate,
    Haste,
    Slow,
}
pub struct Hit {
    pub intent: Intent,
    pub name: Option<(String, PixelPoint)>,
}
fn self_actor(o: &Observation) -> Option<&d::Actor> {
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
}
pub fn life(a: &d::Actor) -> i64 {
    super::battle_motion::life(a).unwrap_or(0)
}
fn team(o: &Observation, entity: i64) -> Option<i32> {
    o.data()
        .room
        .members
        .0
        .get(usize::try_from(entity.checked_sub(1)?).ok()?)
        .map(|m| m.team)
}
pub fn portal(o: &Observation) -> bool {
    let Some(s) = o.data().siege.as_ref() else {
        return false;
    };
    let Some(a) = self_actor(o) else {
        return false;
    };
    o.data().world.phase == d::Phase::Active
        && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        && team(o, a.entity).is_some_and(|t| t != s.attacker)
        && (a.position == s.portal_inside || a.position == s.portal_outside)
}
/// Recheck against the latest followed observation before dispatch. A stamped
/// death/respawn or replaced objective cannot target its successor life.
pub fn target_current(o: &Observation, target: Target) -> bool {
    let d = o.data();
    match target {
        Target::Player {
            entity,
            life: wanted,
        } => {
            entity != d.personal.entity
                && super::battle_space::occupies(d, entity)
                && d.actors.0.iter().any(|a| {
                    a.entity == entity
                        && life(a) == wanted
                        && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
                })
        }
        Target::Npc { serial, life } => d.npcs.as_ref().is_some_and(|n| {
            n.actors
                .0
                .iter()
                .any(|a| a.serial == serial && a.life == life && a.hp > 0)
        }),
        Target::Honor { entity } => d
            .honor
            .objectives
            .0
            .iter()
            .any(|a| a.entity == entity && a.hp > 0),
        Target::Quest { slot, born } => d.quest.as_ref().is_some_and(|q| {
            q.objectives
                .0
                .iter()
                .any(|a| a.id.slot == slot && a.born_revision == born && a.hp > 0)
        }),
        Target::TimeTower => d.time_attack.as_ref().is_some_and(|t| t.tower_hp > 0),
        Target::Siege { slot } => d.siege.as_ref().is_some_and(|s| {
            team(o, d.personal.entity) == Some(s.attacker)
                && s.objectives
                    .0
                    .iter()
                    .any(|a| a.tower.slot as u8 == slot && a.hp > 0)
                && if slot == 2 {
                    s.gate_revision > 0
                } else {
                    s.gate_revision == 0
                }
        }),
    }
}
/// Support spell targeting includes self and allies without weakening attack rules.
pub fn can_haste(o: &Observation, target: Target) -> bool {
    if o.data().haste_state().is_none() && o.data().speed_state().is_none() {
        return false;
    }
    match target {
        Target::Player {
            entity,
            life: wanted,
        } => {
            super::battle_space::occupies(o.data(), entity)
                && o.data().actors.0.iter().any(|a| {
                    a.entity == entity
                        && life(a) == wanted
                        && a.loaded
                        && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
                })
        }
        Target::Npc { .. } => target_current(o, target),
        _ => false,
    }
}
/// Slow can target living players (including self) and NPCs. Team/range policy
/// is still authoritatively checked by the server; no tower namespace fallback.
pub fn can_slow(o: &Observation, target: Target) -> bool {
    o.data().speed_state().is_some() && can_haste(o, target)
}
pub fn can_heal(o: &Observation, target: Target) -> bool {
    if let Target::Player {
        entity,
        life: wanted,
    } = target
    {
        // Server permits participants in the same instance, including self.
        // Quest lanes are separate instances; ordinary teams are not.
        return o.data().healing_actors().is_some()
            && super::battle_space::occupies(o.data(), entity)
            && o.data().actors.0.iter().any(|a| {
                a.entity == entity
                    && a.loaded
                    && life(a) == wanted
                    && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
            });
    }
    let Target::Honor { entity } = target else {
        return false;
    };
    target_current(o, target)
        && o.data()
            .honor
            .objectives
            .0
            .iter()
            .any(|a| a.entity == entity && Some(a.team) == team(o, o.data().personal.entity))
}
pub(crate) fn player_healing_outgoing(
    o: &Observation,
) -> Option<&super::battle_payload::healing::Outgoing> {
    o.data()
        .healing_actors()?
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)?
        .outgoing
        .as_ref()
}
pub fn validate(surface: &Surface) -> Result<()> {
    let d = surface.observation.data();
    ensure!(
        surface.width == 800 && surface.height == 600 && surface.bgra.len() == 800 * 600 * 4,
        "canonical pixel dimensions invalid"
    );
    ensure!(
        surface.geometry.world_id == d.world.identity.match_id as u64
            && surface.geometry.revision == d.world.revision as u64,
        "canonical pixel identity invalid"
    );
    ensure!(
        surface
            .navigation
            .as_ref()
            .is_none_or(|n| n.matches(&surface.observation)),
        "canonical navigation belongs to other pixels"
    );
    let mut ids = std::collections::BTreeSet::new();
    ensure!(
        surface
            .minimap
            .as_ref()
            .is_none_or(|m| m.matches(&surface.observation)
                && surface
                    .navigation
                    .as_ref()
                    .is_some_and(|n| n.matches(&surface.observation))),
        "canonical minimap belongs to other pixels"
    );
    for p in &surface.geometry.actors {
        ensure!(
            ids.insert(p.entity_id)
                && p.width > 0
                && p.width <= 4096
                && p.height > 0
                && p.height <= 4096
                && d.actors.0.iter().any(|a| a.entity as u64 == p.entity_id
                    && a.position.x == p.cell.x
                    && a.position.y == p.cell.y),
            "canonical actor geometry invalid"
        );
    }
    ids.clear();
    for p in &surface.geometry.npcs {
        ensure!(
            ids.insert(p.id.serial)
                && p.width > 0
                && p.width <= 4096
                && p.height > 0
                && p.height <= 4096
                && p.id.scope.room_id == d.world.identity.room as u64
                && p.id.scope.match_id == d.world.identity.match_id as u64
                && p.id.scope.generation == d.world.identity.generation as u64
                && d.npcs
                    .as_ref()
                    .is_some_and(|n| n.actors.0.iter().any(|a| a.serial as u64 == p.id.serial
                        && a.life as u64 == p.life
                        && a.cell.x == p.cell.x
                        && a.cell.y == p.cell.y
                        && a.graphic as u32 == p.graphics_id
                        && (a.hp > 0) == p.alive)),
            "canonical npc geometry invalid"
        );
    }
    ids.clear();
    let mut targets = Vec::new();
    ensure!(
        surface.geometry.objectives.len() == surface.objectives.len()
            && surface
                .geometry
                .objectives
                .iter()
                .zip(&surface.objectives)
                .all(|(a, (_, b))| a == b),
        "canonical objective paint and hit geometry differ"
    );
    for (target, p) in &surface.objectives {
        let cell = match target {
            ObjectiveTarget::Honor { entity, team } => d
                .honor
                .objectives
                .0
                .iter()
                .find(|a| a.entity == *entity && a.team == *team)
                .map(|a| &a.cell),
            ObjectiveTarget::Quest(id) => d.quest.as_ref().and_then(|q| {
                q.objectives
                    .0
                    .iter()
                    .find(|a| a.id == *id)
                    .map(|a| &a.target.cell)
            }),
            ObjectiveTarget::TimeAttack(scope) if *scope == d.world.identity => {
                d.time_attack.as_ref().map(|t| &t.tower.cell)
            }
            ObjectiveTarget::Siege { scope, slot } if *scope == d.world.identity => {
                d.siege.as_ref().and_then(|s| {
                    s.objectives
                        .0
                        .iter()
                        .find(|a| a.tower.slot == *slot)
                        .map(|a| &a.tower.cell)
                })
            }
            _ => None,
        };
        ensure!(
            ids.insert(p.entity_id)
                && !targets.contains(&target)
                && p.width > 0
                && p.width <= 4096
                && p.height > 0
                && p.height <= 4096
                && cell.is_some_and(|c| c.x == p.cell.x && c.y == p.cell.y),
            "canonical objective geometry invalid"
        );
        targets.push(target);
    }
    Ok(())
}
fn pixel(view: Viewport, point: PixelPoint) -> Option<PixelPoint> {
    let x = i64::from(point.x) - i64::from(view.x);
    let y = i64::from(point.y) - i64::from(view.y);
    if view.width <= 0
        || view.height <= 0
        || x < 0
        || y < 0
        || x >= i64::from(view.width)
        || y >= i64::from(view.height)
    {
        return None;
    }
    Some(PixelPoint {
        x: (x * 800 / i64::from(view.width)) as i32,
        y: (y * 600 / i64::from(view.height)) as i32,
    })
}
fn contains(
    point: PixelPoint,
    top: PixelPoint,
    width: u32,
    height: u32,
    mask: Option<&super::world_raster::DrawVisibility>,
) -> bool {
    let x = i64::from(point.x);
    let y = i64::from(point.y);
    x >= i64::from(top.x)
        && y >= i64::from(top.y)
        && x < i64::from(top.x) + i64::from(width)
        && y < i64::from(top.y) + i64::from(height)
        && mask.is_none_or(|m| m.contains(x, y))
}
pub fn hit(surface: &Surface, view: Viewport, point: PixelPoint, skill: Skill) -> Option<Hit> {
    let point = pixel(view, point)?;
    if surface.minimap.as_ref().is_some_and(|m| m.contains(point)) {
        return None; // The locator is an opaque non-interactive overlay, not terrain.
    }
    let o = &surface.observation;
    let d = o.data();
    let stamp = Stamp::of(o);
    let goal = surface.geometry.projection.pixel_to_cell(point)?;
    let mut hits = Vec::new();
    for p in &surface.geometry.actors {
        if !contains(point, p.top_left, p.width, p.height, p.visibility.as_ref()) {
            continue;
        }
        let a = d.actors.0.iter().find(|a| a.entity as u64 == p.entity_id)?;
        let member = d.room.members.0.get(usize::try_from(a.entity - 1).ok()?)?;
        let support_target = Target::Player {
            entity: a.entity,
            life: life(a),
        };
        let kind = if skill == Skill::Heal && can_heal(o, support_target) {
            Kind::Heal(support_target)
        } else if skill == Skill::Slow && can_slow(o, support_target) {
            Kind::Slow(support_target)
        } else if skill == Skill::Haste && can_haste(o, support_target) {
            Kind::Haste(support_target)
        } else if a.entity == stamp.entity && portal(o) {
            Kind::Portal
        } else {
            let t = Target::Player {
                entity: a.entity,
                life: life(a),
            };
            if target_current(o, t) && d.npcs.is_none() {
                Kind::Attack(t)
            } else {
                Kind::Cancel
            }
        };
        hits.push((
            (p.anchor.y, p.anchor.x, p.entity_id, 0),
            kind,
            Some((
                format!("{}({})", member.name.0, member.source_name.0),
                p.anchor,
            )),
        ));
    }
    for p in &surface.geometry.npcs {
        if !contains(point, p.top_left, p.width, p.height, p.visibility.as_ref()) {
            continue;
        }
        let npcs = d.npcs.as_ref()?;
        let a = npcs
            .actors
            .0
            .iter()
            .find(|a| a.serial as u64 == p.id.serial && a.life as u64 == p.life)?;
        let target = Target::Npc {
            serial: a.serial,
            life: a.life,
        };
        let name = npcs
            .names
            .0
            .iter()
            .find(|n| n.source == a.source && n.npc_id == a.npc_id && n.graphic == a.graphic)
            .map(|n| (n.name.0.clone(), p.anchor));
        hits.push((
            (p.anchor.y, p.anchor.x, p.id.serial, 1),
            if target_current(o, target) {
                Kind::Attack(target)
            } else {
                Kind::Cancel
            },
            name,
        ));
    }
    for (target, p) in &surface.objectives {
        if !contains(point, p.top_left, p.width, p.height, p.visibility.as_ref()) {
            continue;
        }
        let t = match target {
            ObjectiveTarget::Honor { entity, .. } => Target::Honor { entity: *entity },
            ObjectiveTarget::Quest(id) => Target::Quest {
                slot: id.slot,
                born: d
                    .quest
                    .as_ref()?
                    .objectives
                    .0
                    .iter()
                    .find(|a| a.id == *id)?
                    .born_revision,
            },
            ObjectiveTarget::TimeAttack(_) => Target::TimeTower,
            ObjectiveTarget::Siege { slot, .. } => Target::Siege { slot: *slot as u8 },
        };
        // Quest/time tower ruins stop occupying space. Siege ruins still occlude.
        if !target_current(o, t) && matches!(t, Target::Quest { .. } | Target::TimeTower) {
            continue;
        }
        hits.push((
            (p.anchor.y, p.anchor.x, p.entity_id, 0),
            if target_current(o, t) {
                Kind::Attack(t)
            } else {
                Kind::Cancel
            },
            None,
        ));
    }
    hits.sort_by_key(|h| h.0);
    let (_, mut kind, name) = hits
        .pop()
        .unwrap_or(((0, 0, 0, 0), Kind::Navigate(goal), None));
    kind = match (skill, kind) {
        (Skill::Melee, k) => k,
        (Skill::Haste, Kind::Haste(t) | Kind::Attack(t)) if can_haste(o, t) => Kind::Haste(t),
        (Skill::Slow, Kind::Slow(t) | Kind::Attack(t)) if can_slow(o, t) => Kind::Slow(t),
        (
            Skill::LightArrow,
            Kind::Attack(t @ (Target::Player { .. } | Target::Npc { .. } | Target::Honor { .. })),
        ) => Kind::LightArrow(t),
        (Skill::Disintegrate, Kind::Attack(t @ (Target::Player { .. } | Target::Npc { .. }))) => {
            Kind::Disintegrate(t)
        }
        (Skill::Heal, Kind::Heal(t) | Kind::Attack(t)) if can_heal(o, t) => Kind::Heal(t),
        _ => Kind::Cancel,
    };
    Some(Hit {
        intent: Intent { stamp, kind },
        name,
    })
}
pub fn health(o: &Observation) -> String {
    self_actor(o).and_then(|a| a.vital.as_ref()).map_or_else(
        || "本場未提供戰鬥 HP／MP".into(),
        |v| {
            format!(
                "HP {}/{}  MP {}/{}  AC {}",
                v.values.hp, v.values.maximum_hp, v.values.mp, v.values.maximum_mp, v.ac
            )
        },
    )
}
pub fn weight(o: &Observation) -> String {
    o.personal().weight().map_or_else(
        || "本場未提供比賽負重。".into(),
        |w| {
            format!(
                "比賽負重 {}／{:.3} · 階段 {}/240 · {}",
                w.total,
                w.maximum(),
                w.stage,
                if w.can_attack() {
                    "可攻擊／施法"
                } else {
                    "過重：可移動／喝藥，禁攻擊／施法"
                }
            )
        },
    )
}
pub fn buffs(o: &Observation) -> String {
    self_actor(o).and_then(|a| a.buffs.as_ref()).map_or_else(
        || "本場未提供限時效果。".into(),
        |b| {
            let seconds =
                |until: i64| until.saturating_sub(b.observed_nanos).max(0) / 1_000_000_000;
            format!(
                "伺服器確認：{} {}秒 · 勇敢 {}秒 · 慎重 {}秒{}",
                if b.haste_origin == d::HasteOrigin::GreaterHaste {
                    "強力加速術"
                } else {
                    "加速"
                },
                seconds(b.haste_until_nanos),
                seconds(b.brave_until_nanos),
                seconds(b.wisdom_until_nanos),
                if o.data().speed_state().is_some() {
                    format!(" · 緩速 {}秒", seconds(b.slow_until_nanos))
                } else {
                    String::new()
                }
            )
        },
    )
}
/// Current qualified lane cost, not the saved SQL template's MP or a prior
/// attachment's legacy combat panel. No local cooldown/MP prediction.
pub fn skill_status(o: &Observation, selected: Skill) -> Option<String> {
    let actor = self_actor(o)?;
    if selected == Skill::Heal {
        if let Some(h) = player_healing_outgoing(o) {
            return Some(format!(
                "初級治癒術（角色）：MP {} · 點選自身或角色 · 動作間隔與恢復量由伺服器確認",
                h.ledger.cost
            ));
        }
    }
    let npc = o.data().npcs.is_some();
    let (name, lane) = match selected {
        Skill::Disintegrate => (
            "究極光裂術",
            if npc {
                d::Lane::NpcDisintegrate
            } else {
                d::Lane::Disintegrate
            },
        ),
        Skill::LightArrow => (
            "光箭",
            if npc {
                d::Lane::NpcLightArrow
            } else {
                d::Lane::LightArrow
            },
        ),
        Skill::Heal => ("初級治癒術", d::Lane::Healing),
        Skill::Haste => ("加速術", d::Lane::Haste),
        Skill::Slow => ("緩速術（需魔法寶石）", d::Lane::Slow),
        // Keep the F shortcut visible when ordinary attacks are selected.
        Skill::Melee => (
            "火風暴 (F)",
            if npc {
                d::Lane::NpcFireStorm
            } else {
                d::Lane::FireStorm
            },
        ),
    };
    let cost = actor
        .spells
        .histories
        .0
        .iter()
        .find(|s| s.lane == lane)?
        .ledger
        .cost;
    Some(if selected == Skill::Disintegrate {
        format!(
            "{name}：MP {cost} · 正義值 {}（需500）",
            actor.spells.lawful
        )
    } else {
        format!("{name}：MP {cost} · 施放與消耗由伺服器確認")
    })
}
pub fn presentation_notice(surface: &Surface) -> Option<&'static str> {
    if surface.observation.data().vip.is_some() {
        Some("完整 VIP 地圖／角色預覽：戰鬥特效與資源資格尚未齊全，不能載入遊戲或操作。")
    } else if surface.effects_skipped {
        Some("部分特效超出呈現預算／可見範圍而略過；傷害、MP與勝敗仍依伺服器確認。")
    } else if surface.limitations.actor_object_occlusion_unimplemented
        || surface.limitations.actor_dependent_roofs_unimplemented
        || surface
            .limitations
            .weapon_and_transformation_art_unimplemented
    {
        Some("目前場景有遮擋或角色外觀的呈現限制；尚未完成視覺驗收。")
    } else {
        None
    }
}
/// Canonical v47 world result for this exact human/source/entity only. The
/// separate controller lifecycle reports persistence; a verdict alone never
/// claims a saved SQL record. Time-attack and honor have their own canonical
/// verdict HUD lines; race has ranks instead of a Win/Loss result.
pub fn result_notice(o: &Observation) -> Option<&'static str> {
    use super::battle_payload::data::{MemberKind, Phase, Verdict};
    let d = o.data();
    if d.world.phase != Phase::Finished || d.time_attack.is_some()
        || d.world.descriptor.ruleset == super::battle_payload::data::Ruleset::HonorTowers
        || d.race.is_some() { return None; }
    let player = &d.personal.player;
    let participant = d.result.as_ref()?.participants.0.iter().find(|p|
        p.entity == d.personal.entity
            && p.identity.kind == MemberKind::Human
            && p.identity.source == player.source
            && p.identity.id == player.object)?;
    Some(match participant.verdict {
        Verdict::Win => "本場勝利（伺服器判定；戰績保存狀態見下方）。",
        Verdict::Loss => "本場敗北（伺服器判定；戰績保存狀態見下方）。",
    })
}
pub fn hud(o: &Observation) -> Vec<String> {
    let d = o.data();
    let mut lines = Vec::new();
    lines.extend(honor_hud::lines(d));
    if let Some(q) = &d.quest {
        for lane in &q.lanes.0 {
            lines.push(format!(
                "隊{} · 第{}階段 · 存活怪物{}",
                lane.lane,
                lane.stage,
                lane.living.0.len()
            ));
        }
    }
    lines.extend(time_attack_hud::lines(d));
    if let Some(s) = &d.siege {
        lines.push(format!(
            "攻城 · {} · 剩餘目標{}",
            if s.gate_revision > 0 {
                "城門已開"
            } else {
                "城門封閉"
            },
            s.objectives.0.iter().filter(|a| a.hp > 0).count()
        ));
    }
    lines.extend(race_hud::lines(d));
    lines
}

#[cfg(test)]
#[path = "battle_window_tests.rs"]
pub(crate) mod tests;
