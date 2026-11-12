//! Controller/renderer boundary for the shared player and static queue.
use super::*;
use crate::modern_lito::{
    native_player_callbacks as player, native_static2941::Queue, native_world::World,
    native_world_view::Presentation,
};
use std::path::Path;

struct DrawHost<'a> {
    root: &'a Path,
    view: &'a mut Presentation,
    environment: &'a super::super::native_environment::State,
    transport: &'a mut Controller,
}
impl player::Driver for DrawHost<'_> {
    fn draw(
        &mut self,
        q: &mut Queue,
        w: &mut World,
        c: u64,
        clock: &mut dyn FnMut() -> Result<u32>,
    ) -> Result<usize> {
        let mut count = self.view.callback_draw(self.root, c, w, self.environment)?;
        let mut enqueued = false;
        for prepared in self.view.take_static2941_ready() {
            enqueued |= q.enqueue_prepared(w, c, self.root, prepared, &mut *clock)?;
        }
        if enqueued {
            count = self.view.callback_draw(self.root, c, w, self.environment)?;
        }
        ensure!(
            self.view
                .frame
                .as_ref()
                .is_some_and(|frame| frame.static_current(w, c)),
            "callback raster not publishable after resource enqueue"
        );
        Ok(count)
    }
    fn send(&mut self, c: u64, body: Zeroizing<Vec<u8>>) -> Result<()> {
        self.transport.send(c, body)
    }
    fn validate_step(&self, w: &World, h: u8) -> Result<()> {
        let frame = self
            .view
            .frame
            .as_ref()
            .context("player callback navigation unavailable")?;
        w.checked_move_request(h, &frame.navigation).map(|_| ())
    }
    fn frame_id(&self) -> Option<u64> {
        self.view.frame.as_ref()?.receipt["elapsed_ns"].as_u64()
    }
}
impl LoginController {
    pub(super) fn sync_player_control(&mut self) {
        self.player_callbacks.synchronize(
            &mut self.static2941,
            &mut self.state.world,
            self.transport.snapshot().epoch,
        );
        if let Some(character) = self
            .state
            .characters
            .iter()
            .find(|row| Some(&row.name) == self.state.selected_character.as_ref())
        {
            self.player_callbacks.character_timing(
                character.fields.reserved,
                self.state
                    .world
                    .status
                    .map_or(character.fields.level, |status| status.level),
            );
        }
    }
    pub(super) fn send_player_action(&mut self, body: Vec<u8>) -> Result<()> {
        let epoch = self.transport.snapshot().epoch;
        self.sync_player_control();
        if let Some(body) = self.player_callbacks.defer(Zeroizing::new(body))? {
            self.transport.send(epoch, body)?;
            self.player_callbacks.sent();
        }
        Ok(())
    }
    pub(crate) fn player_input(&mut self, heading: Option<u8>, cancel: bool) {
        self.sync_player_control();
        self.player_callbacks.held(heading);
        if cancel {
            self.player_callbacks.cancel();
        }
    }
    pub(crate) fn observe_player_frame(
        &mut self,
        frame: &super::super::native_world_scene::Frame,
    ) -> Result<()> {
        if frame.static_current(&self.state.world, self.transport.snapshot().epoch) {
            if let Some(id) = frame.receipt["elapsed_ns"].as_u64() {
                self.player_callbacks.observe_frame(
                    id,
                    frame
                        .player_walk_plan
                        .as_ref()
                        .map_or(0, |p| p.frame_count()),
                    super::super::native_clock::sample,
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn run_player_callbacks(
        &mut self,
        root: &Path,
        view: &mut Presentation,
        pump_only: bool,
    ) -> Result<()> {
        self.sync_player_control();
        let connection = self.transport.snapshot().epoch;
        let mut driver = DrawHost {
            root,
            view,
            environment: &self.state.environment,
            transport: &mut self.transport,
        };
        let mut clock = super::super::native_clock::sample;
        let mut host = player::Host {
            control: &mut self.player_callbacks,
            driver: &mut driver,
            clock: &mut clock,
        };
        host.run(
            &mut self.static2941,
            &mut self.state.world,
            connection,
            pump_only,
        )?;
        if let Some(sent) = self.player_callbacks.take_deferred_sent() {
            if let Some(pending) = &mut self.skill_pending {
                if let Some(ready) = &mut self.skill_ready {
                    *ready += sent.saturating_duration_since(pending.since);
                }
                pending.since = sent;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "native_login_callback_tests.rs"]
mod tests;
