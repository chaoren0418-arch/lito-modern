//! Mandatory schema5 owner tail. Recipient impact deliberately omits the
//! foreign caster's selection epoch. Outgoing scope is the containing owner.
use super::*;
use crate::modern_lito::{
    player_healing::{Effects, Impact, Receipt},
    player_healing_history::{Entry, Incoming, Outgoing},
};

pub(super) fn put(b: &mut Vec<u8>, h: Entry) {
    let i = h.incoming;
    put_world(b, i.world);
    put_participant(b, i.recipient);
    b.extend(i.received.to_be_bytes());
    b.extend(i.restored.to_be_bytes());
    presence(b, i.last.is_some());
    if let Some(last) = i.last {
        impact(b, last);
    }
    presence(b, h.outgoing.is_some());
    if let Some(o) = h.outgoing {
        put_ledger(b, o.spending.current);
        presence(b, o.last.is_some());
        if let Some(last) = o.last {
            impact(b, last.impact());
        }
    }
}
fn presence(b: &mut Vec<u8>, present: bool) {
    b.push(u8::from(present));
    zero(b, 3);
}
fn impact(b: &mut Vec<u8>, h: Impact) {
    put_world(b, h.world);
    put_participant(b, h.caster);
    put_cast(b, h.use_receipt);
    b.extend(h.target_effect_revision.to_be_bytes());
    let e = h.effects;
    b.push(
        u8::from(e.water_life)
            | u8::from(e.pollute_water) << 1
            | u8::from(e.dirty_current) << 2
            | u8::from(e.healing_erosion) << 3,
    );
    zero(b, 3);
    b.extend(h.base_amount.to_be_bytes());
    b.extend(h.adjusted_amount.to_be_bytes());
    put_effects(b, &[h.target]);
}
pub(super) fn get(r: &mut Reader<'_>, owner: Identity) -> Result<Entry, ApplicationError> {
    let incoming = Incoming {
        world: r.world()?,
        recipient: r.participant()?,
        received: r.i64()?,
        restored: r.i64()?,
        last: if r.healing_presence()? {
            Some(r.healing_impact()?)
        } else {
            None
        },
    };
    let outgoing = if r.healing_presence()? {
        let current = r.ledger()?;
        let last = if r.healing_presence()? {
            let i = r.healing_impact()?;
            // Never silently substitute a forged caster or world with owner.
            if i.world != owner.world || i.caster != owner.actor {
                return Err(invalid());
            }
            Some(Receipt {
                identity: owner,
                use_receipt: i.use_receipt,
                target_effect_revision: i.target_effect_revision,
                effects: i.effects,
                base_amount: i.base_amount,
                adjusted_amount: i.adjusted_amount,
                target: i.target,
            })
        } else {
            None
        };
        Some(Outgoing {
            spending: Spell {
                current,
                last: last.map(|h| h.use_receipt),
            },
            last,
        })
    } else {
        None
    };
    let value = Entry { incoming, outgoing };
    if !value.incoming.valid() || value.outgoing.is_some_and(|o| !o.valid()) {
        return Err(invalid());
    }
    Ok(value)
}
impl Reader<'_> {
    fn healing_presence(&mut self) -> Result<bool, ApplicationError> {
        let value = self.flag()?;
        self.zeros(3)?;
        Ok(value)
    }
    fn healing_impact(&mut self) -> Result<Impact, ApplicationError> {
        let world = self.world()?;
        let caster = self.participant()?;
        let use_receipt = self.cast()?;
        let target_effect_revision = self.i64()?;
        let flags = self.u8()?;
        self.zeros(3)?;
        if flags > 15 {
            return Err(invalid());
        }
        let effects = Effects {
            water_life: flags & 1 != 0,
            pollute_water: flags & 2 != 0,
            dirty_current: flags & 4 != 0,
            healing_erosion: flags & 8 != 0,
        };
        let base_amount = self.i64()?;
        let adjusted_amount = self.i64()?;
        let targets = self.effects(Lane::Healing)?;
        if targets.len() != 1 {
            return Err(invalid());
        }
        let value = Impact {
            world,
            caster,
            use_receipt,
            target_effect_revision,
            effects,
            base_amount,
            adjusted_amount,
            target: targets[0],
        };
        if !value.valid() {
            return Err(invalid());
        }
        Ok(value)
    }
}
