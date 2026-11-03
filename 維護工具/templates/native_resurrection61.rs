//! Exact S24/321 consent, owned by the original dead life; effects never acknowledge it.
use super::{native_login::{Snapshot,LoginPhase},native_inventory};
use anyhow::{Result,ensure,Context};
#[derive(Clone,Debug,PartialEq,Eq)]
struct Owner {epoch:u64,generation:u64,map:u16,phase:LoginPhase,room:Option<u32>,source:String,inventory:native_inventory::Owner,actor:(u32,u64,u64)}
impl Owner {
    fn current(s:&Snapshot,epoch:u64)->Option<Self> {
        let w=&s.world;let a=w.own_actor()?;let inv=s.inventory.owner.as_ref()?;
        if !s.phase.active_world()||!s.game_connected||s.observer||!w.ready()||w.battle_result.is_some()||!a.dead
            ||inv.connection!=epoch||s.selected_character.as_deref()!=Some(inv.character.as_str()){return None;}
        Some(Self{epoch,generation:w.generation,map:w.map_id?,phase:s.phase,room:s.joined_room.as_ref().map(|r|r.id),source:s.selected_source.clone()?,inventory:inv.clone(),actor:(a.id,a.incarnation,a.action_sequence)})
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]pub(crate) struct Token {serial:u16,owner:Owner}
impl Token {
    /// Distinct native command IDs prevent queued clicks from accepting a replacement ask.
    /// Never wrap IDs back onto an earlier question in this controller lifetime.
    pub(crate) fn command_id(&self,yes:bool)->u16 {1000+self.serial*2+u16::from(yes)}
}
#[derive(Clone,Debug)]pub(crate) struct Prompt {token:Token,waiting:bool}
impl Prompt {
    pub(crate) fn token(&self)->Token{self.token.clone()}
    pub(crate) fn waiting(&self)->bool{self.waiting}
}
#[derive(Default)]pub(crate) struct Session {serial:u16,prompt:Option<Prompt>}
pub(crate) fn parse(body:&[u8])->Result<Option<u16>> {
    if body.first()!=Some(&0x24){return Ok(None);}ensure!(body.len()>=3,"truncated native ask");
    let id=u16::from_le_bytes([body[1],body[2]]);if id!=321{return Ok(None);}ensure!(body.len()==3,"resurrection ask tail");Ok(Some(id))
}
impl Session {
    pub(crate) fn current(&self,s:&Snapshot,epoch:u64)->Option<&Prompt>{self.prompt.as_ref().filter(|p|Owner::current(s,epoch).as_ref()==Some(&p.token.owner))}
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64){if self.current(s,epoch).is_none(){self.prompt=None;}}
    pub(crate) fn retire(&mut self){self.prompt=None;}
    pub(crate) fn receive(&mut self,s:&Snapshot,epoch:u64,body:&[u8])->Result<bool> {
        if body.first()!=Some(&0x24){return Ok(false);}let id=parse(body)?;
        self.prompt=None;let Some(_)=id else{return Ok(false);};
        if !s.world.hp.is_some_and(|h|h[0]==0){return Ok(true);}let Some(owner)=Owner::current(s,epoch)else{return Ok(true);};
        ensure!(self.serial<30_000,"native resurrection question command IDs exhausted; reconnect using a new window");self.serial+=1;
        self.prompt=Some(Prompt{token:Token{serial:self.serial,owner},waiting:false});Ok(true)
    }
    /// Consume consent before transport send; failure cannot authorize a resend.
    /// A later explicit return abandons waiting locally, without a second C23.
    pub(crate) fn answer(&mut self,s:&Snapshot,epoch:u64,token:&Token,yes:bool)->Result<Option<Vec<u8>>> {
        let current=self.current(s,epoch).context("復活詢問已失效")?;ensure!(&current.token==token,"復活詢問已更新，請重新選擇");
        if current.waiting {ensure!(!yes,"已送出同意，等待伺服器結果");self.prompt=None;return Ok(None);}
        ensure!(!yes||s.world.hp.is_some_and(|h|h[0]==0),"生命狀態已變更，等待伺服器同步");
        if yes{self.prompt.as_mut().unwrap().waiting=true;}else{self.prompt=None;}
        Ok(Some(vec![0x23,0x41,0x01,u8::from(yes),0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot()->Snapshot {let mut s=super::super::native_login::LoginController::default().snapshot().clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();s.world.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}s.phase=LoginPhase::BattlePending;s.game_connected=true;s.selected_source=Some("isolated".into());s.selected_character=Some("fixture".into());s.inventory.owner=Some(native_inventory::Owner{connection:7,selection:1,character:"fixture".into()});let id=s.world.own_actor().unwrap().id;s.world.actors.get_mut(&id).unwrap().dead=true;s.world.hp=Some([0,100]);s}
    #[test]fn exact_ask_does_not_accept_a_string_tail_or_confuse_another_question(){assert_eq!(parse(&[36,65,1]).unwrap(),Some(321));assert!(parse(&[36,65,1,0]).is_err());assert!(parse(&[36,65]).is_err());assert_eq!(parse(&[36,66,1,0]).unwrap(),None);assert_eq!(parse(&[35,65,1]).unwrap(),None);}
    #[test]fn yes_is_exact_and_consumed_once_before_any_transport_or_effect(){let s=snapshot();let mut q=Session::default();q.receive(&s,7,&[36,65,1]).unwrap();let t=q.current(&s,7).unwrap().token();let before=serde_json::to_value(&s.world).unwrap();assert_eq!(q.answer(&s,7,&t,true).unwrap(),Some(vec![35,65,1,1,0]));assert!(q.current(&s,7).unwrap().waiting());assert!(q.answer(&s,7,&t,true).is_err());assert_eq!(serde_json::to_value(&s.world).unwrap(),before);}
    #[test]fn no_is_exact_and_waiting_return_cannot_send_a_second_answer(){let s=snapshot();let mut q=Session::default();q.receive(&s,7,&[36,65,1]).unwrap();let t=q.current(&s,7).unwrap().token();assert_eq!(q.answer(&s,7,&t,false).unwrap(),Some(vec![35,65,1,0,0]));assert!(q.answer(&s,7,&t,false).is_err());q.receive(&s,7,&[36,65,1]).unwrap();let t=q.current(&s,7).unwrap().token();q.answer(&s,7,&t,true).unwrap();assert!(q.answer(&s,7,&t,false).unwrap().is_none());assert!(q.current(&s,7).is_none());}
    #[test]fn replacement_revokes_token_and_native_command_id_even_in_same_dead_life(){let s=snapshot();let mut q=Session::default();q.receive(&s,7,&[36,65,1]).unwrap();let old=q.current(&s,7).unwrap().token();q.receive(&s,7,&[36,65,1]).unwrap();let new=q.current(&s,7).unwrap().token();assert_ne!(old.command_id(true),new.command_id(true));assert!(q.answer(&s,7,&old,true).is_err());q.receive(&s,7,&[36,66,1,0]).unwrap();assert!(q.answer(&s,7,&new,true).is_err());}
    #[test]fn every_owner_boundary_rejects_before_serializing_an_answer(){let s=snapshot();let mut q=Session::default();q.receive(&s,7,&[36,65,1]).unwrap();let t=q.current(&s,7).unwrap().token();assert!(q.answer(&s,8,&t,true).is_err());for n in 0..8{let mut s=s.clone();let id=s.world.own_actor().unwrap().id;match n{0=>s.world.generation+=1,1=>s.world.map_id=Some(s.world.map_id.unwrap()+1),2=>s.selected_source=Some("other".into()),3=>s.inventory.owner.as_mut().unwrap().selection+=1,4=>s.world.actors.get_mut(&id).unwrap().incarnation+=1,5=>s.world.actors.get_mut(&id).unwrap().action_sequence+=1,6=>s.phase=LoginPhase::WorldReady,_=>s.game_connected=false};assert!(q.answer(&s,7,&t,true).is_err());}}
    #[test]fn live_observer_missing_hp_and_unowned_asks_never_authorize_consent(){for n in 0..5{let mut s=snapshot();let id=s.world.own_actor().unwrap().id;match n{0=>s.world.actors.get_mut(&id).unwrap().dead=false,1=>s.observer=true,2=>s.world.hp=None,3=>s.world.hp=Some([1,100]),_=>s.inventory.owner=None};let mut q=Session::default();assert!(q.receive(&s,7,&[36,65,1]).unwrap());assert!(q.current(&s,7).is_none());}}
    #[test]fn authoritative_new_life_retires_prompt_and_hp_alone_never_acknowledges_it(){let mut s=snapshot();let mut q=Session::default();q.receive(&s,7,&[36,65,1]).unwrap();let t=q.current(&s,7).unwrap().token();q.answer(&s,7,&t,true).unwrap();s.world.hp=Some([50,100]);assert!(q.current(&s,7).unwrap().waiting());let own=s.world.own_actor().unwrap().clone();let mut b=vec![0x11];b.extend(own.id.to_le_bytes());b.push(0);b.extend(2_u32.to_le_bytes());b.extend(own.graphics.to_le_bytes());s.world.accept(&b).unwrap();q.synchronize(&s,7);assert!(q.current(&s,7).is_none());assert!(q.answer(&s,7,&t,true).is_err());}
}
