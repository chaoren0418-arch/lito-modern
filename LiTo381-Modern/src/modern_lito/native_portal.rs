//! Server-authorized S_PORTAL destinations; requests never predict a map transition.
use anyhow::{ensure,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Destination{pub map:u16,pub x:u16,pub y:u16}
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Pending{pub destination:Destination,pub object_id:u32}
pub fn parse(body:&[u8])->Result<Destination>{
 ensure!(body.len()==7&&body[0]==0x5a,"invalid S_PORTAL layout");
 Ok(Destination{map:u16::from_le_bytes([body[1],body[2]]),x:u16::from_le_bytes([body[3],body[4]]),y:u16::from_le_bytes([body[5],body[6]])})
}
pub const ACK:[u8;1]=[0x08];

/// Skill 5 UI selects random travel or a received bookmark. Only a live controller-owned
/// request grants this path; the returned coordinates remain server authority.
pub fn admit_teleport_spell(s: &super::native_login::Snapshot, d: Destination, requested_map:u16) -> Result<()> {
    ensure!(s.game_connected && s.phase.active_world() && !s.observer && !s.world.observer_mode
        && s.world.ready() && s.world.battle_result.is_none()
        && s.world.hp.is_some_and(|hp| hp[0] > 0)
        && s.world.own_actor().is_some_and(|a| !a.dead)
        && s.world.learned_skills.contains(&5), "指定傳送角色已失效");
    ensure!(d.map==requested_map, "指定傳送回覆超出所選地圖");
    ensure!(s.phase!=super::native_login::LoginPhase::BattlePending || s.world.map_id==Some(d.map), "戰鬥傳送不可跨地圖");
    Ok(())
}

/// An original-map source entrance, never a client-supplied destination.
#[derive(Clone,Debug)]
pub struct EntryChoice {
    stamp: super::native_navigation::InputStamp,
    pub source:[u16;2],
    position:[u16;2],
    map:u16,
}
impl EntryChoice {
    pub fn capture(epoch:u64,w:&super::native_world::World,nav:&super::native_navigation::Navigation,source:[u16;2])->Result<Self>{
        let own=w.own_actor().ok_or_else(||anyhow::anyhow!("尚未收到角色"))?;
        let choice=Self{stamp:super::native_navigation::InputStamp::capture(epoch,w),source,position:[own.x,own.y],map:w.map_id.ok_or_else(||anyhow::anyhow!("尚未收到地圖"))?};
        choice.request(epoch,w,nav)?;
        Ok(choice)
    }
    pub fn nearest(epoch:u64,w:&super::native_world::World,nav:&super::native_navigation::Navigation)->Result<Self>{
        let own=w.own_actor().ok_or_else(||anyhow::anyhow!("尚未收到角色"))?;
        let mut nearby:Vec<_>=nav.portal_sources().map(|p|(own.x.abs_diff(p[0]).max(own.y.abs_diff(p[1])),p)).filter(|(d,_)|*d<=3).collect();
        nearby.sort_unstable();
        let (distance,source)=*nearby.first().ok_or_else(||anyhow::anyhow!("附近沒有可使用的地圖入口"))?;
        ensure!(nearby.get(1).is_none_or(|(d,_)|*d!=distance),"附近有多個等距入口，請靠近要使用的入口");
        Self::capture(epoch,w,nav,source)
    }
    pub fn request(&self,epoch:u64,w:&super::native_world::World,nav:&super::native_navigation::Navigation)->Result<Vec<u8>>{
        ensure!(self.stamp==super::native_navigation::InputStamp::capture(epoch,w),"入口選取已過期");
        let own=w.own_actor().ok_or_else(||anyhow::anyhow!("角色已離開場景"))?;
        ensure!(w.ready()&&!w.observer_mode&&w.battle_result.is_none()&&w.hp.is_some_and(|h|h[0]>0)&&!own.dead&&w.movement_blocks.is_empty(),"目前無法使用入口");
        ensure!(w.map_id==Some(self.map)&&nav.map_id==u32::from(self.map)&&[own.x,own.y]==self.position&&nav.has_portal_source(self.source),"入口或角色位置已變更");
        ensure!(own.x.abs_diff(self.source[0]).max(own.y.abs_diff(self.source[1]))<=3,"入口距離超出範圍");
        let mut body=vec![0x73];body.extend(self.source[0].to_le_bytes());body.extend(self.source[1].to_le_bytes());Ok(body)
    }
    pub fn current(&self,epoch:u64,w:&super::native_world::World)->bool{
        self.stamp==super::native_navigation::InputStamp::capture(epoch,w)&&w.ready()&&!w.observer_mode&&w.battle_result.is_none()&&w.hp.is_some_and(|h|h[0]>0)&&w.movement_blocks.is_empty()&&w.own_actor().is_some_and(|a|[a.x,a.y]==self.position)
    }
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,serde::Serialize)]
pub enum EntryFeedback{#[default] Idle,Pending,SceneUpdated,NoResponse,Interrupted}
impl EntryFeedback{
    pub fn label(self)->&'static str{match self{Self::Idle=>"",Self::Pending=>"入口：等待回覆",Self::SceneUpdated=>"入口：場景已更新",Self::NoResponse=>"入口：沒有回覆，請重新操作",Self::Interrupted=>"入口：操作已取消"}}
}
pub struct EntryPending{pub choice:EntryChoice,pub since:std::time::Instant}
impl EntryPending{
    pub fn observe(&self,epoch:u64,w:&super::native_world::World,now:std::time::Instant)->EntryFeedback{
        if !self.choice.current(epoch,w){EntryFeedback::Interrupted}
        else if now.saturating_duration_since(self.since)>=std::time::Duration::from_secs(4){EntryFeedback::NoResponse}
        else{EntryFeedback::Pending}
    }
}

/// Qualified hotel map changes and formal ordinary-battle same-map travel.
/// The server chooses the destination. No character/room/observer transition
/// or unqualified ordinary cross-map portal is admitted here.
pub fn admit(s: &super::native_login::Snapshot, destination: Destination) -> Result<()> {
    use super::native_login::LoginPhase;
    ensure!(s.phase == LoginPhase::BattlePending && s.game_connected,
        "portal outside active player world");
    let room=s.joined_room.as_ref().ok_or_else(||anyhow::anyhow!("portal without room assignment"))?;
    // Preserve the established rental portal admission; World::accept still
    // checks ready ownership and resets the scene. The new ordinary travel
    // branch has the formal GmTravel living/non-observer/same-map constraints.
    if room.mode != 6 {
        ensure!(!s.observer && !s.world.observer_mode && s.world.ready()
            && s.world.battle_result.is_none() && s.world.hp.is_some_and(|hp|hp[0]>0)
            && s.world.own_actor().is_some_and(|a|!a.dead), "portal without ready living actor");
        ensure!(s.world.map_id == Some(destination.map), "unqualified ordinary cross-map portal");
    }
    Ok(())
}

/// Qualified skill131 contract: exact retained destination and an explicit caller-owned request.
/// LoginController calls this only for its current generation/object request within two seconds.
pub fn admit_world_tree(s:&super::native_login::Snapshot,d:Destination)->Result<()> {
    use super::native_login::LoginPhase;
    ensure!(s.phase.active_world()&&s.game_connected&&!s.observer&&!s.world.observer_mode
        &&s.world.ready()&&s.world.battle_result.is_none()&&s.world.hp.is_some_and(|h|h[0]>0)
        &&s.world.own_actor().is_some_and(|a|!a.dead)&&s.world.movement_blocks.is_empty(),"world tree without active living owner");
    ensure!(d==Destination{map:4,x:33051,y:32337},"unqualified world tree destination");
    ensure!(s.world.learned_skills.contains(&131),"world tree not learned");
    ensure!(if s.phase==LoginPhase::BattlePending{s.joined_room.as_ref().is_some_and(|r|r.mode!=6)}else{s.joined_room.is_none()},"world tree membership mismatch");
    Ok(())
}
