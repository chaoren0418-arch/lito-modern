//! Formal arena door authority. Other S_CHANGE_ATTR users remain recorded but
//! are not guessed to be doors from coordinates alone.
use super::native_world::{Actor, World};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Attribute {
    pub position: [u16;2],
    pub direction: u8,
    pub passable: u8,
}
impl Attribute {
    pub fn key(self)->u64 {u64::from(self.position[0])|(u64::from(self.position[1])<<16)|(u64::from(self.direction)<<32)}
}
pub fn decode(b:&[u8])->Result<Attribute>{
    ensure!(b.len()==7 && b[0]==0x22,"door attribute envelope");
    ensure!(b[5]<=1 && matches!(b[6],0|65),"door attribute domain");
    Ok(Attribute{position:[u16::from_le_bytes([b[1],b[2]]),u16::from_le_bytes([b[3],b[4]])],direction:b[5],passable:b[6]})
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Record {
    pub attribute: Attribute,
    pub sequence: u64,
    pub owner: Option<(u32,u64)>,
}
/// Protocol.battleDoor identifies this arena object with gfx 2083 and a held
/// open/closed action. Castle doors have separate contracts still to qualify.
pub fn is_door(a:&Actor)->bool {a.graphics==2083 && matches!(a.walk_action,28|29)}
fn current_owner<'a>(w:&'a World,r:&Record)->Option<&'a Actor>{
    let (id,incarnation)=r.owner?;
    w.actors.get(&id).filter(|a|a.incarnation==incarnation && is_door(a) && [a.x,a.y]==r.attribute.position)
}
pub fn record(w:&World,id:u32)->Option<&Record>{
    let mut rows=w.door_attributes.values().filter(|r|current_owner(w,r).is_some_and(|a|a.id==id));
    let first=rows.next()?;rows.next().is_none().then_some(first)
}
pub fn blocks(w:&World,x:u16,y:u16)->bool {
    w.door_attributes.values().any(|r|r.attribute.position==[x,y]&&r.attribute.passable==65&&current_owner(w,r).is_some())
}
/// Unowned attributes are directional terrain updates (including boundaries).
/// Qualified arena doors use BattleDoor.blocks(destination) instead: their
/// attribute must not trap an actor already on the closing door's source cell,
/// or open a static wall the server still checks independently.
pub fn edge_override(w: &World, x: u16, y: u16, direction: u8) -> Option<bool> {
    let key = Attribute { position: [x, y], direction, passable: 0 }.key();
    let r = w.door_attributes.get(&key)?;
    r.owner.is_none().then_some(r.attribute.passable == 65)
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Choice {
    pub epoch:u64,pub generation:u64,pub id:u32,pub incarnation:u64,
    pub attribute:Attribute,pub sequence:u64,
}
impl Choice {
    pub fn capture(epoch:u64,w:&World,id:u32)->Result<Self>{
        let a=w.actors.get(&id).context("門已離開視野")?;
        let r=record(w,id).context("尚未取得可操作的門狀態")?;
        ensure!(w.actor_visible(id),"門不在目前視野");
        Ok(Self{epoch,generation:w.generation,id,incarnation:a.incarnation,attribute:r.attribute,sequence:r.sequence})
    }
    pub fn request(&self,epoch:u64,w:&World)->Result<Vec<u8>>{
        ensure!(self.epoch==epoch&&self.generation==w.generation,"門選取已過期");
        let a=w.actors.get(&self.id).context("門已離開視野")?;
        let r=record(w,self.id).context("門狀態已失效")?;
        ensure!(a.incarnation==self.incarnation&&r.sequence==self.sequence&&r.attribute==self.attribute
            &&w.actor_visible(self.id),"門已變更，請重新選取");
        ensure!(w.ready()&&w.battle_result.is_none()&&w.hp.is_some_and(|v|v[0]>0)&&w.movement_blocks.is_empty(),"目前不能開關門");
        let own=w.own_actor().context("角色尚未載入")?;
        ensure!(!own.dead&&own.x.abs_diff(a.x)<=2&&own.y.abs_diff(a.y)<=1,"請靠近門再操作");
        ensure!(a.walk_action==if r.attribute.passable==65 {29}else{28},"等待門狀態同步");
        let mut body=vec![0x0e];body.extend(a.x.to_le_bytes());body.extend(a.y.to_le_bytes());body.extend(a.id.to_le_bytes());Ok(body)
    }
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,serde::Serialize)]
pub enum Feedback {#[default] Idle,Pending,Changed,NoResponse,Interrupted}
pub struct Pending {pub choice:Choice,pub since:Instant}
impl Pending {
    pub fn observe(&self,epoch:u64,w:&World,now:Instant)->Feedback{
        let c=&self.choice;
        if epoch!=c.epoch||w.generation!=c.generation||w.battle_result.is_some()
            ||w.own_actor().is_none_or(|a|a.dead)||w.actors.get(&c.id).is_none_or(|a|a.incarnation!=c.incarnation){return Feedback::Interrupted;}
        if let Some(r)=record(w,c.id){
            let expected=if c.attribute.passable==65{0}else{65};
            if r.sequence>c.sequence&&r.attribute.passable==expected
                &&w.actors[&c.id].walk_action==if expected==0 {28}else{29}{return Feedback::Changed;}
        }
        if now.saturating_duration_since(self.since)>=Duration::from_secs(2){Feedback::NoResponse}else{Feedback::Pending}
    }
}
