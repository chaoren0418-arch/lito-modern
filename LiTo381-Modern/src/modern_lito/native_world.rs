//! Current formal world state. Object identity comes from S_STATUS/S_PUT_OBJECT.
use super::{native_city_runtime::status_receipt, native_login::decode_text};
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
pub fn destination(x: u16, y: u16, heading: u8) -> Result<(u16, u16)> {
    const DIRECTIONS: [(i32, i32); 8] = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];
    let (dx, dy) = *DIRECTIONS
        .get(usize::from(heading))
        .context("heading outside 0..7")?;
    Ok((
        u16::try_from(i32::from(x) + dx).context("move X boundary")?,
        u16::try_from(i32::from(y) + dy).context("move Y boundary")?,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Actor {
    /// Object/life revision, renewed by S_PUT_OBJECT and S_RESURRECT.
    pub incarnation: u64,
    /// Invalidates retained selection when a remote actor becomes invisible.
    pub target_revision: u64,
    pub action_sequence: u64,
    pub actions: Vec<super::native_combat::ActionCue>,
    pub dead: bool,
    pub motion_revision: u64,
    pub motion_from: Option<[u16; 2]>,
    pub position_predicted: bool,
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub graphics: u16,
    pub walk_action: u8,
    pub heading: u8,
    pub light: u8,
    pub speed: u8,
    pub speed_seconds: u16,
    pub brave: u8,
    pub brave_seconds: u16,
    pub experience: u32,
    pub lawful: i16,
    pub name: String,
    pub title: String,
    pub status: u8,
    pub draw_state: super::native_actor_states::State,
    pub emblem: u32,
    pub clan: String,
    /// Native FA writes actor+0x13a. Marker rendering/input semantics are separate.
    pub clan_marker_active: bool,
    pub master: String,
    pub unknown: u8,
    pub hp_ratio: u8,
    pub companion: u8,
    pub level: u8,
    pub auxiliary_name: String,
    pub tail_word: u16,
    pub tail_byte: u8,
    /// Present exactly when S_PUT_OBJECT status bit 0x04 is set.
    /// Preserve the deployed five-byte extension without inventing semantics.
    pub status_tail: Option<[u8; 5]>,
    pub relations: super::native_actor_relations::Relations,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct World {
    /// GAME-owned clan context survives scene replacement, not reconnection.
    pub relations: super::native_actor_relations::Context,
    /// Role comes from S_E5, never inferred from the bounds sent to all peers.
    pub observer_mode: bool,
    /// Local camera only; no actor or authoritative player coordinates are created.
    pub observer_camera: Option<[u16;2]>,
    pub portal:Option<super::native_portal::Pending>,
    pub completed_portals:u64,
    pub resurrection_packets: u64,
    pub last_resurrection: Option<Resurrection>,
    /// Server-supplied half-open observer area. This does not enable observer mode
    /// or impose spectator rules on ordinary player movement.
    pub observer_bounds: Option<ObserverBounds>,
    pub door_attributes: BTreeMap<u64, super::native_doors::Record>,
    pub emblems: super::native_emblems::Cache,
    pub learned_skills: BTreeSet<u16>,
    pub effects: Vec<super::native_skills::Effect>,
    pub effects_dropped: u64,
    /// Direct S4A cues, scoped to this world. No inferred spell or expiry.
    pub wire_sounds: Vec<WireSound>,
    pub wire_sounds_dropped: u64,
    pub wire_sound_sequence: u64,
    pub own_cast_packets: u64,
    pub message_count: u64,
    pub system_message_count: u64,
    pub last_system_message: Option<(u8,String)>,
    /// Received SP/MR modifiers, not inferred total spell power or resistance.
    pub magic_modifiers: Option<[i8;2]>,
    /// Ability, experience, armor/resistance, and magic updates for the current local identity.
    pub attribute_updates: [u64;4],
    /// Last active server ability receipts for this local identity; no inferred expiry.
    pub ability_buffs: BTreeMap<u8,super::native_ability_buffs::Update>,
    pub last_message: Option<(u16, Vec<String>)>,
    pub last_skill_rejection: Option<(u64, u16)>,
    pub death_confirmation_pending: bool,
    pub death_confirmations_sent: u64,
    pub last_death_confirmed_action: Option<u64>,
    pub battle_result: Option<i32>,
    pub scores: Vec<Score>,
    pub attack_packets: u64,
    pub own_attack_packets: u64,
    pub last_attack: Option<super::native_combat::Attack>,
    pub projectiles: Vec<super::native_combat::ProjectileCue>,
    pub many_attacks: Vec<super::native_attack_many::Cue>,
    pub many_attacks_dropped:u64,
    pub projectile_cues_dropped: u64,
    pub movement_blocks: BTreeSet<u8>,
    pub control_interruptions: u64,
    pub map_id: Option<u16>,
    pub underwater: u8,
    pub started: bool,
    pub generation: u64,
    pub revision: u64,
    pub visual_revision: u64,
    pub predicted_moves: u64,
    pub local_corrections: u64,
    pub movement_locks: u64,
    pub orphan_updates: u64,
    pub actors: BTreeMap<u32, Actor>,
    pub status: Option<status_receipt::Status>,
    pub time_limit: Option<u32>,
    pub game_time: Option<u32>,
    pub hp: Option<[u16; 2]>,
    pub mp: Option<[u16; 2]>,
    pub integrity_challenge: Option<[u32; 3]>,
    pub minimap: super::native_minimap::State,
    pub unhandled_opcodes: BTreeSet<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Resurrection {
    pub target: u32,
    pub action: u8,
    pub source: u32,
    pub graphics: u16,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct WireSound {
    pub sequence: u64,
    pub kind: u8,
    pub sound: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ObserverBounds {
    pub min: [u16; 2],
    pub max_exclusive: [u16; 2],
}
impl ObserverBounds {
    pub fn contains(self, x: u16, y: u16) -> bool {
        x >= self.min[0] && y >= self.min[1]
            && x < self.max_exclusive[0] && y < self.max_exclusive[1]
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Score {
    pub name: String,
    pub points: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub force: [u8; 2],
}
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).context("world offset")?;
        let b = self.b.get(self.at..end).context("truncated world field")?;
        self.at = end;
        Ok(b)
    }
    fn c(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn h(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn d(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn s(&mut self) -> Result<String> {
        self.s_capacity(256)
    }
    fn s_raw(&mut self) -> Result<(String,Vec<u8>)> {
        let start=self.at;
        let text=self.s()?;
        Ok((text,self.b[start..self.at-1].to_vec()))
    }
    /// The original consumes the entire wire string but copies capacity-1 bytes.
    /// Reject malformed MS950 and a truncated final lead byte atomically.
    fn s_capacity(&mut self, capacity: usize) -> Result<String> {
        let n = self.b[self.at..]
            .iter()
            .take(256)
            .position(|v| *v == 0)
            .context("world string bound")?;
        let raw = &self.take(n + 1)?[..n];
        if raw.is_empty() {
            Ok(String::new())
        } else {
            decode_text(raw)?;
            decode_text(&raw[..raw.len().min(capacity - 1)])
        }
    }
    fn end(&self) -> Result<()> {
        ensure!(self.at == self.b.len(), "trailing world fields");
        Ok(())
    }
}
pub fn actor(b: &[u8]) -> Result<Actor> {
    ensure!(
        b.first() == Some(&0x0b) && b.len() <= 1400,
        "world actor envelope"
    );
    let mut r = Reader { b, at: 1 };
    let mut relations=super::native_actor_relations::Relations::default();
    let mut a = Actor {
        incarnation: 0,
        target_revision: 0,
        action_sequence: 0,
        actions: Vec::new(),
        dead: false,
        motion_revision: 0,
        motion_from: None,
        position_predicted: false,
        x: r.h()?,
        y: r.h()?,
        id: r.d()?,
        graphics: r.h()?,
        walk_action: r.c()?,
        heading: r.c()?,
        light: r.c()?,
        speed: r.c()?,
        speed_seconds: 0,
        brave: 0,
        brave_seconds: 0,
        experience: r.d()?,
        lawful: r.h()? as i16,
        name: {let (text,raw)=r.s_raw()?; relations.name=raw; text},
        title: r.s()?,
        status: r.c()?,
        draw_state: Default::default(),
        emblem: r.d()?,
        clan: {let (text,raw)=r.s_raw()?; relations.clan=raw; text},
        clan_marker_active: false,
        master: {let (text,raw)=r.s_raw()?; relations.master=raw; text},
        unknown: r.c()?,
        hp_ratio: r.c()?,
        companion: r.c()?,
        level: r.c()?,
        auxiliary_name: r.s()?,
        tail_word: r.h()?,
        tail_byte: r.c()?,
        status_tail: None,
        relations,
    };
    if a.status & 4 != 0 {
        a.status_tail = Some(r.take(5)?.try_into()?);
    }
    r.end()?;
    ensure!(
        a.id != 0 && a.heading < 8 && a.speed <= 2,
        "world actor domain"
    );
    a.draw_state = super::native_actor_states::State::from_status(a.status);
    a.relations.flag_1f.reset(a.status&4!=0);
    Ok(a)
}
impl World {
    pub fn set_observer_mode(&mut self, enabled:bool) {
        if self.observer_mode!=enabled {self.visual_revision+=1;}
        self.observer_mode=enabled;
        if !enabled {self.observer_camera=None;}
        self.refresh_observer_camera();
    }
    fn refresh_observer_camera(&mut self) {
        if !self.observer_mode {return;}
        if let Some(b)=&self.observer_bounds {
            let contains=|p:[u16;2]|(0..2).all(|i|p[i]>=b.min[i]&&p[i]<b.max_exclusive[i]);
            if self.observer_camera.is_none_or(|p|!contains(p)) {
                self.observer_camera=Some(std::array::from_fn(|i|b.min[i]+(b.max_exclusive[i]-b.min[i])/2));
            }
        }
    }
    pub fn view_ready(&self)->bool {
        if !self.observer_mode {return self.ready();}
        self.started&&self.portal.is_none()&&self.map_id.is_some()&&self.status.is_some()
            &&self.observer_bounds.as_ref().zip(self.observer_camera).is_some_and(|(b,p)|(0..2).all(|i|p[i]>=b.min[i]&&p[i]<b.max_exclusive[i]))
    }
    pub fn pan_observer(&mut self,heading:u8)->Result<bool> {
        ensure!(self.observer_mode&&self.view_ready(),"observer camera unavailable");
        let [x,y]=self.observer_camera.unwrap();let (x,y)=destination(x,y,heading)?;
        let b=self.observer_bounds.as_ref().unwrap();
        if x<b.min[0]||x>=b.max_exclusive[0]||y<b.min[1]||y>=b.max_exclusive[1] {return Ok(false);}
        self.observer_camera=Some([x,y]);self.visual_revision+=1;Ok(true)
    }
    /// Empty authority without reusing the identity of any previously rendered
    /// world. The next map packet advances this generation, including reconnects.
    pub fn clear_preserving_generation(&mut self) {
        *self = Self { generation: self.generation,
            relations:std::mem::take(&mut self.relations), ..Default::default() };
    }
    /// Ordinary view: see-invisible/GM perception is not yet a qualified mode.
    pub fn actor_visible(&self, id: u32) -> bool {
        self.actors.get(&id).is_some_and(|a| {
            a.draw_state
                .visible(self.status.is_some_and(|s| s.object_id == id))
        })
    }

    pub fn attack_request(&self, target: u32) -> Result<Vec<u8>> {
        ensure!(self.battle_result.is_none(), "battle has ended");
        ensure!(
            self.ready() && self.hp.is_some_and(|v| v[0] > 0) && self.movement_blocks.is_empty(),
            "local actor cannot attack"
        );
        let own = self.own_actor().context("local actor absent")?;
        let other = self.actors.get(&target).context("attack target absent")?;
        ensure!(
            target != own.id && !own.dead && !other.dead && !super::native_obstacles::destroyed_guardian(other) && self.actor_visible(target),
            "invalid/dead attack target"
        );
        let ranged = matches!(own.walk_action, 20 | 62);
        // Actor stance identifies the envelope. Exact weapon range and ammo are
        // server-owned until inventory/equipment presentation is connected.
        ensure!(
            own.x.abs_diff(other.x).max(own.y.abs_diff(other.y)) <= if ranged { 19 } else { 1 },
            "attack target outside presentation range"
        );
        super::native_combat::attack_request(target, other.x, other.y, ranged)
    }
    fn action(&mut self, id: u32, action: u8, source: &'static str) {
        if let Some(a) = self.actors.get_mut(&id) {
            if action == 8 { a.draw_state.native_light_dead = true; }
            // Guardian damage uses persistent S_ACTION 32..35 states, not
            // the character hurt/action queue. Repeated hits must not reset
            // the object clock or attempt the absent character action 2.
            if super::native_obstacles::guardian(a) {return;}
            if a.dead && action != 8 {
                return;
            }
            a.action_sequence = self.visual_revision + 1;
            a.motion_from = None;
            a.motion_revision = self.visual_revision + 1;
            if action == 8 {
                a.dead = true;
                a.actions.clear();
            }
            // Coalesce before the bounded history evicts an unsampled cast.
            // Retain the newest sequence so a later hurt can still enter after
            // playback has completed the previous one. Authority is unchanged.
            if action == 2 && a.actions.last().is_some_and(|cue| cue.action == 2) {
                *a.actions.last_mut().unwrap() = super::native_combat::ActionCue {
                    sequence: a.action_sequence,
                    action,
                    source,
                };
                return;
            }
            if a.actions.len() == 16 {
                a.actions.remove(0);
            }
            a.actions.push(super::native_combat::ActionCue {
                sequence: a.action_sequence,
                action,
                source,
            });
        } else {
            self.orphan_updates += 1;
        }
    }

    pub fn move_request(&self, heading: u8) -> Result<Vec<u8>> {
        ensure!(self.battle_result.is_none(), "battle has ended");
        ensure!(
            self.ready() && self.hp.is_some_and(|hp| hp[0] > 0),
            "local actor cannot move"
        );
        ensure!(
            self.movement_blocks.is_empty(),
            "movement restricted by server state"
        );
        let a = self.own_actor().context("local actor missing")?;
        ensure!(!a.dead, "dead actor cannot move");
        let (x,y)=destination(a.x, a.y, heading)?;
        ensure!(!super::native_obstacles::blocks(self,x,y),"門或守護塔擋住去路");
        let mut b = vec![0x0a];
        b.extend(a.x.to_le_bytes());
        b.extend(a.y.to_le_bytes());
        b.push(heading);
        Ok(b)
    }
    /// Accepted Tournament movement has no self echo. Keep prediction explicit
    /// until an authoritative object/position packet replaces it.
    pub fn predict_step(&mut self, heading: u8) -> Result<()> {
        self.move_request(heading)?;
        let id = self.status.context("local status")?.object_id;
        let a = self.actors.get_mut(&id).context("local actor")?;
        let (x, y) = destination(a.x, a.y, heading)?;
        a.motion_from = Some([a.x, a.y]);
        a.motion_revision = self.visual_revision + 1;
        a.x = x;
        a.y = y;
        a.heading = heading;
        a.position_predicted = true;
        self.predicted_moves += 1;
        self.visual_revision += 1;
        self.revision += 1;
        Ok(())
    }
    pub fn own_actor(&self) -> Option<&Actor> {
        self.status
            .as_ref()
            .and_then(|s| self.actors.get(&s.object_id))
    }
    pub fn ready(&self) -> bool {
        !self.observer_mode && self.started && self.portal.is_none() && self.own_actor().is_some()
    }
    pub fn finish_portal_if_ready(&mut self)->Result<bool>{
        let Some(pending)=self.portal else{return Ok(false)};
        let Some(actor)=self.own_actor()else{return Ok(false)};
        ensure!(actor.id==pending.object_id&&[actor.x,actor.y]==[pending.destination.x,pending.destination.y]&&self.map_id==Some(pending.destination.map),"portal rebuilt a different local identity/destination");
        self.completed_portals=self.completed_portals.checked_add(1).context("portal counter")?;self.portal=None;Ok(true)
    }
    /// False means another subsystem owns the opcode. Unknown world opcodes
    /// are listed explicitly; receipt does not imply their behavior is implemented.
    pub fn accept(&mut self, b: &[u8]) -> Result<bool> {
        let op = *b.first().context("empty world packet")?;
        let mut r = Reader { b, at: 1 };
        if op==0x5a{
            let destination=super::native_portal::parse(b)?;ensure!(self.ready(),"portal before ready world or repeated pending portal");
            let object_id=self.own_actor().unwrap().id;let generation=self.generation.checked_add(1).context("portal generation")?;
            *self=Self{portal:Some(super::native_portal::Pending{destination,object_id}),completed_portals:self.completed_portals,
                map_id:Some(destination.map),generation,started:true,learned_skills:std::mem::take(&mut self.learned_skills),
                relations:std::mem::take(&mut self.relations),..Default::default()};
            return Ok(true);
        }
        if op == 0x35 {
            if self.emblems.accept(b)? {
                self.visual_revision = self.visual_revision.checked_add(1).context("emblem visual revision")?;
                self.revision = self.revision.checked_add(1).context("emblem world revision")?;
            }
            return Ok(true);
        }
        if op==0x36 {
            let kind=r.c()?; let (_,first)=r.s_raw()?; let (_,second)=r.s_raw()?; r.end()?;
            let mut context=self.relations.clone();
            let updates=context.notice(&self.actors,self.own_actor().map(|a|a.relations.clan.as_slice()),kind,&first,&second)?;
            let changed=context!=self.relations || !updates.is_empty();
            self.relations=context;
            for (id,relations) in updates {self.actors.get_mut(&id).unwrap().relations=relations;}
            if changed {self.visual_revision+=1; self.revision+=1;}
            return Ok(true);
        }
        if matches!(op, 0x1e | 0x1f) {
            let skills = super::native_skills::book(b)?;
            if op == 0x1e {
                self.learned_skills.extend(skills);
            } else {
                self.learned_skills.retain(|id| !skills.contains(id));
            }
            self.revision += 1;
            return Ok(true);
        }
        if op == 0x28 {
            let map = r.h()?;
            let underwater = r.c()?;
            r.end()?;
            let generation = self.generation.checked_add(1).context("world generation")?;
            *self = Self {
                observer_mode: self.observer_mode,
                learned_skills: std::mem::take(&mut self.learned_skills),
                relations: std::mem::take(&mut self.relations),
                map_id: Some(map),
                underwater,
                generation,
                ..Default::default()
            };
            return Ok(true);
        }
        if self.map_id.is_none() {
            return Ok(false);
        }
        if op == 0xd2 {
            r.end()?;
            ensure!(!self.started, "duplicate world start");
            self.started = true;
            return Ok(true);
        }
        if !self.started {
            return Ok(false);
        }
        match op {
            0xe8 => {
                if self.minimap.accept(b)? {self.visual_revision += 1;}
            }
            0x6b|0x6c=>{
                let update=super::native_ability_buffs::parse(b)?;
                if let Some(status)=&mut self.status {
                    let count=self.attribute_updates[0].checked_add(1).context("ability receipt counter exhausted")?;
                    if op==0x6b {status.strength=update.score;status.weight_gauge=update.weight_gauge.unwrap();}
                    else {status.dexterity=update.score;}
                    self.attribute_updates[0]=count;
                    if update.kind==1&&update.received_seconds==0 {self.ability_buffs.remove(&op);}
                    else {self.ability_buffs.insert(op,update);}
                }else{self.orphan_updates+=1;}
            }
            0x50|0x51|0x52|0x56=>{
                use super::native_attributes::Update;
                let update=super::native_attributes::parse(b)?;
                if let Some(status)=&mut self.status {
                    let index=match update{
                        Update::Ability{values,weight}=>{[status.strength,status.intelligence,status.wisdom,status.dexterity,status.constitution,status.charisma]=values;status.weight_gauge=weight;0},
                        Update::Experience{level,experience}=>{status.level=level;status.experience=experience;1},
                        Update::Armor(values)=>{[status.armor_class,status.fire_resistance,status.water_resistance,status.wind_resistance,status.earth_resistance]=values;2},
                        Update::Magic(values)=>{self.magic_modifiers=Some(values);3},
                    };self.attribute_updates[index]+=1;
                }else{self.orphan_updates+=1;}
            }
            0x11 => {
                let event = Resurrection { target: r.d()?, action: r.c()?, source: r.d()?, graphics: r.h()? };
                r.end()?;
                ensure!(event.target != 0 && event.source != 0, "zero resurrection identity");
                if let Some(a) = self.actors.get_mut(&event.target) {
                    // A new life retires held death, old selections and projectile
                    // arrivals. HP and ratio still await their own server packets.
                    let revision = self.visual_revision + 1;
                    a.incarnation = revision;
                    a.target_revision = revision;
                    a.action_sequence = revision;
                    a.motion_revision = revision;
                    a.actions.clear();
                    a.motion_from = None;
                    a.position_predicted = false;
                    a.dead = false;
                    a.draw_state.native_light_dead = false;
                    a.graphics = event.graphics;
                    a.walk_action = event.action;
                    self.projectiles.retain(|p| p.attacker != event.target && p.target != event.target);
                    if self.status.is_some_and(|s| s.object_id == event.target) {
                        self.death_confirmation_pending = false;
                        self.last_death_confirmed_action = None;
                    }
                    self.visual_revision = revision;
                } else {
                    self.orphan_updates += 1;
                }
                self.resurrection_packets += 1;
                self.last_resurrection = Some(event);
            }
            0xe2 => {
                let bounds = ObserverBounds { min: [r.h()?, r.h()?], max_exclusive: [r.h()?, r.h()?] };
                r.end()?;
                ensure!(bounds.min[0] < bounds.max_exclusive[0] && bounds.min[1] < bounds.max_exclusive[1], "empty observer bounds");
                self.observer_bounds = Some(bounds);
                self.refresh_observer_camera();
            }
            0x4f => {
                let id = r.d()?;
                let raw_start = r.at;
                let (master,raw_master) = r.s_raw()?;
                r.end()?;
                ensure!(id != 0 && r.at - raw_start <= 64 && !master.chars().any(char::is_control), "invalid master update");
                if let Some(actor) = self.actors.get_mut(&id) {
                    if actor.master != master || actor.relations.master!=raw_master {
                        actor.master = master;
                        actor.relations.master=raw_master;
                        self.visual_revision += 1;
                    }
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x1b => {
                let id=r.d()?;let light=r.c()?;r.end()?;
                ensure!(id!=0,"zero light object ID");
                if let Some(actor)=self.actors.get_mut(&id) {
                    if actor.light!=light {actor.light=light;self.visual_revision+=1;}
                } else {self.orphan_updates+=1;}
            }
            0x4a => {
                let kind=r.c()?;let sound=r.h()?;r.end()?;
                let sequence=self.wire_sound_sequence.checked_add(1).context("wire sound counter")?;
                if self.wire_sounds.len()==128 {self.wire_sounds.remove(0);self.wire_sounds_dropped+=1;}
                self.wire_sounds.push(WireSound{sequence,kind,sound});
                self.wire_sound_sequence=sequence;
            }
            0x22 => {
                let attribute=super::native_doors::decode(b)?;
                let key=attribute.key();
                ensure!(self.door_attributes.contains_key(&key)||self.door_attributes.len()<4096,"dynamic attribute budget");
                let mut owners=self.actors.values().filter(|a|super::native_doors::is_door(a)&&[a.x,a.y]==attribute.position);
                let first=owners.next().map(|a|(a.id,a.incarnation));
                let owner=if owners.next().is_none(){first}else{None};
                self.door_attributes.insert(key,super::native_doors::Record{attribute,sequence:self.revision+1,owner});
                self.visual_revision+=1;
            }
            0x2f | 0x69 | 0xfa => {
                let id = r.d()?;
                let (clan, title, emblem, marker) = match op {
                    0x2f => (None, r.s()?, None, None),
                    0x69 => (Some(r.s_raw()?), r.s()?, Some(r.d()?), None),
                    0xfa => {
                        let emblem = r.d()?;
                        let title = r.s_capacity(17)?;
                        (None, title, Some(emblem), Some(r.d()? != 0))
                    }
                    _ => unreachable!(),
                };
                r.end()?;
                ensure!(id != 0, "zero label-update actor");
                if let Some(a) = self.actors.get_mut(&id) {
                    let mut changed = a.title != title;
                    a.title = title;
                    if let Some((clan,raw_clan)) = clan {
                        let previous=a.relations.clone();
                        changed |= a.clan != clan;
                        a.clan = clan;
                        a.relations.clan=raw_clan;
                        self.relations.clan_changed(&mut a.relations);
                        changed |= previous!=a.relations;
                    }
                    if let Some(emblem) = emblem {
                        changed |= a.emblem != emblem;
                        a.emblem = emblem;
                    }
                    if let Some(marker) = marker {
                        changed |= a.clan_marker_active != marker;
                        a.clan_marker_active = marker;
                    }
                    if changed {
                        self.visual_revision += 1;
                    }
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x0f => {
                let kind=r.c()?;let text=r.s()?;r.end()?;ensure!(!text.is_empty(),"empty system message");
                self.system_message_count+=1;self.last_system_message=Some((kind,text));
            }
            0x10 => {
                ensure!(b.len() <= 8192, "message byte budget");
                let code = r.h()?;
                let count = r.c()?;
                let mut args = Vec::new();
                for _ in 0..count {
                    args.push(r.s()?);
                }
                r.end()?;
                self.message_count += 1;
                self.last_message = Some((code, args));
                if matches!(code, 276 | 278 | 280 | 285 | 299 | 316 | 352) {
                    self.last_skill_rejection = Some((self.message_count, code));
                }
            }
            0x37 | 0x53 => {
                let (id, position) = if op == 0x37 {
                    (Some(r.d()?), [0, 0])
                } else {
                    (None, [r.h()?, r.h()?])
                };
                let graphics = r.h()?;
                r.end()?;
                ensure!(graphics != 0 && id != Some(0), "invalid native effect");
                let target = if let Some(id) = id {
                    let Some(actor) = self.actors.get(&id) else {
                        self.orphan_updates += 1;
                        return Ok(true);
                    };
                    Some((id, actor.incarnation))
                } else {
                    None
                };
                let position = id
                    .map(|id| [self.actors[&id].x, self.actors[&id].y])
                    .unwrap_or(position);
                if self.effects.len() == 128 {
                    self.effects.remove(0);
                    self.effects_dropped += 1;
                }
                self.effects.push(super::native_skills::Effect {
                    sequence: self.visual_revision + 1,
                    graphics,
                    target,
                    position,
                });
                self.visual_revision += 1;
            }
            0xe3 => {
                r.end()?;
                ensure!(
                    self.hp.is_some_and(|v| v[0] == 0) && self.own_actor().is_some_and(|a| a.dead),
                    "death confirmation without own death"
                );
                ensure!(
                    !self.death_confirmation_pending,
                    "duplicate death confirmation"
                );
                ensure!(
                    self.own_actor().map(|a| a.action_sequence) != self.last_death_confirmed_action,
                    "death already confirmed"
                );
                self.death_confirmation_pending = true;
            }
            0xd3 => {
                let result = r.d()? as i32;
                r.end()?;
                ensure!(matches!(result, -1..=1), "invalid battle result");
                ensure!(self.battle_result.is_none(), "duplicate battle result");
                self.battle_result = Some(result);
                self.visual_revision += 1;
            }
            0xdb => {
                let score = Score {
                    name: r.s()?,
                    points: r.d()?,
                    kills: r.d()?,
                    deaths: r.d()?,
                    assists: r.d()?,
                    force: [r.c()?, r.c()?],
                };
                r.end()?;
                ensure!(
                    self.battle_result.is_some()
                        && self.scores.len() < 256
                        && !score.name.is_empty(),
                    "unexpected scoreboard row"
                );
                self.scores.push(score);
            }
            0x39 => {
                let Some(attack)=super::native_attack_many::decode(b)? else {self.unhandled_opcodes.insert(op);return Ok(false);};
                self.action(attack.caster,attack.action,"spell");
                if let Some(actor)=self.actors.get(&attack.caster).filter(|a|!a.dead) {
                    if self.many_attacks.len()==128 {self.many_attacks.remove(0);self.many_attacks_dropped+=1;}
                    let targets=attack.targets.iter().map(|t|(t.id,t.hit,self.actors.get(&t.id).map(|a|a.incarnation))).collect();
                    self.many_attacks.push(super::native_attack_many::Cue{attack:attack.clone(),caster_incarnation:actor.incarnation,action_sequence:actor.action_sequence,targets});
                }
                if let Some(a)=self.actors.get_mut(&attack.caster){a.heading=attack.heading;}
                self.attack_packets+=1;
                if self.status.is_some_and(|s|s.object_id==attack.caster){self.own_cast_packets+=1;}
                self.visual_revision+=1;
            }
            0x23 => {
                let Some(attack) = super::native_combat::decode_attack(b)? else {
                    self.unhandled_opcodes.insert(op);
                    return Ok(false);
                };
                let spell = attack.projectile.as_ref().is_some_and(|p| {
                    (p.flight_type == 0 || matches!(attack.action, 18 | 19))
                        && !super::native_projectiles::supports_graphics(p.graphics)
                });
                self.action(
                    attack.attacker,
                    attack.action,
                    if spell { "spell" } else { "attack" },
                );
                if let Some(projectile) = &attack.projectile {
                    if let Some(actor) = self.actors.get(&attack.attacker).filter(|a| !a.dead) {
                        if self.projectiles.len() == 128 {
                            self.projectiles.remove(0);
                            self.projectile_cues_dropped += 1;
                        }
                        self.projectiles.push(super::native_combat::ProjectileCue {
                            heading: attack.heading,
                            attacker_incarnation: actor.incarnation,
                            target_incarnation: self
                                .actors
                                .get(&attack.target)
                                .map(|a| a.incarnation),
                            action_sequence: actor.action_sequence,
                            attacker: attack.attacker,
                            target: attack.target,
                            damage_marker: attack.damage_marker,
                            projectile: projectile.clone(),
                        });
                    }
                }
                if let Some(a) = self.actors.get_mut(&attack.attacker) {
                    a.heading = attack.heading;
                }
                // Qualified class3 projectile hurt is released on arrival.
                // Unknown projectile classes retain the previous fallback.
                if attack.damage_marker > 0
                    && !attack.projectile.as_ref().is_some_and(|p| {
                        super::native_projectiles::supports_graphics(p.graphics)
                            || super::native_spell_projectiles::supports_graphics(p.graphics)
                                || super::native_class1::supports_graphics(p.graphics)
                                || super::native_disintegrate::supports_graphics(p.graphics)
                                || super::native_cone_cold::supports_graphics(p.graphics)
                    })
                {
                    self.action(attack.target, 2, "damage");
                }
                self.attack_packets += 1;
                if self.status.is_some_and(|v| v.object_id == attack.attacker) {
                    if spell {
                        self.own_cast_packets += 1;
                    } else {
                        self.own_attack_packets += 1;
                    }
                }
                self.last_attack = Some(attack);
                self.visual_revision += 1;
            }
            0x20 => {
                let id = r.d()?;
                let action = r.c()?;
                r.end()?;
                ensure!(id != 0, "zero action actor");
                if matches!(action,28|29) {
                    if let Some(a)=self.actors.get_mut(&id).filter(|a|super::native_doors::is_door(a)) {
                        a.walk_action=action;
                    }
                }
                self.action(id, action, "action");
                if matches!(action,0|32|33|34|35) {
                    if let Some(a)=self.actors.get_mut(&id).filter(|a|super::native_obstacles::guardian(a)) {
                        if action==35&&a.walk_action!=35 {a.target_revision=self.visual_revision+1;}
                        if a.walk_action!=action {a.motion_revision=self.visual_revision+1;}
                        a.walk_action=action;
                    }
                }
                if matches!(action, 18 | 19) && self.status.is_some_and(|s| s.object_id == id) {
                    self.own_cast_packets += 1;
                }
                self.visual_revision += 1;
            }
            0x68 => {
                let id = r.d()?;
                let ratio = r.c()?;
                r.end()?;
                ensure!(id != 0 && ratio <= 100, "invalid HP ratio");
                if let Some(a) = self.actors.get_mut(&id) {
                    a.hp_ratio = ratio;
                    self.visual_revision += 1;
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x32 | 0x34 => {
                let id = r.d()?;
                let first = r.c()?;
                let second = if op == 0x32 { r.c()? } else { 0 };
                r.end()?;
                ensure!(
                    id != 0 && first <= 1 && second <= 1,
                    "actor draw state domain"
                );
                if let Some(a) = self.actors.get_mut(&id) {
                    let old = a.draw_state;
                    if op == 0x32 {
                        a.draw_state.green = first != 0;
                        a.draw_state.dim = second != 0;
                    } else {
                        if first != 0
                            && !old.invisible
                            && !self.status.is_some_and(|s| s.object_id == id)
                        {
                            a.target_revision = self.visual_revision + 1;
                        }
                        a.draw_state.invisible = first != 0;
                    }
                    if a.draw_state != old {
                        self.visual_revision += 1;
                    }
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x0b => {
                let mut a = actor(b)?;
                a.draw_state.native_light_dead = self.actors.get(&a.id)
                    .is_some_and(|previous|previous.draw_state.native_light_dead);
                a.relations.side_20=self.actors.get(&a.id).map_or(0,|previous|previous.relations.side_20);
                let own_name=if self.status.is_some_and(|s|s.object_id==a.id) {Some(a.relations.name.clone())}
                    else {self.own_actor().map(|a|a.relations.name.clone())};
                self.relations.put(&mut a,own_name.as_deref());
                a.incarnation = self.visual_revision + 1;
                a.motion_revision = self.visual_revision + 1;
                ensure!(
                    self.actors.contains_key(&a.id) || self.actors.len() < 2048,
                    "world actor budget"
                );
                self.door_attributes.retain(|_,r|r.owner.is_none_or(|(id,_)|id!=a.id));
                if self.actors.contains_key(&a.id) {self.minimap.remove(a.id);}
                self.actors.insert(a.id, a);
                self.visual_revision += 1;
            }
            0x12 => {
                let id = r.d()?;
                let x = r.h()?;
                let y = r.h()?;
                let source = [x,y];
                let heading = r.c()?;
                r.end()?;
                ensure!(id != 0 && heading < 8, "world movement domain");
                // Current formal NativeMovement emits the source cell; the
                // authoritative destination is one step in the wire heading.
                let (x, y) = destination(x, y, heading)?;
                if let Some(a) = self.actors.get_mut(&id) {
                    a.motion_from = if self.status.is_some_and(|s| s.object_id == id) {
                        None // Own S_MOVE_OBJECT is authoritative correction, not a new prediction.
                    } else {
                        Some(source) // Original4E1870 starts at the received source cell, including corrections.
                    };
                    a.motion_revision = self.visual_revision + 1;
                    a.x = x;
                    a.y = y;
                    a.heading = heading;
                    a.position_predicted = false;
                    self.visual_revision += 1;
                    if self.status.is_some_and(|s| s.object_id == id) {
                        self.local_corrections += 1;
                    }
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x1c => {
                let id = r.d()?;
                let heading = r.c()?;
                r.end()?;
                ensure!(id != 0 && heading < 8, "world heading domain");
                if let Some(a) = self.actors.get_mut(&id) {
                    a.heading = heading;
                    self.visual_revision += 1;
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x15 => {
                let id = r.d()?;
                r.end()?;
                ensure!(id != 0, "zero removed actor");
                if self.minimap.remove(id) {self.visual_revision += 1;}
                if self.actors.remove(&id).is_some() {
                    self.door_attributes.retain(|_,r|r.owner.is_none_or(|(owner,_)|owner!=id));
                    self.visual_revision += 1;
                }
            }
            0x63 => {
                r.end()?;
                self.movement_locks += 1;
            }
            0x25 => {
                let mode = r.c()?;
                r.end()?;
                ensure!(
                    matches!(mode, 2..=7 | 10..=13 | 17 | 22..=25),
                    "unverified paralysis mode"
                );
                if mode == 17 {
                    // Formal 6928A147 C35 acknowledgement: movement-lock
                    // release is not a cure for independently timed controls.
                } else if mode % 2 == 0 {
                    if self.movement_blocks.insert(mode) {
                        self.control_interruptions += 1;
                    }
                } else {
                    self.movement_blocks.remove(&(mode - 1));
                }
            }
            0x29 | 0x62 => {
                let id = r.d()?;
                let mode = r.c()?;
                let seconds = r.h()?;
                r.end()?;
                ensure!(
                    id != 0 && (mode != 0 || seconds == 0),
                    "invalid speed envelope"
                );
                if op == 0x29 {
                    super::native_speed::factor(mode, 0, 0)?;
                } else {
                    super::native_speed::factor(0, mode, 0)?;
                }
                if let Some(a) = self.actors.get_mut(&id) {
                    if op == 0x29 {
                        a.speed = mode;
                        a.speed_seconds = seconds;
                    } else {
                        a.brave = mode;
                        a.brave_seconds = seconds;
                    }
                    self.visual_revision += 1;
                } else {
                    self.orphan_updates += 1;
                }
            }
            0x0c => {
                let s = status_receipt::decode(b)?;
                ensure!(s.object_id != 0, "zero local object ID");
                if self.status.is_none_or(|old| old.object_id != s.object_id) {
                    self.magic_modifiers=None;self.attribute_updates=[0;4];self.ability_buffs.clear();
                    self.visual_revision += 1;
                }
                self.hp = Some([s.current_hp, s.maximum_hp]);
                self.mp = Some([s.current_mp, s.maximum_mp]);
                self.status = Some(s);
            }
            0xe9 => {
                let t = r.d()?;
                r.end()?;
                self.time_limit = Some(t);
            }
            0x21 => {
                let time = r.d()?;
                r.end()?;
                self.game_time = Some(time);
            }
            0x7b if b.get(1) == Some(&33) => {
                r.c()?;
                let challenge = [r.d()?, r.d()?, r.d()?];
                r.end()?;
                ensure!(
                    challenge.iter().all(|v| *v != 0),
                    "invalid integrity challenge"
                );
                self.integrity_challenge = Some(challenge);
            }
            0x4d | 0x0d => {
                let value = [r.h()?, r.h()?];
                r.end()?;
                if op == 0x4d {
                    self.mp = Some(value)
                } else {
                    self.hp = Some(value)
                }
            }
            _ => {
                self.unhandled_opcodes.insert(op);
                return Ok(false);
            }
        }
        self.revision = self.revision.checked_add(1).context("world revision")?;
        Ok(true)
    }
}
