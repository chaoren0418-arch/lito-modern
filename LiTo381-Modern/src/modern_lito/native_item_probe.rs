//! Loopback fixture exercise. Type/quantity choices refer only to its pinned test stock.
use super::{
    native_item_actions::{Action, Choice, Feedback},
    native_login::LoginController,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Probe {
    current: Option<Choice>,
    sent: Option<Instant>,
    settled: Option<Instant>,
    pub rows: Vec<serde_json::Value>,
}
impl Probe {
    pub fn tick(&mut self, c: &mut LoginController, refused: bool) -> Result<bool> {
        let plan: &[u8] = if refused { &[0, 0] } else { &[1, 23, 1, 23] };
        if self.rows.len() == plan.len() {
            return Ok(true);
        }
        if self.current.is_none() {
            let kind = plan[self.rows.len()];
            let found: Vec<_> = c
                .snapshot()
                .inventory
                .items
                .values()
                .filter(|i| i.use_type == kind && (kind != 0 || i.count == 2))
                .map(|i| i.id)
                .collect();
            ensure!(
                found.len() == 1,
                "fixture action must select one authoritative item"
            );
            let choice = Choice::capture(c.snapshot(), found[0])?;
            let before = c.snapshot().inventory.clone();
            c.item_action(choice.clone(), Action::Use)?;
            ensure!(
                c.snapshot().inventory == before,
                "request predicted inventory mutation"
            );
            ensure!(
                c.item_action(choice.clone(), Action::Use).is_err(),
                "duplicate pending request accepted"
            );
            self.current = Some(choice);
            self.sent = Some(Instant::now());
            return Ok(false);
        }
        ensure!(
            self.sent.unwrap().elapsed() < Duration::from_secs(6),
            "item fixture response timeout"
        );
        let feedback = c.snapshot().item_feedback;
        if feedback == Feedback::Pending {
            return Ok(false);
        }
        let expected = if refused {
            Feedback::NoResponse
        } else {
            Feedback::InventoryChanged
        };
        ensure!(
            feedback == expected,
            "unexpected item feedback {feedback:?}"
        );
        if self.settled.is_none() {
            self.settled = Some(Instant::now());
        }
        if self.settled.unwrap().elapsed() < Duration::from_millis(200) {
            return Ok(false);
        }
        let choice = self.current.take().unwrap();
        choice
            .owner
            .eq(c
                .snapshot()
                .inventory
                .owner
                .as_ref()
                .context("item owner absent")?)
            .then_some(())
            .context("item owner changed")?;
        let after = c
            .snapshot()
            .inventory
            .items
            .get(&choice.item.id)
            .context("equipment/potion disappeared")?;
        ensure!(
            after.count == choice.item.count,
            "fixture action changed count"
        );
        ensure!(
            (after == &choice.item) == refused,
            "expected item presentation update mismatch"
        );
        self.rows.push(serde_json::json!({"owner":choice.owner,"before":choice.item,"after":after,"feedback":feedback,"duplicate_pending_rejected":true}));
        self.sent = None;
        self.settled = None;
        Ok(self.rows.len() == plan.len())
    }
}
/// Actual world item exercise; inventory and HP remain server-owned.
#[derive(Default)]
pub struct WorldProbe {
    choice: Option<super::native_item_actions::Choice>,
    sent: Option<std::time::Instant>,
    ground_baseline: Vec<u32>,
    pub rows: Vec<serde_json::Value>,
}
impl WorldProbe {
    pub fn drop(
        &mut self,
        c: &mut LoginController,
        frame: &super::native_world_scene::Frame,
        refused: bool,
    ) -> Result<bool> {
        let s = c.snapshot();
        if let Some(sent) = self.sent {
            ensure!(
                sent.elapsed() < Duration::from_secs(7),
                "drop response timeout"
            );
            if sent.elapsed() < Duration::from_millis(if refused { 2400 } else { 800 }) {
                return Ok(false);
            }
            let previous = self.choice.as_ref().context("drop choice")?;
            let after = s.inventory.items.get(&previous.item.id);
            let ground: Vec<_> = s
                .world
                .actors
                .values()
                .filter(|a| !self.ground_baseline.contains(&a.id))
                .collect();
            if refused {
                ensure!(
                    s.item_feedback == Feedback::NoResponse
                        && after == Some(&previous.item)
                        && ground.is_empty(),
                    "refused drop changed authority"
                );
            } else {
                ensure!(
                    s.item_feedback == Feedback::InventoryChanged
                        && after.map_or(0, |i| i.count) == previous.item.count - 2,
                    "drop stock delta missing"
                );
                ensure!(ground.len() == self.rows.len(), "ground drop count differs");
                for a in &ground {
                    let own = s.world.own_actor().context("drop own")?;
                    ensure!(
                        a.experience == 2 && a.graphics == 773 && [a.x, a.y] == [own.x, own.y],
                        "ground item wire differs"
                    );
                    ensure!(
                        frame.receipt["actors"]
                            .as_array()
                            .context("rendered actors")?
                            .iter()
                            .any(|r| r["id"] == a.id
                                && r["rendered"] == true
                                && r["phase"] == "ground_item"),
                        "original ground item not rendered"
                    );
                }
                ensure!(
                    previous.validate(s).is_err(),
                    "stale dropped item remains usable"
                );
            }
            let row = self.rows.last_mut().unwrap();
            row["after"] = serde_json::to_value(after)?;
            row["ground"] = serde_json::to_value(ground)?;
            row["feedback"] = serde_json::to_value(s.item_feedback)?;
            if self.rows.len() == 2 {
                return Ok(true);
            }
        } else {
            self.ground_baseline = s.world.actors.keys().copied().collect();
        }
        let s = c.snapshot();
        let id = if let Some(choice) = &self.choice {
            choice.item.id
        } else {
            let rows: Vec<_> = s
                .inventory
                .items
                .values()
                .filter(|i| i.use_type == 0 && i.count == 4)
                .collect();
            ensure!(rows.len() == 1, "unique controlled gem fixture");
            rows[0].id
        };
        let choice = Choice::capture(s, id)?;
        let inventory = s.inventory.clone();
        let before_world = serde_json::to_value(&s.world)?;
        let own = s.world.own_actor().context("drop position")?;
        let position = [own.x, own.y];
        c.item_action(choice.clone(), Action::Drop(2))?;
        ensure!(
            c.snapshot().inventory == inventory
                && serde_json::to_value(&c.snapshot().world)? == before_world,
            "drop predicted authority"
        );
        ensure!(
            c.item_action(choice.clone(), Action::Drop(2)).is_err(),
            "duplicate pending drop admitted"
        );
        self.rows.push(serde_json::json!({"owner":choice.owner,"before":choice.item,"position":position,"quantity":2,"duplicate_pending_rejected":true}));
        self.choice = Some(choice);
        self.sent = Some(Instant::now());
        Ok(false)
    }
    pub fn heal(
        &mut self,
        c: &mut super::native_login::LoginController,
        frame: &super::native_world_scene::Frame,
        initial_hp: u16,
    ) -> anyhow::Result<bool> {
        use super::native_item_actions::{Action, Choice, Feedback};
        use anyhow::{ensure, Context};
        use std::time::{Duration, Instant};
        let s = c.snapshot();
        if self.sent.is_none() {
            if s.world.own_attack_packets != 1 || s.world.hp.context("item HP")?[0] >= initial_hp {
                return Ok(false);
            }
        } else {
            let elapsed = self.sent.unwrap().elapsed();
            ensure!(
                elapsed < Duration::from_secs(8),
                "world potion did not settle"
            );
            let fx = &frame.receipt["item_effects"];
            if elapsed < Duration::from_millis(1200)
                || fx["started"] != self.rows.len()
                || fx["active"] != 0
            {
                return Ok(false);
            }
            let previous = self.choice.as_ref().context("item choice")?;
            let expected = previous.item.count - 1;
            ensure!(
                s.item_feedback == Feedback::InventoryChanged,
                "potion update missing"
            );
            ensure!(
                s.inventory
                    .items
                    .get(&previous.item.id)
                    .map_or(0, |i| i.count)
                    == expected,
                "potion stock delta missing"
            );
            ensure!(
                s.world.own_attack_packets == 1
                    && s.world.own_cast_packets == 0
                    && s.world.hp == Some([50, 50])
                    && fx["omitted"] == 0,
                "potion authority/effect mismatch"
            );
            ensure!(
                previous.validate(s).is_err(),
                "stale potion selection remained valid"
            );
            let row = self.rows.last_mut().unwrap();
            row["after"] = serde_json::to_value(s.inventory.items.get(&previous.item.id))?;
            row["hp_after"] = serde_json::to_value(s.world.hp)?;
            row["feedback"] = serde_json::to_value(s.item_feedback)?;
            row["effect_finished"] = serde_json::json!(true);
            if self.rows.len() == 2 {
                return Ok(true);
            }
        }
        let s = c.snapshot();
        let candidates: Vec<_> = s
            .inventory
            .items
            .values()
            .filter(|i| i.use_type == 0 && i.count == 2 - self.rows.len() as u32)
            .collect();
        ensure!(candidates.len() == 1, "unique controlled potion fixture");
        let choice = Choice::capture(s, candidates[0].id)?;
        let before_inventory = s.inventory.clone();
        let hp = s.world.hp;
        c.item_action(choice.clone(), Action::Use)?;
        ensure!(
            c.snapshot().inventory == before_inventory && c.snapshot().world.hp == hp,
            "client predicted potion authority"
        );
        ensure!(
            c.item_action(choice.clone(), Action::Use).is_err(),
            "duplicate pending potion admitted"
        );
        self.rows.push(serde_json::json!({"owner":choice.owner,"before":choice.item,"hp_before":hp,"duplicate_pending_rejected":true}));
        self.choice = Some(choice);
        self.sent = Some(Instant::now());
        Ok(false)
    }
}
