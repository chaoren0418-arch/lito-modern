//! Ordinary Native roster producer: opcode3 -> count, opcode203 -> record,
//! then a selected 0..3 control -> raw actor alias and separate +5C bound.
//! No ModernCharacterSummary conversion, caller-supplied record or actor ID.
//! Native AFBC/AFBD alternate branches and actor205 replacement remain OPEN.
use anyhow::{ensure, Context, Result};

pub const SLOT_COUNT: usize = 4;
pub const JAVA_ROW_BOUND:usize=8;
#[path="native_character_directory_fields.rs"]pub mod fields;
const FORMAT: &[u8] = b"sscchhhcccccccccddddddddddcdddddc";
const OFFSETS: [usize; 30] = [0x2d,0x2e,0x30,0x32,0x34,0x36,0x37,0x38,0x39,0x3a,0x3b,0x3c,0x3d,0x3e,0x40,0x48,0x44,0x4c,0x50,0x54,0x58,0x60,0x64,0x68,0x5c,0x84,0x80,0x88,0x8c,0x90];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record { bytes: [u8; 180], opcode_write_byte: u8, fields:fields::Fields }
impl Record {
    pub fn fields(&self)->&fields::Fields{&self.fields}
    pub fn clan_name(&self)->&[u8]{let n=self.bytes[0x1c..0x2d].iter().position(|b|*b==0).unwrap();&self.bytes[0x1c..0x1c+n]}
    pub fn display_name(&self)->Result<String>{fields::display_ms950(self.name())}
    pub fn name(&self) -> &[u8] { let n=self.bytes[..28].iter().position(|b|*b==0).unwrap(); &self.bytes[..n] }
    pub fn class_byte(&self) -> u8 { self.bytes[0x2d] }
    pub fn variant_byte(&self) -> u8 { self.bytes[0x2e] }
    pub fn selection_guard_byte(&self) -> u8 { self.bytes[0x37] }
    pub fn arena_bound_byte(&self) -> u8 { self.bytes[0x5c] }
    /// Field32 writes the saved incoming ESI (the opcode address), NOT +94.
    /// Modern keeps the immutable event; no caller memory is modified.
    pub fn opcode_write_byte(&self) -> u8 { self.opcode_write_byte }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedIdentity { slot: usize, actor_alias: Vec<u8>, class_byte: u8, variant_byte: u8, arena_bound_byte: u8 }
impl SelectedIdentity {
    pub fn slot(&self)->usize { self.slot }
    pub fn actor_alias(&self)->&[u8] { &self.actor_alias }
    pub fn class_byte(&self)->u8 { self.class_byte }
    pub fn variant_byte(&self)->u8 { self.variant_byte }
    pub fn arena_bound_byte(&self)->u8 { self.arena_bound_byte }
    /// This alias comes from selected roster record ->64ABF8. Actor205 has
    /// not yet been admitted; no identifier may be borrowed from mode206.
    pub fn actor_identifier(&self)->Option<u32> { None }
}

pub(crate) enum Packet { Start(u8), Record(Record) }
struct Reader<'a> { body: &'a [u8], at: usize }
impl Reader<'_> {
    fn take(&mut self,n:usize)->Result<&[u8]> { let end=self.at.checked_add(n).context("roster offset overflow")?;let b=self.body.get(self.at..end).context("short Native roster body")?;self.at=end;Ok(b) }
    fn string(&mut self,cap:usize)->Result<&[u8]> {
        let rest=&self.body[self.at..];let n=rest.iter().take(cap).position(|b|*b==0).context("roster string exceeds NUL capacity")?;self.take(n+1)
    }
}
pub(crate) fn decode(body:&[u8])->Result<Packet> {
    ensure!(body.len()<=155,"Native roster body exceeds bound");
    if body.first()==Some(&3) { ensure!(body.len()==2 && (body[1] as i8)>=0 && usize::from(body[1])<=JAVA_ROW_BOUND,"roster count outside bounded signed-byte rows");return Ok(Packet::Start(body[1])); }
    ensure!(body.first()==Some(&203),"not a Native roster body");
    let mut r=Reader{body,at:1};let mut bytes=[0;180];
    for (offset,cap) in [(0,28),(0x1c,17)] { let s=r.string(cap)?;bytes[offset..offset+s.len()].copy_from_slice(s); }
    let suffix=r.at;
    for (code,offset) in FORMAT[2..32].iter().zip(OFFSETS) {
        let n=match code { b'c'=>1,b'h'=>2,b'd'=>4,_=>unreachable!() };let b=r.take(n)?;bytes[offset..offset+n].copy_from_slice(b);
    }
    let opcode_write_byte=r.take(1)?[0];ensure!(r.at==body.len(),"trailing Native roster bytes");
    let fields=fields::Fields::decode(&body[suffix..])?;
    Ok(Packet::Record(Record{bytes,opcode_write_byte,fields}))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change { Started { count: u8 }, Appended { slot: usize, complete: bool } }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State { expected: Option<u8>, records: Vec<Record>, selected: Option<SelectedIdentity>, complete: bool }
impl State {
    pub fn records(&self)->&[Record] { &self.records }
    pub fn selected(&self)->Option<&SelectedIdentity> { self.selected.as_ref() }
    pub fn complete(&self)->bool { self.complete }
    pub fn expected(&self)->Option<u8> { self.expected }
    pub(crate) fn prepare_packet(&self,body:&[u8])->Result<(Self,Change)> {
        let packet=decode(body)?;let mut next=self.clone();
        let change=match packet {
            Packet::Start(count)=> { next=Self{expected:Some(count),complete:count==0,..Self::default()};Change::Started{count} }
            Packet::Record(record)=> { let count=next.expected.context("roster203 before count3")?;ensure!(!next.complete && next.records.len()<usize::from(count),"unsolicited or excess roster203");let slot=next.records.len();next.records.push(record);next.complete=next.records.len()==usize::from(count);Change::Appended{slot,complete:next.complete} }
        };Ok((next,change))
    }
    ///NativeBack retains raw62FAA0 roster bytes;Modern retireslogicalselected binding.
    pub(crate) fn without_selected(&self)->Self{let mut next=self.clone();next.selected=None;next}
    pub(crate) fn prepare_select(&self,slot:usize)->Result<Self> {
        self.prepare_select_bounded(slot,SLOT_COUNT)
    }
    pub(crate) fn prepare_select_bounded(&self,slot:usize,bound:usize)->Result<Self> {
        ensure!(self.complete && slot<bound&&bound<=JAVA_ROW_BOUND,"roster selection not ready");let r=self.records.get(slot).context("unreceived roster slot")?;
        ensure!(r.class_byte()!=255 && r.variant_byte()!=255 && r.selection_guard_byte()!=0,"Native selected record refused");
        let mut next=self.clone();next.selected=Some(SelectedIdentity{slot,actor_alias:r.name().to_vec(),class_byte:r.class_byte(),variant_byte:r.variant_byte(),arena_bound_byte:r.arena_bound_byte()});Ok(next)
    }
}
