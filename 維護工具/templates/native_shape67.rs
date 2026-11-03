//! Formal S2A monlist grants and one-shot C_HACTION, independent of the caster's book.
use super::{native_inventory, native_login::{LoginPhase, Snapshot}};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Form { pub(crate) action: &'static str, pub(crate) minimum_level: u8 }
// @STANDARD_MENU_FORMS@

#[derive(Clone, Debug, PartialEq, Eq)]
struct Owner {
    epoch: u64, generation: u64, map: u16, phase: LoginPhase, room: Option<u32>,
    source: String, inventory: native_inventory::Owner, actor: (u32, u64, u64),
    interruptions: u64,
}
impl Owner {
    fn current(s: &Snapshot, epoch: u64) -> Option<Self> {
        let w=&s.world; let a=w.own_actor()?; let inv=s.inventory.owner.as_ref()?;
        if !s.game_connected || !s.phase.active_world() || s.observer || w.observer_mode
            || !w.ready() || w.battle_result.is_some() || a.dead
            || !w.hp.is_some_and(|h|h[0]>0) || inv.connection!=epoch
            || s.selected_character.as_deref()!=Some(inv.character.as_str()) { return None; }
        Some(Self { epoch, generation:w.generation, map:w.map_id?, phase:s.phase,
            room:s.joined_room.as_ref().map(|r|r.id), source:s.selected_source.clone()?,
            inventory:inv.clone(), actor:(a.id,a.incarnation,a.target_revision),
            interruptions:w.control_interruptions })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Control { List=0, Apply=1, Close=2 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token { serial:u16, owner:Owner }
impl Token {
    /// Never reuse an old native command ID in this window's lifetime.
    pub(crate) fn command_id(&self, control:Control)->u16 {1000+self.serial*3+control as u16}
}
#[derive(Clone, Debug)]
pub(crate) struct Prompt { token:Token }
impl Prompt { pub(crate) fn token(&self)->Token {self.token.clone()} }
#[derive(Default)]
pub(crate) struct Session { serial:u16, prompt:Option<Prompt> }

#[derive(Debug, PartialEq, Eq)]
enum Dialog { Open(u32), Close(u32), Other(u32) }
fn parse(body:&[u8])->Result<Option<Dialog>> {
    if body.first()!=Some(&0x2a) {return Ok(None);}
    ensure!((9..=65533).contains(&body.len()),"S_HYPERTEXT envelope");
    let id=u32::from_le_bytes(body[1..5].try_into().unwrap()); ensure!(id!=0,"hypertext object id");
    fn string<'a>(body:&'a [u8], at:&mut usize)->Result<&'a [u8]> {
        let tail=body.get(*at..).context("hypertext string offset")?;
        let n=tail.iter().position(|b|*b==0).context("unterminated hypertext string")?;
        *at+=n+1; Ok(&tail[..n])
    }
    let mut at=5; let html=string(body,&mut at)?; let parameter=string(body,&mut at)?;
    let count=body.get(at..at+2).context("hypertext argument count")?;
    let count=usize::from(u16::from_le_bytes(count.try_into().unwrap())); at+=2;
    ensure!(count<=body.len()-at,"hypertext arguments truncated");
    for _ in 0..count {string(body,&mut at)?;}
    ensure!(at==body.len(),"hypertext trailing data");
    if html==b"monlist" {
        ensure!(parameter.is_empty()&&count==0,"unexpected monlist parameters");
        Ok(Some(Dialog::Open(id)))
    } else if html.is_empty() {
        ensure!(parameter.is_empty()&&count==0,"unexpected hypertext close parameters");
        Ok(Some(Dialog::Close(id)))
    } else {Ok(Some(Dialog::Other(id)))}
}
impl Session {
    pub(crate) fn current(&self,s:&Snapshot,epoch:u64)->Option<&Prompt> {
        self.prompt.as_ref().filter(|p|Owner::current(s,epoch).as_ref()==Some(&p.token.owner))
    }
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64) {
        if self.current(s,epoch).is_none() {self.prompt=None;}
    }
    pub(crate) fn retire(&mut self) {self.prompt=None;}
    pub(crate) fn receive(&mut self,s:&Snapshot,epoch:u64,body:&[u8])->Result<bool> {
        // Parse completely before changing a grant, including malformed close packets.
        let Some(dialog)=parse(body)? else{return Ok(false);};
        let own=s.world.own_actor().map(|a|a.id);
        match dialog {
            Dialog::Open(id)=> {
                // A monlist for another actor must never become an NPC/rental dialog.
                if own!=Some(id) {return Ok(true);}
                self.prompt=None;
                let Some(owner)=Owner::current(s,epoch) else{return Ok(true);};
                ensure!(self.serial<20_000,"變形選單命令已用盡，請重新開啟視窗"); self.serial+=1;
                self.prompt=Some(Prompt{token:Token{serial:self.serial,owner}}); Ok(true)
            }
            Dialog::Close(id) if own==Some(id)=> {self.prompt=None;Ok(true)}
            Dialog::Other(id) if own==Some(id)=> {self.prompt=None;Ok(false)}
            _=>Ok(false),
        }
    }
    /// UI indices: zero explicitly removes polymorph; 1..=96 are catalog actions.
    /// A dismissed window is local and must never serialize the action "none".
    pub(crate) fn answer(&mut self,s:&Snapshot,epoch:u64,token:&Token,index:usize)->Result<Vec<u8>> {
        let p=self.current(s,epoch).context("變形選單已失效")?;
        ensure!(&p.token==token,"變形選單已更新，請重新選取");
        let action=if index==0 {"none"} else {FORMS.get(index-1).context("請明確選取變形種類")?.action};
        let id=p.token.owner.actor.0;
        self.prompt=None; // consume before serialization/send; no transport retry authority
        let mut body=vec![0x27]; body.extend(id.to_le_bytes());
        body.extend(action.as_bytes()); body.extend([0,0]); Ok(body)
    }
    pub(crate) fn dismiss(&mut self,s:&Snapshot,epoch:u64,token:&Token)->Result<()> {
        ensure!(self.current(s,epoch).is_some_and(|p|&p.token==token),"變形選單已失效");
        self.prompt=None; Ok(())
    }
}
