//! Isolated pickup fixture through the production controller; no local authority changes.
use super::{
    native_ground_pickup::{Choice, Feedback},
    native_item_actions as items,
    native_login::LoginController,
    native_world_scene::Frame,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Probe {
    since: Option<Instant>,
    drop: Option<items::Choice>,
    choice: Option<Choice>,
    pub drop_receipt: Option<serde_json::Value>,
    pub rows: Vec<serde_json::Value>,
}
impl Probe {
    pub fn tick(&mut self, c: &mut LoginController, f: &Frame, stale: bool) -> Result<bool> {
        let s = c.snapshot();
        if self.drop.is_none() {
            let candidates: Vec<_> = s
                .inventory
                .items
                .values()
                .filter(|i| i.count == 4)
                .collect();
            ensure!(candidates.len() == 1, "pickup gem fixture");
            let choice = items::Choice::capture(s, candidates[0].id)?;
            let before = serde_json::to_value(s)?;
            c.item_action(choice.clone(), items::Action::Drop(4))?;
            ensure!(
                serde_json::to_value(&c.snapshot().inventory)? == before["inventory"],
                "drop predicted inventory"
            );
            self.drop_receipt =
                Some(serde_json::json!({"before":before["inventory"],"choice":choice}));
            self.drop = Some(choice);
            self.since = Some(Instant::now());
            return Ok(false);
        }
        let elapsed = self.since.context("pickup since")?.elapsed();
        ensure!(elapsed < Duration::from_secs(12), "pickup stage deadline");
        if elapsed < Duration::from_millis(2500)
            || !f.movement_ready(&s.world)
            || !f.input_ready(&s.world)
        {
            return Ok(false);
        }
        if self.choice.is_none() {
            ensure!(
                s.item_feedback == items::Feedback::InventoryChanged
                    && self.drop.as_ref().unwrap().validate(s).is_err(),
                "fixture drop missing"
            );
            ensure!(
                f.ground_items.len() == 1 && f.ground_items[0].experience == 4,
                "fixture ground stack missing"
            );
            self.drop_receipt.as_mut().unwrap()["after"] = serde_json::to_value(&s.inventory)?;
        } else {
            let previous = self.choice.as_ref().unwrap();
            let row = self.rows.last_mut().unwrap();
            row["after"] = serde_json::json!({"inventory":s.inventory,"ground":s.world.actors.get(&previous.item.id),"hp":s.world.hp});
            row["feedback"] = serde_json::to_value(s.pickup_feedback)?;
            ensure!(
                s.world.hp == Some([50, 50])
                    && s.world.own_attack_packets == 0
                    && s.world.own_cast_packets == 0,
                "pickup changed battle authority"
            );
            if stale {
                ensure!(
                    s.pickup_feedback == Feedback::NoResponse && row["before"] == row["after"],
                    "stale pickup mutated authority"
                );
            } else {
                ensure!(
                    s.pickup_feedback == Feedback::ObservedUpdates && previous.validate(s).is_err(),
                    "pickup authority/stale selection missing"
                );
                ensure!(
                    s.world
                        .actors
                        .get(&previous.item.id)
                        .map_or(0, |a| a.experience)
                        == 4 - self.rows.len() as u32 * 2,
                    "pickup ground count"
                );
            }
            if self.rows.len() == 2 {
                return Ok(true);
            }
        }
        let s = c.snapshot();
        let id = f.ground_items.first().context("pickup target missing")?.id;
        let choice = Choice::capture(s, f, id)?;
        let before = serde_json::json!({"inventory":s.inventory,"ground":s.world.actors.get(&id),"hp":s.world.hp});
        let wire = super::native_ground_pickup::request(s, &choice, 2)?;
        c.pickup(choice.clone(), 2, f)?;
        ensure!(
            before
                == serde_json::json!({"inventory":c.snapshot().inventory,"ground":c.snapshot().world.actors.get(&id),"hp":c.snapshot().world.hp}),
            "pickup predicted authority"
        );
        ensure!(
            c.pickup(choice.clone(), 2, f).is_err(),
            "duplicate pending pickup admitted"
        );
        let potion = c
            .snapshot()
            .inventory
            .items
            .values()
            .next()
            .context("pickup potion fixture")?
            .id;
        let item = items::Choice::capture(c.snapshot(), potion)?;
        ensure!(
            c.item_action(item, items::Action::Use).is_err(),
            "item action admitted during pickup wait"
        );
        self.rows.push(serde_json::json!({"before":before,"choice":choice,"quantity":2,"wire":wire,"duplicate_pending_rejected":true,"item_pending_rejected":true}));
        self.choice = Some(choice);
        self.since = Some(Instant::now());
        Ok(false)
    }
}
