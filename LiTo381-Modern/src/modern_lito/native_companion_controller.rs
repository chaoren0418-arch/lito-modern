//! Companion requests share the active GAME transport and received world.
use super::*;
use crate::modern_lito::native_companions::{self as pets, Action, Choice};
#[cfg(test)]
#[path = "native_companion_tests.rs"]
mod native_t063;
impl LoginController {
    pub(super) fn tick_companions(&mut self) {
        let mut session = std::mem::take(&mut self.state.companions);
        session.tick(&self.state, self.transport.snapshot().epoch, Instant::now());
        self.state.companions = session;
    }
    pub(super) fn receive_companion(&mut self, body: &[u8]) -> Result<bool> {
        if !matches!(body.first(), Some(0x2a | 0x57 | 0x24)) {
            return Ok(false);
        }
        let mut session = std::mem::take(&mut self.state.companions);
        let result = session.receive(
            &self.state,
            self.transport.snapshot().epoch,
            body,
            Instant::now(),
        );
        self.state.companions = session;
        result
    }
    pub fn companion_talk(&mut self, c: Choice) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待場景入口回覆");
        self.tick_companions();
        c.validate(&self.state, self.transport.snapshot().epoch)?;
        ensure!(self.state.companions.can_request(), "請等待同伴回覆");
        self.transport.send(
            self.transport.snapshot().epoch,
            Zeroizing::new(crate::modern_lito::native_merchant::talk(c.id)?),
        )?;
        self.state.companions.opened(c, Instant::now());
        Ok(())
    }
    pub fn companion_action(&mut self, c: &Choice, a: Action) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待場景入口回覆");
        self.tick_companions();
        let body =
            self.state
                .companions
                .action(&self.state, self.transport.snapshot().epoch, c, a)?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.state.companions.sent(a, Instant::now());
        Ok(())
    }
    pub fn companion_target(&mut self, target: u32) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待場景入口回覆");
        let body = self.state.companions.target_request(
            &self.state,
            self.transport.snapshot().epoch,
            target,
            Instant::now(),
        )?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.state.companions.replied();
        Ok(())
    }
    pub fn companion_rename(&mut self, name: &str) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待場景入口回覆");
        let body = self.state.companions.rename_request(
            &self.state,
            self.transport.snapshot().epoch,
            name,
            Instant::now(),
        )?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.state.companions.replied();
        Ok(())
    }
    pub fn companion_give(
        &mut self,
        c: &Choice,
        item: &crate::modern_lito::native_item_actions::Choice,
        quantity: u32,
    ) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待場景入口回覆");
        let body = pets::give(
            &self.state,
            self.transport.snapshot().epoch,
            c,
            item,
            quantity,
        )?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.state.companions.feedback = "物品交付已送出；背包與同伴容量依伺服器更新。".into();
        Ok(())
    }
}
