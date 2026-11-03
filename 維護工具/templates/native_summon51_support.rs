
/// Wire selections are catalog values, never actor, NPC or inventory object IDs.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct SummonSelection {pub selection:u32,pub label:&'static str,pub required_level:u16}
pub fn summon_targeted(skill:u16)->bool {matches!(skill,51)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct SummonTarget {
    selection:u32,stamp:super::native_navigation::InputStamp,
    inventory_owner:super::native_inventory::Owner,inventory_revision:u64,map:Option<u16>,
}
impl SummonTarget {
    pub fn capture(snapshot:&super::native_login::Snapshot,connection:u64,selection:u32)->Result<Self> {
        ensure!(SUMMON_SELECTIONS.iter().any(|s|s.selection==selection),"召喚選項無效");
        let owner=snapshot.inventory.owner.clone().context("等待角色背包")?;
        let value=Self{selection,stamp:super::native_navigation::InputStamp::capture(connection,&snapshot.world),
            inventory_owner:owner,inventory_revision:snapshot.inventory.revision,map:snapshot.world.map_id};
        value.validate(snapshot,connection)?;Ok(value)
    }
    pub fn selection(&self)->u32 {self.selection}
    pub fn validate(&self,snapshot:&super::native_login::Snapshot,connection:u64)->Result<()> {
        ensure!(snapshot.game_connected&&snapshot.phase.active_world()&&!snapshot.observer&&!snapshot.world.observer_mode,
            "請在世界中選擇召喚種類");
        can_cast(&snapshot.world,51)?;
        ensure!(self.inventory_owner.connection==connection
            &&snapshot.inventory.owner.as_ref()==Some(&self.inventory_owner)
            &&snapshot.inventory.revision==self.inventory_revision&&snapshot.world.map_id==self.map
            &&self.stamp==super::native_navigation::InputStamp::capture(connection,&snapshot.world),
            "角色、裝備或場景已更新，請重新選擇召喚種類");
        Ok(())
    }
}
pub fn summon_request(snapshot:&super::native_login::Snapshot,connection:u64,target:&SummonTarget)->Result<Vec<u8>> {
    target.validate(snapshot,connection)?;
    // Server controlAbility5 chooses this value, otherwise server directGrade.
    // No local ring/material/capacity/HP or NPC-template inference.
    let mut bytes=vec![0x14,6,2];bytes.extend(target.selection.to_le_bytes());Ok(bytes)
}
impl Pending {
    pub fn observe_summon(&self,world:&World,now:Instant)->Feedback {
        let ordinary=self.observe(world,now);
        if ordinary==Feedback::Interrupted{return ordinary;}
        // Original map/level failures can follow the caster action and effect.
        if world.message_count>self.messages {
            if let Some((code,_))=&world.last_message {if matches!(code,353|743){return Feedback::Rejected(*code);}}
        }
        ordinary
    }
}
