//! Formal native room membership. Server names are display identities, not actor IDs.
use super::native_login::{decode_text, encode_text};
use anyhow::{ensure, Context, Result};
use zeroize::Zeroizing;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Member {
    pub name: String,
    pub force: u8,
    pub class: u8,
    pub sex: u8,
    pub matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub state: u8,
    pub affiliation: String,
    pub disconnects: u32,
    pub channel_selector: i8,
    pub preview_state: u8,
    pub rank: u32,
    pub grade: u8,
    pub certificates: [u8; 12],
    pub badges: [u8; 5],
    pub master: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub attributes: u8,
    pub password_required: bool,
    pub users_summary: u16,
    pub capacity: u16,
    pub mode: u8,
    pub map_id: u16,
    pub summary: [u8; 5],
    pub owner: String,
    pub options: [u8; 4],
    pub selections: Vec<u8>,
    pub members: Vec<Member>,
    pub countdown: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Observer(bool),
    KickStatus(bool),
    Kicked(String),
    Info { room: u32, update: Info },
    Joined(Room),
    Rejected(u8),
    Member { room: u32, member: Member },
    Ready { name: String, state: u8 },
    Force { name: String, force: u8 },
    Departed { room: u32, name: String },
    Deleted(u32),
    Owner { room: u32, name: String },
    Countdown(u8),
    BattleStart,
    Other,
}
/// Borrowed draft only; passwords must never enter serializable room snapshots.
pub struct Settings<'a> {
    pub name:&'a str, pub password:&'a str, pub attributes:u8, pub capacity:u16,
    pub options:Option<([u8;4],&'a [u8])>,
}
impl Settings<'_> {
    pub fn request(&self,opcode:u8)->Result<Zeroizing<Vec<u8>>> {
        ensure!(matches!(opcode,0xcc|0xdd),"invalid room settings opcode");
        ensure!(self.attributes & !0x3c==0 && (self.attributes & 4!=0)==!self.password.is_empty(),"room password flag mismatch");
        ensure!((2..=200).contains(&self.capacity),"room wire capacity outside 2..200");
        let name=encode_text(self.name,89)?;
        let mut body=Zeroizing::new(vec![opcode]);body.extend_from_slice(&name);body.push(0);body.push(self.attributes);
        if !self.password.is_empty(){let password=Zeroizing::new(encode_text(self.password,4)?);body.extend_from_slice(&password);}
        body.push(0);body.extend(self.capacity.to_le_bytes());
        if let Some((values,selections))=self.options {
            validate_options(values, selections)?;
            body.extend(values);body.push(selections.len() as u8);body.extend_from_slice(selections);
        }
        Ok(body)
    }
}
pub fn observer_request(enabled:bool)->Zeroizing<Vec<u8>> {Zeroizing::new(vec![0xe1,u8::from(enabled)])}
pub fn kick_request(name:&str)->Result<Zeroizing<Vec<u8>>> {
    // Only a name from the received 27-byte room membership may be targeted.
    let mut body=Zeroizing::new(vec![0xd4]);body.extend_from_slice(&encode_text(name,27)?);body.push(0);Ok(body)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Info {
    Mode(u8),
    Map(u16),
    Summary { name: String, mode: u8, map: u16, owner: String, reserved: [u8; 5] },
    Configuration { name: String, password_required: bool, attributes: u8, capacity: u16, options: [u8;4], selections: Vec<u8> },
}
pub fn apply_info(active: &mut Room, id: u32, update: Info) -> Result<()> {
    ensure!(active.id == id, "game info for different room");
    match update {
        Info::Mode(mode) => active.mode = mode,
        Info::Map(map) => active.map_id = map,
        Info::Summary { name, mode, map, owner, reserved } => {
            active.name=name;active.mode=mode;active.map_id=map;active.owner=owner;active.summary=reserved;
        }
        Info::Configuration { name, password_required, attributes, capacity, options, selections } => {
            active.name=name;active.password_required=password_required;active.attributes=attributes;
            active.capacity=capacity;active.options=options;active.selections=selections;
        }
    }
    Ok(())
}
/// C02 BattleRules.Mode source catalogue; never a portal destination list.
pub fn mode_maps(mode:u8)->&'static [u16] {
    match mode {
        0=>&[565,566,569,570],
        1=>&[564,514,509,508,551,504,505,552,506,526,525,516,531,534,529],
        2=>&[554,553,513,527,518,519,520,521,522],
        3=>&[573,563,547],
        4=>&[540],
        5=>&[510,528,532,512,517,524,530,535],
        6=>&[500,560],
        _=>&[],
    }
}
pub fn map_selection(mode:u8,map:u16)->Result<Zeroizing<Vec<u8>>> {
    ensure!(mode_maps(mode).contains(&map),"地圖不屬於目前房間模式");
    let mut body=Zeroizing::new(vec![0xd1,3]);body.extend(map.to_le_bytes());Ok(body)
}
pub fn mode_step(direction: u8) -> Result<Zeroizing<Vec<u8>>> {
    ensure!(matches!(direction,1|2), "mode direction must be next or previous");
    Ok(Zeroizing::new(vec![0xd1,2,direction]))
}
/// Same authenticated join only: broadcast S_CD may precede its S_CE.
/// These snapshots cannot create a room or complete membership on their own.
#[derive(Default)]
pub struct EarlyRoster {binding:Option<(u64,u32)>,members:Vec<Member>}
impl EarlyRoster {
    pub fn clear(&mut self){self.binding=None;self.members.clear();}
    pub fn push(&mut self,epoch:u64,requested:Option<u32>,room:u32,member:Member)->Result<()> {
        ensure!(room!=0&&requested.is_none_or(|id|id==room),"early participant for another requested room");
        ensure!(self.binding.is_none_or(|b|b==(epoch,room)),"early participant crossed room or connection");
        if let Some(old)=self.members.iter_mut().find(|m|m.name==member.name){*old=member;}else{ensure!(self.members.len()<256,"early participant limit");self.members.push(member);}
        self.binding=Some((epoch,room));Ok(())
    }
    /// A peer broadcast may precede our CE. It can only retire a snapshot
    /// within this requested join; it cannot create or accept room membership.
    pub fn depart(&mut self,epoch:u64,requested:Option<u32>,room:u32,name:&str)->Result<()> {
        ensure!(room!=0 && requested.is_none_or(|id|id==room),"early departure for another requested room");
        ensure!(self.binding.is_none_or(|b|b==(epoch,room)),"early departure crossed room or connection");
        ensure!(requested.is_some() || self.binding==Some((epoch,room)),"early departure without a bound pending room");
        encode_text(name,27)?;
        self.members.retain(|m|m.name!=name);
        self.binding=Some((epoch,room));Ok(())
    }
    pub fn take(&mut self,epoch:u64,room:u32)->Result<Vec<Member>> {
        ensure!(self.binding.is_none_or(|b|b==(epoch,room)),"room acceptance does not match early participants");
        self.binding=None;Ok(std::mem::take(&mut self.members))
    }
}
/// Only peers still present when our own departure is acknowledged may send
/// late departure notifications. This bounded tombstone never updates a room.
#[derive(Default)]
pub struct RetiredDepartures {
    room: Option<(u32, std::collections::BTreeSet<String>)>,
}
/// The formal writer can interleave S_D4 between S_CE and the S_CD roster.
/// A notification cannot create a member. A later full S_CD supersedes it;
/// unresolved names expire without extending the original eight-second bound.
#[derive(Default)]
pub struct PendingRosterForces {
    room: Option<u32>,
    names: std::collections::BTreeMap<String, std::time::Instant>,
}
impl PendingRosterForces {
    pub fn clear(&mut self) { self.room=None; self.names.clear(); }
    pub fn expired(&self, now:std::time::Instant)->bool {
        self.names.values().any(|first|now.saturating_duration_since(*first)>=std::time::Duration::from_secs(8))
    }
    pub fn member_received(&mut self, room:u32, name:&str)->Result<()> {
        ensure!(self.room.is_none_or(|id|id==room),"pending force belongs to another room");
        self.names.remove(name);
        if self.names.is_empty(){self.room=None;}
        Ok(())
    }
    pub fn apply(&mut self, room:&mut Room, name:&str, force:u8, awaiting_roster:bool, now:std::time::Instant)->Result<bool> {
        ensure!(force<=1,"invalid room force");
        ensure!(self.room.is_none_or(|id|id==room.id),"pending force belongs to another room");
        if let Some(member)=room.members.iter_mut().find(|m|m.name==name) {
            member.force=force;
            return Ok(true);
        }
        ensure!(awaiting_roster,"force update for unknown participant");
        // Names have the same strict bound as actual room membership.
        encode_text(name,27)?;
        ensure!(self.names.contains_key(name)||self.names.len()+room.members.len()<usize::from(room.capacity),"pending roster capacity exceeded");
        self.names.entry(name.to_owned()).or_insert(now);
        self.room=Some(room.id);
        Ok(false)
    }
}
impl RetiredDepartures {
    pub fn clear(&mut self) { self.room = None; }
    pub(crate) fn consume(&mut self, room:u32, name:&str)->bool {
        self.room.as_mut().is_some_and(|(id,peers)|*id==room && peers.remove(name))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Departure { SelfLeft, PeerLeft, RetiredPeer }
pub fn depart(
    active: &mut Option<Room>, retired: &mut RetiredDepartures,
    room_id: u32, name: &str, own: &str,
) -> Result<Departure> {
    let Some(room) = active.as_mut() else {
        if retired.consume(room_id,name) { return Ok(Departure::RetiredPeer); }
        anyhow::bail!("departure without current or known retired room");
    };
    ensure!(room.id == room_id, "departure for a different room");
    if name == own {
        retired.room = Some((room_id, room.members.iter().filter(|m|m.name!=own)
            .map(|m|m.name.clone()).collect()));
        *active = None;
        Ok(Departure::SelfLeft)
    } else {
        room.members.retain(|m|m.name!=name);
        Ok(Departure::PeerLeft)
    }
}
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).context("room offset overflow")?;
        let b = self.b.get(self.at..end).context("truncated room field")?;
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
    fn raw(&mut self, max: usize) -> Result<&'a [u8]> {
        let n = self.b[self.at..]
            .iter()
            .take(max + 1)
            .position(|x| *x == 0)
            .context("room string lacks bounded NUL")?;
        Ok(&self.take(n + 1)?[..n])
    }
    fn text(&mut self, max: usize, empty: bool) -> Result<String> {
        let b = self.raw(max)?;
        if empty && b.is_empty() {
            Ok(String::new())
        } else {
            decode_text(b)
        }
    }
    fn end(&self) -> Result<()> {
        ensure!(self.at == self.b.len(), "trailing room bytes");
        Ok(())
    }
}
pub fn parse(body: &[u8]) -> Result<Packet> {
    ensure!(!body.is_empty(), "missing room opcode");
    let mut r = Reader { b: body, at: 1 };
    let packet = match body[0] {
        0xe5 => {let value=r.c()?;ensure!(value<=1,"observer state domain");Packet::Observer(value!=0)}
        0xfc => {let value=r.d()?;ensure!(value<=1,"kick result domain");Packet::KickStatus(value==0)}
        0xd6 => Packet::Kicked(r.text(27,false)?),
        0xd5 if matches!(body.get(1), Some(0..=3)) => {
            ensure!(body.len() <= 180, "room info envelope");
            let kind=r.c()?;let room=r.d()?;ensure!(room!=0,"zero room info identity");
            let update=match kind {
                0 => {
                    let name=r.text(89,false)?;let mode=r.c()?;ensure!(mode<=6,"unknown room mode");
                    let map=r.h()?;let first=r.c()?;let owner=r.text(27,true)?;let tail=r.take(4)?;
                    Info::Summary{name,mode,map,owner,reserved:[first,tail[0],tail[1],tail[2],tail[3]]}
                }
                1 => {
                    let name=r.text(89,false)?;
                    let password=Zeroizing::new(r.text(4,true)?);let password_required=!password.is_empty();
                    let attributes=r.c()?;let capacity=u16::from(r.c()?);let options=r.take(4)?.try_into()?;
                    let count=r.c()?;ensure!(count<=8,"room selection bound");let selections=r.take(count as usize)?.to_vec();
                    ensure!(attributes & !0x3c == 0 && (attributes & 4 != 0)==password_required
                        && (2..=200).contains(&capacity) && !selections.contains(&0),"invalid room configuration");
                    validate_options(options, &selections)?;
                    Info::Configuration{name,password_required,attributes,capacity,options,selections}
                }
                2 => {let mode=r.c()?;ensure!(mode<=6,"unknown room mode");Info::Mode(mode)}
                3 => Info::Map(r.h()?),
                _ => anyhow::bail!("unknown room info subtype"),
            };
            Packet::Info{room,update}
        }
        0xce => {
            ensure!(body.len() <= 155, "join reply bound");
            let result = r.c()?;
            if result != 0 {
                ensure!(
                    [1, 3, 4, 5, 7, 10, 11, 12, 13, 14, 20, 21].contains(&result),
                    "unsupported room rejection"
                );
                Packet::Rejected(result)
            } else {
                let id = r.d()?;
                ensure!(id != 0, "zero joined room");
                let name = r.text(89, false)?;
                let attributes = r.c()?;
                // Validate but never store or serialize the server's room password.
                let password = r.raw(4)?;
                let password_required = !password.is_empty();
                if password_required {
                    let _ = Zeroizing::new(decode_text(password)?);
                }
                let users_summary = r.h()?;
                let capacity = r.h()?;
                let mode = r.c()?;
                let map_id = r.h()?;
                let summary = r.take(5)?.try_into()?;
                let owner = r.text(27, false)?;
                let options = r.take(4)?.try_into()?;
                let count = r.c()?;
                ensure!(count <= 8, "room selection bound");
                let selections = r.take(count as usize)?.to_vec();
                ensure!(attributes & !0x3c == 0 && (attributes & 4 != 0)==password_required
                    && (2..=200).contains(&capacity) && mode<=6, "invalid joined room configuration");
                validate_options(options, &selections)?;
                Packet::Joined(Room {
                    id,
                    name,
                    attributes,
                    password_required,
                    users_summary,
                    capacity,
                    mode,
                    map_id,
                    summary,
                    owner,
                    options,
                    selections,
                    members: vec![],
                    countdown: None,
                })
            }
        }
        0xcd => {
            ensure!(body.len() <= 105, "participant bound");
            let room = r.d()?;
            let name = r.text(27, false)?;
            let force = r.c()?;
            let class = r.c()?;
            let sex = r.c()?;
            let matches = r.d()?;
            let wins = r.d()?;
            let losses = r.d()?;
            let state = r.c()?;
            let affiliation = r.text(16, true)?;
            ensure!(
                r.c()? == 0 && r.raw(16)? == b"Ready",
                "participant reserved/label"
            );
            let disconnects = r.d()?;
            let channel_selector = r.c()? as i8;
            let preview_state = r.c()?;
            let rank = r.d()?;
            let grade = r.c()?;
            let certificates = r.take(12)?.try_into()?;
            let badges: [u8; 5] = r.take(5)?.try_into()?;
            let master = r.d()?;
            ensure!(
                room != 0
                    && force <= 1
                    && class <= 4
                    && sex <= 1
                    && state <= 2
                    && preview_state <= 2
                    && (-1..=4).contains(&channel_selector)
                    && grade <= 14
                    && badges.iter().all(|v| *v <= 127)
                    && master <= 1,
                "participant field domain"
            );
            Packet::Member {
                room,
                member: Member {
                    name,
                    force,
                    class,
                    sex,
                    matches,
                    wins,
                    losses,
                    state,
                    affiliation,
                    disconnects,
                    channel_selector,
                    preview_state,
                    rank,
                    grade,
                    certificates,
                    badges,
                    master: master != 0,
                },
            }
        }
        0xd1 => {
            let name = r.text(27, false)?;
            let state = r.c()?;
            ensure!(state <= 2, "invalid participant ready state");
            Packet::Ready { name, state }
        }
        0xd4 => {
            let name = r.text(27, false)?;
            let force = r.c()?;
            ensure!(force <= 1, "invalid participant force");
            Packet::Force { name, force }
        }
        0xd0 => Packet::Departed {
            room: r.d()?,
            name: r.text(27, false)?,
        },
        0xcf => Packet::Deleted(r.d()?),
        0xd5 => {
            if r.c()? != 5 {
                return Ok(Packet::Other);
            }
            let room = r.d()?;
            ensure!(room != 0, "zero room owner identity");
            Packet::Owner {
                room,
                name: r.text(27, false)?,
            }
        }
        0xe1 => Packet::Countdown(r.c()?),
        0xd2 => Packet::BattleStart,
        _ => return Ok(Packet::Other),
    };
    r.end()?;
    Ok(packet)
}
pub fn force_request(force: u8) -> Result<Zeroizing<Vec<u8>>> {
    ensure!(force <= 1, "invalid requested force");
    Ok(Zeroizing::new(vec![0xd3, force]))
}
pub fn join(id: u32, password: &str) -> Result<Zeroizing<Vec<u8>>> {
    ensure!(id != 0, "missing room id");
    let mut body = Zeroizing::new(vec![0xcf]);
    body.extend(id.to_le_bytes());
    if !password.is_empty() {
        body.extend_from_slice(&encode_text(password, 4)?);
    }
    body.push(0);
    Ok(body)
}
fn validate_options(values: [u8; 4], selections: &[u8]) -> Result<()> {
    ensure!(selections.len()<=8 && !selections.contains(&0),"room selection domain");
    ensure!((values==[0;4] && selections.is_empty()) || (values[0]<=7 && values[1]<=2 && (1..=127).contains(&values[2]) && values[3]<=31),"room option domain");
    Ok(())
}
pub fn upsert(room: &mut Room, member: Member) -> Result<()> {
    if let Some(old) = room.members.iter_mut().find(|m| m.name == member.name) {
        *old = member;
    } else {
        ensure!(room.members.len() < 256, "participant limit");
        room.members.push(member);
    }
    Ok(())
}
