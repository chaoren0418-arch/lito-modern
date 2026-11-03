//! Source component: authoritative bookmarks, exact #69 intent, consent and S55 portal grant.
//! Controller/native-window integration is deliberately tracked separately; no GAME qualification.
use super::{native_login::{LoginPhase,Snapshot},native_inventory,native_portal::Destination};
use anyhow::{ensure,Context,Result};
use std::time::{Duration,Instant};

#[derive(Clone,Debug,PartialEq,Eq)]
struct Character {epoch:u64,source:String,inventory:native_inventory::Owner}
impl Character {
    fn current(s:&Snapshot,epoch:u64)->Option<Self> {
        let inventory=s.inventory.owner.as_ref()?;
        if !s.game_connected||inventory.connection!=epoch||s.selected_character.as_deref()!=Some(inventory.character.as_str()) {return None;}
        Some(Self{epoch,source:s.selected_source.clone()?,inventory:inventory.clone()})
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Scene {character:Character,generation:u64,map:u16,phase:LoginPhase,room:Option<u32>,actor:(u32,u64)}
impl Scene {
    fn current(s:&Snapshot,epoch:u64)->Option<Self> {
        let w=&s.world;let a=w.own_actor()?;
        if !s.phase.active_world()||s.observer||w.observer_mode||!w.ready()||w.battle_result.is_some()
            ||a.dead||!w.hp.is_some_and(|h|h[0]>0) {return None;}
        Some(Self{character:Character::current(s,epoch)?,generation:w.generation,map:w.map_id?,phase:s.phase,
            room:s.joined_room.as_ref().map(|r|r.id),actor:(a.id,a.incarnation)})
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Bookmark {pub(crate) label:String,pub(crate) destination:Destination,raw_name:Vec<u8>}
fn bookmark(body:&[u8])->Result<Option<Bookmark>> {
    if body.first()!=Some(&0x30){return Ok(None);}
    let end=body[1..].iter().position(|b|*b==0).map(|i|i+1).context("bookmark name terminator")?;
    ensure!((1..=27).contains(&(end-1))&&body.len()==end+7,"bookmark name/tail");
    let raw=&body[1..end];ensure!(raw.iter().all(|b|*b>=0x20&&*b!=0x7f),"bookmark control character");
    let label=super::native_login::decode_text(raw)?;ensure!(!label.trim().is_empty(),"blank bookmark");
    let h=|n|u16::from_le_bytes([body[n],body[n+1]]);
    // Protocol.addBookmark is NAME, X, Y, MAP, unlike the C14 MAP, X, Y request.
    Ok(Some(Bookmark{label,destination:Destination{x:h(end+1),y:h(end+3),map:h(end+5)},raw_name:raw.to_vec()}))
}
#[derive(Default)]
pub(crate) struct Bookmarks {owner:Option<Character>,rows:Vec<Bookmark>,revision:u64}
impl Bookmarks {
    pub(crate) fn rows(&self,s:&Snapshot,epoch:u64)->&[Bookmark] {
        if self.owner.as_ref()==Character::current(s,epoch).as_ref()&&self.owner.is_some(){&self.rows}else{&[]}
    }
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64)->Result<()> {
        let owner=Character::current(s,epoch);
        if self.owner!=owner {let revision=self.revision.checked_add(1).context("bookmark revision exhausted")?;self.rows.clear();self.owner=owner;self.revision=revision;}
        Ok(())
    }
    pub(crate) fn receive(&mut self,s:&Snapshot,epoch:u64,body:&[u8])->Result<bool> {
        let Some(row)=bookmark(body)?else{return Ok(false);};
        let owner=Character::current(s,epoch).context("bookmark without selected GAME character")?;
        let same=self.owner.as_ref()==Some(&owner);
        let index=if same{self.rows.iter().position(|r|r.raw_name==row.raw_name)}else{None};
        if same&&index.is_some_and(|i|self.rows[i]==row){return Ok(true);}
        ensure!(!same||index.is_some()||self.rows.len()<50,"bookmark count exceeds formal limit");
        let revision=self.revision.checked_add(1).context("bookmark revision exhausted")?;
        if !same{self.rows.clear();self.owner=Some(owner);}
        if let Some(i)=index{self.rows[i]=row;}else{self.rows.push(row);}
        self.revision=revision;Ok(true)
    }
}
#[derive(Clone,Debug)]
pub(crate) struct Choice {owner:Scene,destination:Destination,bookmark:Option<(u64,Bookmark)>}
impl Choice {
    pub(crate) fn random(s:&Snapshot,epoch:u64)->Result<Self> {
        let owner=Scene::current(s,epoch).context("請在世界中選擇傳送目的地")?;
        Ok(Self{destination:Destination{map:owner.map,x:0,y:0},owner,bookmark:None})
    }
    pub(crate) fn stored(s:&Snapshot,epoch:u64,book:&Bookmarks,index:usize)->Result<Self> {
        let row=book.rows(s,epoch).get(index).context("書籤已更新，請重新選擇")?.clone();
        ensure!(row.destination.x!=0&&row.destination.y!=0,"書籤座標不可用");
        Ok(Self{owner:Scene::current(s,epoch).context("角色已離開場景")?,destination:row.destination,bookmark:Some((book.revision,row))})
    }
    pub(crate) fn request(&self,s:&Snapshot,epoch:u64,book:&Bookmarks)->Result<Vec<u8>> {
        ensure!(Scene::current(s,epoch).as_ref()==Some(&self.owner),"角色或場景已更新，請重新選擇目的地");
        ensure!(s.world.learned_skills.contains(&69)&&s.world.mp.is_some_and(|m|m[0]>0)&&s.world.movement_blocks.is_empty(),"尚未學會集體傳送術，或目前無法施法");
        if let Some((revision,row))=&self.bookmark {ensure!(*revision==book.revision&&book.rows(s,epoch).contains(row),"書籤已更新，請重新選擇");}
        let d=self.destination;let mut body=vec![0x14,8,4];body.extend(d.map.to_le_bytes());body.extend(d.x.to_le_bytes());body.extend(d.y.to_le_bytes());Ok(body)
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Token {serial:u16,owner:Scene}
impl Token {pub(crate) fn command_id(&self,yes:bool)->u16{1000+self.serial*2+u16::from(yes)}}
#[derive(Clone,Debug)]
pub(crate) struct Prompt {token:Token,received:Instant,waiting:bool}
impl Prompt {pub(crate) fn token(&self)->Token{self.token.clone()}pub(crate) fn waiting(&self)->bool{self.waiting}}
#[derive(Default)]
pub(crate) struct Invitation {serial:u16,prompt:Option<Prompt>}
fn ask(body:&[u8])->Result<Option<u16>> {
    if body.first()!=Some(&0x24){return Ok(None);}
    ensure!(body.len()>=3,"truncated native question");let id=u16::from_le_bytes([body[1],body[2]]);
    if id!=748{return Ok(None);}ensure!(body==[0x24,0xec,2,0],"invalid mass teleport question string/tail");Ok(Some(id))
}
impl Invitation {
    pub(crate) fn current(&self,s:&Snapshot,epoch:u64,now:Instant)->Option<&Prompt> {
        self.prompt.as_ref().filter(|p|Scene::current(s,epoch).as_ref()==Some(&p.token.owner)
            &&now.saturating_duration_since(p.received)<Duration::from_secs(30))
    }
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64,now:Instant){if self.current(s,epoch,now).is_none(){self.prompt=None;}}
    pub(crate) fn retire(&mut self){self.prompt=None;}
    pub(crate) fn receive(&mut self,s:&Snapshot,epoch:u64,now:Instant,body:&[u8])->Result<bool> {
        if body.first()!=Some(&0x24){return Ok(false);}let question=ask(body)?;
        self.prompt=None;let Some(_)=question else{return Ok(false);};
        let Some(owner)=Scene::current(s,epoch)else{return Ok(true);};
        ensure!(self.serial<30_000,"集體傳送詢問命令已用盡，請重新開啟視窗");self.serial+=1;
        self.prompt=Some(Prompt{token:Token{serial:self.serial,owner},received:now,waiting:false});Ok(true)
    }
    pub(crate) fn answer(&mut self,s:&Snapshot,epoch:u64,now:Instant,token:&Token,yes:bool)->Result<Option<Vec<u8>>> {
        let p=self.current(s,epoch,now).context("集體傳送詢問已失效")?;ensure!(&p.token==token,"集體傳送詢問已更新");
        if p.waiting{ensure!(!yes,"已送出同意，等待伺服器結果");self.prompt=None;return Ok(None);}
        // No caster name, destination or local clan/MP/book inference is carried by S24/748.
        if yes{self.prompt.as_mut().unwrap().waiting=true;}else{self.prompt=None;}
        Ok(Some(vec![0x23,0xec,2,u8::from(yes),0]))
    }
}
/// Native server-owned S55 continuation, also used by other explicit server teleports.
/// It is a grant from this stream, not an inferred success from a spell graphic.
#[derive(Default)]
pub(crate) struct PortalGrant {owner:Option<Scene>}
impl PortalGrant {
    pub(crate) fn current(&self,s:&Snapshot,epoch:u64)->bool {self.owner.is_some()&&self.owner.as_ref()==Scene::current(s,epoch).as_ref()}
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64){if !self.current(s,epoch){self.owner=None;}}
    pub(crate) fn retire(&mut self){self.owner=None;}
    pub(crate) fn receive(&mut self,s:&Snapshot,epoch:u64,body:&[u8])->Result<Option<[u8;2]>> {
        if body.first()!=Some(&0x55){return Ok(None);}
        ensure!(body.len()==7&&body[1..3]==[0,64],"invalid S_REQUEST_SUMMON layout");
        let id=u32::from_le_bytes(body[3..7].try_into().unwrap());ensure!(id!=0,"zero summon owner");
        let owner=Scene::current(s,epoch).context("summon continuation without live world owner")?;
        ensure!(id==owner.actor.0,"summon continuation for another actor");
        ensure!(!self.current(s,epoch),"duplicate summon continuation");
        self.owner=Some(owner);
        // Formal returnSummon reads but does not branch on this byte; zero is a codec value.
        // The grant is consumed only by one S5A, not by a repeated S55 or local animation.
        Ok(Some([0x4e,0]))
    }
    pub(crate) fn take(&mut self,s:&Snapshot,epoch:u64,d:Destination)->Result<LoginPhase> {
        let owner=self.owner.take().context("portal without native summon grant")?;
        ensure!(Scene::current(s,epoch).as_ref()==Some(&owner),"retired summon grant");
        if owner.phase==LoginPhase::BattlePending {ensure!(d.map==owner.map,"battle summon portal changed map");}
        Ok(owner.phase)
    }
}
