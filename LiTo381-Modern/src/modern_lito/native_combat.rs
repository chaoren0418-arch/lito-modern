//! Current formal melee envelopes. Damage markers never mutate authoritative HP.
use anyhow::{ensure, Result};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Feedback {
    #[default]
    Idle,
    Pending,
    Acknowledged,
    NoResponse,
    Interrupted,
}
impl Feedback {
    pub fn text(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Pending => "等待攻擊回覆…",
            Self::Acknowledged => "攻擊已回覆。",
            Self::NoResponse => "未收到攻擊回覆，可再試。",
            Self::Interrupted => "攻擊等待已中止。",
        }
    }
}
pub struct Pending {
    pub since: std::time::Instant,
    pub count: u64,
    pub target: u32,
    pub generation: u64,
}
/// One physical press owns a target and world. Cancellation requires a fresh press.
#[derive(Default)]
pub struct HeldAttack {
    intent: Option<AttackIntent>,
}
struct AttackIntent {
    target: super::native_skill_selection::Target,
    stamp: super::native_navigation::InputStamp,
    sent_count: Option<u64>,
}
impl HeldAttack {
    pub fn press(&mut self, target: u32, epoch: u64, world: &super::native_world::World) {
        self.intent = super::native_skill_selection::Target::capture(world, target).map(|target| {
            AttackIntent {
                target,
                stamp: super::native_navigation::InputStamp::capture(epoch,world),
                sent_count: None,
            }
        });
    }
    pub fn clear(&mut self) {
        self.intent = None;
    }
    pub fn active(&self) -> bool {
        self.intent.is_some()
    }
    pub fn ready(
        &mut self,
        epoch: u64,
        world: &super::native_world::World,
        feedback: Feedback,
        battle: bool,
        frame_ready: bool,
    ) -> Option<u32> {
        let intent = self.intent.as_ref()?;
        if !battle
            || intent.stamp != super::native_navigation::InputStamp::capture(epoch,world)
            || intent.target.resolve(world).is_none()
            || world.attack_request(intent.target.id).is_err()
            || (intent.sent_count.is_some()
                && matches!(feedback, Feedback::NoResponse | Feedback::Interrupted))
        {
            self.clear();
            return None;
        }
        if !frame_ready || feedback == Feedback::Pending {
            return None;
        }
        if intent
            .sent_count
            .is_some_and(|count| world.own_attack_packets <= count)
        {
            return None;
        }
        Some(intent.target.id)
    }
    pub fn sent(&mut self, world: &super::native_world::World) {
        if let Some(intent) = &mut self.intent {
            intent.sent_count = Some(world.own_attack_packets);
        }
    }
}
impl Pending {
    pub fn observe(&self, world: &super::native_world::World, now: std::time::Instant) -> Feedback {
        if world.generation != self.generation {
            return Feedback::Interrupted;
        }
        if world.own_attack_packets > self.count {
            return Feedback::Acknowledged;
        }
        if world.battle_result.is_some()
            || world.own_actor().is_none_or(|a| a.dead)
            || !world.actors.contains_key(&self.target)
        {
            return Feedback::Interrupted;
        }
        if now.saturating_duration_since(self.since) >= std::time::Duration::from_secs(2) {
            Feedback::NoResponse
        } else {
            Feedback::Pending
        }
    }
}
pub fn target_label(actor: &super::native_world::Actor) -> String {
    target_label_with_ratio(actor, actor.hp_ratio)
}
fn target_label_with_ratio(actor: &super::native_world::Actor, hp_ratio: u8) -> String {
    if super::native_obstacles::destroyed_guardian(actor){return format!("{}（已摧毀）",actor.name);}
    if super::native_doors::is_door(actor) {
        return if actor.walk_action==28 {"門（開啟）"}else{"門（關閉）"}.into();
    }
    if actor.dead {
        format!("{}（已死亡）", actor.name)
    } else if hp_ratio <= 100 {
        format!("{}（{}%）", actor.name, hp_ratio)
    } else {
        format!("{}（HP 未知）", actor.name)
    }
}

/// HUD values captured with the rendered animation/effect frame. Network
/// authority can advance while the resource worker still draws an older frame.
#[derive(Clone, Debug)]
pub struct FrameVitals {
    pub hp: Option<[u16; 2]>,
    pub mp: Option<[u16; 2]>,
    generation: u64,
    targets: std::collections::BTreeMap<u32, (u64, String)>,
}
impl FrameVitals {
    pub fn capture(world: &super::native_world::World) -> Self {
        Self {
            hp: world.hp,
            mp: world.mp,
            generation: world.generation,
            targets: world.actors.values()
                .map(|actor| (actor.id, (actor.incarnation, target_label(actor))))
                .collect(),
        }
    }

    pub fn target_label(&self, world: &super::native_world::World, id: u32) -> Option<&str> {
        let actor = world.actors.get(&id)?;
        let (incarnation, label) = self.targets.get(&id)?;
        (self.generation == world.generation && *incarnation == actor.incarnation)
            .then_some(label.as_str())
    }
}

/// Presentation-side HP/MP. The world keeps the newest authoritative values;
/// this gate commits one only after the animator has started every action the
/// actor had received when that value arrived. An actor whose pose failed has
/// no animation to wait for, so its value commits directly.
#[derive(Default)]
pub struct VitalsGate {
    generation: Option<u64>,
    hp: Option<[u16; 2]>,
    mp: Option<[u16; 2]>,
    own: Option<(u32, u64)>,
    ratios: std::collections::BTreeMap<u32, (u64, u8)>,
    held: u64,
}
impl VitalsGate {
    /// Number of value/frame pairs held behind unplayed actions so far.
    pub fn held(&self) -> u64 { self.held }

    pub fn present(
        &mut self,
        world: &super::native_world::World,
        started: impl Fn(&super::native_world::Actor, u64) -> bool,
    ) -> FrameVitals {
        if self.generation != Some(world.generation) {
            *self = Self { generation: Some(world.generation), held: self.held, ..Self::default() };
        }
        let own = world.own_actor();
        let own_key = own.map(|a| (a.id, a.incarnation));
        if self.own != own_key {
            // A different local object has no older presentation to retain.
            self.own = own_key;
            self.hp = world.hp;
            self.mp = world.mp;
        }
        let ready = |barrier: u64| own.is_none_or(|a| started(a, barrier));
        if self.hp != world.hp {
            if self.hp.is_none() || ready(world.hp_barrier) { self.hp = world.hp; } else { self.held += 1; }
        }
        if self.mp != world.mp {
            if self.mp.is_none() || ready(world.mp_barrier) { self.mp = world.mp; } else { self.held += 1; }
        }
        self.ratios.retain(|id, (incarnation, _)| {
            world.actors.get(id).is_some_and(|a| a.incarnation == *incarnation)
        });
        let mut targets = std::collections::BTreeMap::new();
        for actor in world.actors.values() {
            let barrier = world.hp_ratio_barriers.get(&actor.id)
                .filter(|(incarnation, _)| *incarnation == actor.incarnation)
                .map_or(0, |(_, barrier)| *barrier);
            let entry = self.ratios.entry(actor.id).or_insert((actor.incarnation, actor.hp_ratio));
            if entry.1 != actor.hp_ratio {
                if started(actor, barrier) { entry.1 = actor.hp_ratio; } else { self.held += 1; }
            }
            targets.insert(actor.id, (actor.incarnation, target_label_with_ratio(actor, entry.1)));
        }
        FrameVitals { hp: self.hp, mp: self.mp, generation: world.generation, targets }
    }
}
pub fn score_label(score: &super::native_world::Score) -> String {
    format!(
        "{}　分數 {}　擊殺 {}　死亡 {}　助攻 {}　隊伍 {}",
        score.name,
        score.points,
        score.kills,
        score.deaths,
        score.assists,
        u16::from(score.force[0]) + 1
    )
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ActionCue {
    pub sequence: u64,
    pub action: u8,
    pub source: &'static str,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Attack {
    pub action: u8,
    pub attacker: u32,
    pub target: u32,
    pub damage_marker: u8,
    pub heading: u8,
    pub projectile: Option<Projectile>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Projectile {
    pub id: u32,
    pub flight_type: u8,
    pub graphics: u16,
    pub from: [u16; 2],
    pub to: [u16; 2],
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProjectileCue {
    pub heading: u8,
    pub attacker_incarnation: u64,
    pub target_incarnation: Option<u64>,
    pub action_sequence: u64,
    pub attacker: u32,
    pub target: u32,
    pub damage_marker: u8,
    pub projectile: Projectile,
}
pub fn decode_attack(b: &[u8]) -> Result<Option<Attack>> {
    ensure!(
        b.first() == Some(&0x23) && b.len() >= 16,
        "truncated attack header"
    );
    let mut a = Attack {
        action: b[1],
        attacker: u32::from_le_bytes(b[2..6].try_into()?),
        target: u32::from_le_bytes(b[6..10].try_into()?),
        damage_marker: b[10],
        heading: b[11],
        projectile: None,
    };
    ensure!(
        a.attacker != 0 && a.heading < 8,
        "invalid attack identity/heading"
    );
    let projectile_id = u32::from_le_bytes(b[12..16].try_into()?);
    if projectile_id != 0 {
        ensure!(b.len() >= 19, "truncated attack effect header");
        // Other native effect branches remain explicitly unhandled.
        if !matches!(b[18], 0 | 0x7f) {
            return Ok(None);
        }
        ensure!(
            b.len() == 29 && b[27..29] == [0, 0],
            "invalid ranged attack tail"
        );
        let h = |at| u16::from_le_bytes([b[at], b[at + 1]]);
        let graphics = h(16);
        ensure!(
            graphics != 0 && (a.target != 0 || a.damage_marker == 0),
            "invalid ranged attack fields"
        );
        a.projectile = Some(Projectile {
            id: projectile_id,
            flight_type: b[18],
            graphics,
            from: [h(19), h(21)],
            to: [h(23), h(25)],
        });
        return Ok(Some(a));
    }
    ensure!(
        b.len() == 16 && a.target != 0 && a.target != a.attacker,
        "invalid melee attack layout"
    );
    Ok(Some(a))
}
pub fn request(target: u32, x: u16, y: u16) -> Result<Vec<u8>> {
    attack_request(target, x, y, false)
}
pub fn attack_request(target: u32, x: u16, y: u16, ranged: bool) -> Result<Vec<u8>> {
    ensure!(target != 0, "zero attack target");
    let mut b = vec![if ranged { 0x18 } else { 0x17 }];
    b.extend(target.to_le_bytes());
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    Ok(b)
}
