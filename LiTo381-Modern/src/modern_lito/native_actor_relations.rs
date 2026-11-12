//! Packet-owned relation fields used by the original water-light classifier.
//! Names retain MS950 bytes: decoding to Unicode loses duplicate encodings.
use super::{native_light_sources::Master, native_world::Actor};
use anyhow::{ensure, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit MS950 policy: DBCS pairs compare exactly; ASCII letters fold only
/// outside pairs. The old process's unresolved ACP/table initializer is not
/// qualified by this policy. Inputs here have already passed the wire decoder.
pub fn equal_ms950(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {return false;}
    let mut i = 0;
    while i < left.len() {
        let a = left[i]; let b = right[i];
        if (0x81..=0xfe).contains(&a) {
            if a != b || i+1 >= left.len() || left[i+1] != right[i+1] {return false;}
            i += 2;
        } else {
            if !a.eq_ignore_ascii_case(&b) {return false;}
            i += 1;
        }
    }
    true
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Flag {
    set: bool,
    /// S36 clears only classes11/6/12. Resolve metadata lazily, retaining the
    /// graphics at each transition, never the actor's later morph graphics.
    clear_graphics: BTreeSet<u16>,
}
impl Flag {
    pub fn reset(&mut self, set: bool) {self.set=set; self.clear_graphics.clear();}
    fn clear_for(&mut self, graphics:u16)->Result<()> {
        if self.set {
            ensure!(self.clear_graphics.contains(&graphics) || self.clear_graphics.len()<256,
                "relation transition graphics budget");
            self.clear_graphics.insert(graphics);
        }
        Ok(())
    }
    pub fn resolve(&self, mut base_class:impl FnMut(u16)->Result<u8>)->Result<bool> {
        if !self.set {return Ok(false);}
        for graphics in &self.clear_graphics {
            if matches!(base_class(*graphics)?,11|6|12) {return Ok(false);}
        }
        Ok(true)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Relations {
    pub name:Vec<u8>,
    pub clan:Vec<u8>,
    pub master:Vec<u8>,
    pub flag_1f:Flag,
    pub side_20:u8,
}
impl Relations {
    pub fn master_kind(&self, own_name:Option<&[u8]>)->Master {
        if self.master.is_empty() {Master::None}
        else if own_name.is_some_and(|name|equal_ms950(&self.master,name)) {Master::Own}
        else {Master::Other}
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Context {
    owner:Option<u64>,
    pub active:bool,
    pub own_clan:Vec<u8>,
    /// Native insertion retains duplicates; end-war removes the first match.
    pub enemies:Vec<Vec<u8>>,
}
impl Context {
    pub fn begin(&mut self, epoch:u64)->Result<()> {
        ensure!(epoch!=0,"zero relation GAME epoch");
        if self.owner==Some(epoch) {return Ok(());}
        *self=Self{owner:Some(epoch),..Default::default()}; Ok(())
    }
    pub fn retire(&mut self) {*self=Self::default();}
    pub fn owner(&self)->Option<u64> {self.owner}
    /// S0B resets1F, preserves an existing actor's20, then applies context and
    /// finally status bit4. Same-clan pets owned by the player skip the setter.
    pub fn put(&self, actor:&mut Actor, own_name:Option<&[u8]>) {
        let a=&mut actor.relations;
        a.flag_1f.reset(false);
        if self.active && !a.clan.is_empty() {
            if self.enemies.iter().any(|clan|equal_ms950(&a.clan,clan)) {
                a.side_20=2; a.flag_1f.reset(true);
            }
            if equal_ms950(&a.clan,&self.own_clan) {
                a.side_20=1;
                if a.master_kind(own_name)!=Master::Own {a.flag_1f.reset(true);}
            }
        }
        if actor.status&4!=0 {a.flag_1f.reset(true);}
    }
    /// S69/4E7690 differs from S0B: it does not reset1F and has no own-master
    /// exception. An empty clan clears20 only while the war gate is active.
    pub fn clan_changed(&self, a:&mut Relations) {
        if !self.active {return;}
        if a.clan.is_empty() {a.side_20=0; return;}
        if self.enemies.iter().any(|clan|equal_ms950(&a.clan,clan)) {
            a.side_20=2; a.flag_1f.reset(true);
        }
        if equal_ms950(&a.clan,&self.own_clan) {a.side_20=1; a.flag_1f.reset(true);}
    }
    /// Caller stages a cloned Context. Return replacement relation fields so
    /// resource-budget errors cannot leave partially updated live actors.
    pub fn notice(&mut self, actors:&BTreeMap<u32,Actor>, own_clan:Option<&[u8]>,
        kind:u8, first:&[u8], second:&[u8])->Result<Vec<(u32,Relations)>> {
        if !matches!(kind,1|2|3|4|8) {return Ok(Vec::new());}
        let Some(own)=own_clan.filter(|v|!v.is_empty()) else {return Ok(Vec::new());};
        let (own,enemy)=if equal_ms950(own,first) {(first,second)}
            else if equal_ms950(own,second) {(second,first)} else {return Ok(Vec::new());};
        let declaring=matches!(kind,1|8);
        if declaring {
            ensure!(self.enemies.len()<2048,"relation enemy clan budget");
            self.enemies.push(enemy.to_vec()); self.own_clan=own.to_vec(); self.active=true;
        } else {
            if let Some(i)=self.enemies.iter().position(|v|equal_ms950(v,enemy)) {self.enemies.remove(i);}
            if self.enemies.is_empty() {self.active=false; self.own_clan.clear();}
        }
        let mut changes=Vec::new();
        for (id,actor) in actors {
            let mut a=actor.relations.clone();
            if a.clan.is_empty() {continue;}
            if declaring {
                if equal_ms950(&a.clan,enemy) {a.side_20=2; a.flag_1f.reset(true);}
                else if equal_ms950(&a.clan,own) {a.side_20=1;}
            } else if equal_ms950(&a.clan,enemy)
                || (self.enemies.is_empty() && equal_ms950(&a.clan,own)) {
                a.side_20=0; a.flag_1f.clear_for(actor.graphics)?;
            }
            if a!=actor.relations {changes.push((*id,a));}
        }
        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn war()->Context {
        Context{owner:Some(1),active:true,own_clan:b"Own".to_vec(),enemies:vec![b"Foe".to_vec()]}
    }
    fn relations(clan:&[u8],side:u8,flag:bool)->Relations {
        let mut a=Relations{clan:clan.to_vec(),side_20:side,..Default::default()};
        a.flag_1f.reset(flag); a
    }
    fn after(context:&Context,mut a:Relations)->(u8,bool) {
        context.clan_changed(&mut a); (a.side_20,a.flag_1f.set)
    }
    /// Native 4E7690 (S69 clan setter) under the war gate 834F14.
    #[test]
    fn s69_clan_update_sets_side_and_flag_only_while_war_is_active() {
        let idle=Context::default();
        assert_eq!(after(&idle,relations(b"Foe",0,false)),(0,false));
        assert_eq!(after(&idle,relations(b"",2,true)),(2,true));
        let war=war();
        assert_eq!(after(&war,relations(b"Foe",0,false)),(2,true));
        assert_eq!(after(&war,relations(b"fOE",1,false)),(2,true));
        assert_eq!(after(&war,relations(b"Own",0,false)),(1,true));
        // An unrelated clan keeps both fields; an empty clan clears only +0x20.
        assert_eq!(after(&war,relations(b"Other",2,true)),(2,true));
        assert_eq!(after(&war,relations(b"Other",0,false)),(0,false));
        assert_eq!(after(&war,relations(b"",2,true)),(0,true));
        // The own-clan compare runs after the enemy loop and wins.
        let both=Context{enemies:vec![b"Own".to_vec()],..war};
        assert_eq!(after(&both,relations(b"Own",0,false)),(1,true));
    }
}
