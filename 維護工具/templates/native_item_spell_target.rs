/// Inventory spell intent is a separate authority domain from actor IDs.
pub fn item_targeted(skill:u16)->bool {matches!(skill,12|21)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ItemTarget {
    choice:super::native_item_actions::Choice,
    stamp:super::native_navigation::InputStamp,
    inventory_revision:u64,
    map:Option<u16>,
}
impl ItemTarget {
    pub fn capture(snapshot:&super::native_login::Snapshot,connection:u64,id:u32)->Result<Self> {
        let choice=super::native_item_actions::Choice::capture(snapshot,id)?;
        let value=Self{choice,stamp:super::native_navigation::InputStamp::capture(connection,&snapshot.world),
            inventory_revision:snapshot.inventory.revision,map:snapshot.world.map_id};
        value.validate(snapshot,connection)?;Ok(value)
    }
    pub fn object_id(&self)->u32 {self.choice.item.id}
    pub fn validate(&self,snapshot:&super::native_login::Snapshot,connection:u64)->Result<()> {
        ensure!(snapshot.game_connected&&snapshot.phase.active_world()&&!snapshot.observer&&!snapshot.world.observer_mode
            &&snapshot.world.ready(),"請在世界中選取背包物品");
        self.choice.validate(snapshot)?;
        ensure!(self.choice.owner.connection==connection&&snapshot.inventory.revision==self.inventory_revision
            &&snapshot.world.map_id==self.map
            &&self.stamp==super::native_navigation::InputStamp::capture(connection,&snapshot.world),
            "背包、角色或場景已更新，請重新選取附魔物品");
        Ok(())
    }
}
pub fn item_request(snapshot:&super::native_login::Snapshot,connection:u64,skill:u16,target:&ItemTarget)->Result<Vec<u8>> {
    ensure!(item_targeted(skill),"此技能不是物品附魔");
    target.validate(snapshot,connection)?;can_cast(&snapshot.world,skill)?;
    // Template kind, eligibility, cost and actual item effect stay on server.
    let mut body=vec![0x14,((skill-1)/8)as u8,((skill-1)%8)as u8];
    body.extend_from_slice(&target.object_id().to_le_bytes());Ok(body)
}
