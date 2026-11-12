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
    /// A server S_MESSAGE_CODE that the formal attack path sends instead of S23.
    Rejected(u16),
    /// Local preflight refused the request; nothing was sent.
    Refused(super::native_combat_ops::Refusal),
}
impl Feedback {
    pub fn text(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Pending => "等待攻擊回覆…",
            Self::Acknowledged => "攻擊已回覆。",
            Self::NoResponse => "未收到攻擊回覆，可再試。",
            Self::Interrupted => "攻擊等待已中止。",
            Self::Rejected(code) => rejection_text(code),
            Self::Refused(refusal) => refusal.text(),
        }
    }
}
/// Formal ClientSession.attack / EmptyMelee send code 110 when weight240 is at
/// or above BattleInventoryWeight.BUFFED_REGEN_LIMIT (197), with no S23.
pub const MESSAGE_ATTACK_OVERWEIGHT: u16 = 110;
pub fn rejection_text(code: u16) -> &'static str {
    match code {
        MESSAGE_ATTACK_OVERWEIGHT => "負重過重，無法攻擊。",
        _ => "伺服器拒絕了這次攻擊。",
    }
}
/// Retained own-attack replies; older entries are dropped, never guessed.
pub const REPLY_LOG: usize = 16;
pub struct Pending {
    pub since: std::time::Instant,
    pub count: u64,
    /// Zero for the native empty swing (`17 00 00 00 00 x y`).
    pub target: u32,
    pub generation: u64,
    /// world.message_count when the request was committed.
    pub messages: u64,
    /// Object lives at send time: `(own id, own incarnation)` and the target's
    /// incarnation. Removal plus same-ID re-entry (Return-to-Nature release,
    /// rental portal rebuild, corpse re-put) is a different life, like S11.
    pub attacker: Option<(u32, u64)>,
    pub target_incarnation: Option<u64>,
}
/// Counts own S23 replies that did not acknowledge the request pending at
/// that moment (late replies to an earlier, timed-out or other target).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Ledger {
    pub seen: u64,
    pub late: u64,
    pub acknowledged: u64,
}
impl Ledger {
    /// Consume newly logged own replies. Returns how many were late.
    pub fn observe(&mut self, world: &super::native_world::World, pending: Option<&Pending>) -> u64 {
        if world.own_attack_packets < self.seen {
            // World was replaced; start over without counting history as late.
            self.seen = world.own_attack_packets;
            return 0;
        }
        let mut late = 0;
        let mut acknowledged = false;
        // Entries evicted from the bounded log are counted as late as well:
        // without a target they cannot prove acknowledgement.
        let logged: u64 = world.own_attack_log.iter().filter(|(seq, _)| *seq > self.seen).count() as u64;
        let total = world.own_attack_packets - self.seen;
        late += total - logged.min(total);
        for (seq, target) in world.own_attack_log.iter().filter(|(seq, _)| *seq > self.seen) {
            let matches = pending.is_some_and(|p| *seq > p.count && *target == p.target
                && world.generation == p.generation);
            if matches && !acknowledged { acknowledged = true; } else { late += 1; }
        }
        self.seen = world.own_attack_packets;
        self.late += late;
        if acknowledged { self.acknowledged += 1; }
        late
    }
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
    /// Mouse-started attacks may walk toward an out-of-reach target.
    chase: bool,
    chase_steps: u32,
}
impl AttackIntent {
    /// A terminal reply to a request this intent sent ends the press. A stale
    /// terminal feedback from before the press does not.
    fn ended_by(&self, feedback: Feedback) -> bool {
        self.sent_count.is_some()
            && matches!(feedback, Feedback::NoResponse | Feedback::Interrupted | Feedback::Rejected(_) | Feedback::Refused(_))
    }
}
impl HeldAttack {
    pub fn press(&mut self, target: u32, epoch: u64, world: &super::native_world::World) {
        self.press_with(target, epoch, world, false);
    }
    /// Like `press`, but an out-of-reach target keeps the intent and
    /// `chase_heading` proposes ordinary checked steps toward it.
    pub fn press_chase(&mut self, target: u32, epoch: u64, world: &super::native_world::World) {
        self.press_with(target, epoch, world, true);
    }
    fn press_with(&mut self, target: u32, epoch: u64, world: &super::native_world::World, chase: bool) {
        self.intent = super::native_skill_selection::Target::capture(world, target).map(|target| {
            AttackIntent {
                target,
                stamp: super::native_navigation::InputStamp::capture(epoch,world),
                sent_count: None,
                chase,
                chase_steps: 0,
            }
        });
    }
    pub fn chasing(&self) -> bool {
        self.intent.as_ref().is_some_and(|i| i.chase)
    }
    /// Next heading toward the held target, only while every other attack gate
    /// passes and the target is beyond reach. The step itself must still go
    /// through the controller's checked/presented movement path.
    pub fn chase_heading(
        &mut self,
        epoch: u64,
        world: &super::native_world::World,
        navigation: &super::native_navigation::Navigation,
        feedback: Feedback,
    ) -> Option<u8> {
        let intent = self.intent.as_ref().filter(|i| i.chase)?;
        if feedback == Feedback::Pending || intent.ended_by(feedback)
            || intent.stamp != super::native_navigation::InputStamp::capture(epoch, world)
            || intent.target.resolve(world).is_none()
            || !world.attack_out_of_reach(intent.target.id)
        {
            return None;
        }
        if intent.chase_steps >= super::native_combat_ops::CHASE_STEP_LIMIT {
            self.clear();
            return None;
        }
        let own = world.own_actor()?;
        let other = world.actors.get(&intent.target.id)?;
        let reach = world.attack_reach(intent.target.id)?.1;
        match super::native_combat_ops::chase_step([own.x, own.y], [other.x, other.y], reach, |x, y, h| {
            navigation.can_step(x, y, h) && super::native_world::destination(own.x, own.y, h)
                .is_ok_and(|(nx, ny)| !super::native_obstacles::blocks(world, nx, ny))
        }) {
            super::native_combat_ops::ChaseStep::Step(heading) => Some(heading),
            _ => {
                self.clear();
                None
            }
        }
    }
    /// The checked step was accepted. Server corrections, locks or a new
    /// world still change the stamp and retire the chase.
    pub fn chase_sent(&mut self) {
        if let Some(intent) = &mut self.intent {
            intent.chase_steps += 1;
        }
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
        if battle && intent.chase && !intent.ended_by(feedback)
            && intent.stamp == super::native_navigation::InputStamp::capture(epoch,world)
            && intent.target.resolve(world).is_some()
            && world.attack_out_of_reach(intent.target.id)
        {
            // Still approaching; chase_heading owns this state.
            return None;
        }
        if !battle
            || intent.stamp != super::native_navigation::InputStamp::capture(epoch,world)
            || intent.target.resolve(world).is_none()
            || world.attack_request(intent.target.id).is_err()
            || intent.ended_by(feedback)
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
    pub fn capture(world: &super::native_world::World, target: u32, since: std::time::Instant) -> Self {
        Self {
            since,
            count: world.own_attack_packets,
            target,
            generation: world.generation,
            messages: world.message_count,
            attacker: world.own_actor().map(|a| (a.id, a.incarnation)),
            target_incarnation: world.actors.get(&target).map(|a| a.incarnation),
        }
    }
    /// Alias of `capture`.
    pub fn new(world: &super::native_world::World, target: u32, since: std::time::Instant) -> Self {
        Self::capture(world, target, since)
    }
    /// True when an own reply after this request names the requested target.
    /// Own replies naming another target are late replies to older requests.
    pub fn acknowledged_by(&self, world: &super::native_world::World) -> bool {
        world.own_attack_log.iter().any(|(seq, target)| *seq > self.count && *target == self.target)
    }
    pub fn observe(&self, world: &super::native_world::World, now: std::time::Instant) -> Feedback {
        if world.generation != self.generation {
            return Feedback::Interrupted;
        }
        if world.own_attack_packets > self.count && self.acknowledged_by(world) {
            return Feedback::Acknowledged;
        }
        if world.message_count > self.messages {
            if let Some((code, _)) = &world.last_message {
                if *code == MESSAGE_ATTACK_OVERWEIGHT {
                    return Feedback::Rejected(*code);
                }
            }
        }
        if world.battle_result.is_some()
            || world.own_actor().is_none_or(|a| a.dead
                || self.attacker.is_some_and(|life| life != (a.id, a.incarnation)))
            // Target 0 is the empty swing: there is no target life to lose.
            || (self.target != 0 && world.actors.get(&self.target).is_none_or(|a|
                self.target_incarnation.is_some_and(|life| life != a.incarnation)))
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
    if super::native_obstacles::destroyed_guardian(actor){return format!("{}（已摧毀）",actor.name);}
    if super::native_doors::is_door(actor) {
        return if actor.walk_action==28 {"門（開啟）"}else{"門（關閉）"}.into();
    }
    if actor.dead {
        format!("{}（已死亡）", actor.name)
    } else if actor.hp_ratio <= 100 {
        format!("{}（{}%）", actor.name, actor.hp_ratio)
    } else {
        format!("{}（HP 未知）", actor.name)
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
    pub retain_on_arrival: u8,
    /// Original bounded, NUL-terminated MS950 bytes, without the terminator.
    pub name: Vec<u8>,
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
        // Native 4A5ED0 reads hchhhhcs for every nonzero projectile ID.
        // The byte is passed to the effect constructor, while list.spr's
        // attribute 109 selects its class. It is not a packet discriminator.
        ensure!((29..=156).contains(&b.len()), "invalid ranged attack tail length");
        ensure!(b.last() == Some(&0) && !b[28..b.len()-1].contains(&0),
            "invalid ranged attack name terminator");
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
            retain_on_arrival: b[27],
            name: b[28..b.len()-1].to_vec(),
        });
        return Ok(Some(a));
    }
    // Formal EmptyMelee broadcasts `23 01 <attacker> 00000000 00 <heading>
    // 00000000` for the native target-zero swing; the swing carries no damage.
    ensure!(
        b.len() == 16 && a.target != a.attacker && (a.target != 0 || a.damage_marker == 0),
        "invalid melee attack layout"
    );
    Ok(Some(a))
}
/// Native target-zero melee: `17 00 00 00 00 <x u16> <y u16>`, e.g. the
/// recorded `17 00 00 00 00 DE 7F 1D 80` (x 0x7FDE, y 0x801D).
pub fn empty_request(x: u16, y: u16) -> Vec<u8> {
    let mut b = vec![0x17, 0, 0, 0, 0];
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    b
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
