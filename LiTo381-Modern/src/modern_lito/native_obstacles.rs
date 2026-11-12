//! Dynamic occupancy qualified against the deployed arena object contracts.
use super::native_world::{Actor, World};
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum EncounterMode { Quest, TimeAttack }
impl EncounterMode {
    /// Native room wire modes, not map IDs or inferred sprite families.
    pub fn from_room_mode(mode: u8) -> Option<Self> {
        match mode { 2 => Some(Self::Quest), 4 => Some(Self::TimeAttack), _ => None }
    }
}
pub fn guardian(a:&Actor)->bool {
    (0x6b000000..=0x6bffffff).contains(&a.id)
        && matches!(a.graphics,2665|2667|2669|2671)
        && matches!(a.walk_action,0|32|33|34|35)
}
pub fn destroyed_guardian(a:&Actor)->bool{guardian(a)&&a.walk_action==35}
/// Deployed BattleQuestEncounter / BattleTimeAttackEncounter reserve these
/// object namespaces. Players, companions and decorative sprites do not block.
/// Do not use HP ratio: a living low-HP actor can have a rounded zero bar.
pub fn encounter_actor(mode: Option<EncounterMode>, a: &Actor) -> bool {
    let namespace = match mode {
        Some(EncounterMode::Quest) => 0x6c000000..=0x6cffffff,
        Some(EncounterMode::TimeAttack) => 0x6d000000..=0x6dffffff,
        None => return false,
    };
    namespace.contains(&a.id) && a.graphics != 0 && a.walk_action == 0
}
pub fn blocks(w:&World,x:u16,y:u16)->bool {
    super::native_doors::blocks(w,x,y) || w.actors.values().any(|a|
        a.x == x && a.y == y && ((guardian(a) && a.walk_action != 35)
            || (encounter_actor(w.encounter_mode, a) && !a.dead)))
}
