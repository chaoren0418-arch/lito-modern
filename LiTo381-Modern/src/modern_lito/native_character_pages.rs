//! Four visible rows over the server-owned directory; never synthesize names.
use super::super::native_login::{LoginPhase, Snapshot};
use anyhow::{ensure, Result};
use std::ops::Range;

pub const PAGE_SIZE: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Turn {
    Previous,
    Next,
}

/// Keyed by both transport and directory generation so a refreshed roster
/// cannot reuse an old page or submit an old visible index.
#[derive(Default)]
pub struct Pages {
    owner: Option<(u64, u64)>,
    count: usize,
    page: usize,
    wheel_delta: i32,
}

impl Pages {
    pub fn sync(&mut self, epoch: u64, snapshot: &Snapshot) {
        if !matches!(
            snapshot.phase,
            LoginPhase::CharactersReady | LoginPhase::CharacterPending
        ) {
            *self = Self::default();
            return;
        }
        let owner = Some((epoch, snapshot.directory_generation));
        if self.owner != owner || self.count != snapshot.characters.len() {
            *self = Self {
                owner,
                count: snapshot.characters.len(),
                ..Self::default()
            };
        }
        if !Self::enabled(snapshot) {
            self.wheel_delta = 0;
        }
    }

    pub fn enabled(snapshot: &Snapshot) -> bool {
        snapshot.phase == LoginPhase::CharactersReady && snapshot.notice.is_none()
    }

    pub fn range(&self) -> Range<usize> {
        let start = self.page * PAGE_SIZE;
        start..(start + PAGE_SIZE).min(self.count)
    }

    pub fn label(&self) -> String {
        if self.count == 0 {
            "角色（0 筆）".into()
        } else {
            format!(
                "角色 {}/{} 頁（{}–{}／{}）",
                self.page + 1,
                self.count.div_ceil(PAGE_SIZE),
                self.range().start + 1,
                self.range().end,
                self.count
            )
        }
    }

    pub fn can_turn(&self, turn: Turn) -> bool {
        match turn {
            Turn::Previous => self.page > 0,
            Turn::Next => self.range().end < self.count,
        }
    }

    pub fn turn(&mut self, turn: Turn) -> bool {
        self.wheel_delta = 0;
        if !self.can_turn(turn) {
            return false;
        }
        match turn {
            Turn::Previous => self.page -= 1,
            Turn::Next => self.page += 1,
        }
        true
    }

    /// Windows high-resolution wheels may deliver fractions of WHEEL_DELTA.
    pub fn wheel(&mut self, delta: i16) -> bool {
        self.wheel_delta += i32::from(delta);
        let steps = self.wheel_delta / 120;
        let remainder = self.wheel_delta % 120;
        let mut changed = false;
        for _ in 0..steps.unsigned_abs() {
            changed |= self.turn(if steps > 0 {
                Turn::Previous
            } else {
                Turn::Next
            });
        }
        self.wheel_delta = remainder;
        changed
    }

    pub fn index(&self, epoch: u64, snapshot: &Snapshot, visible: usize) -> Result<usize> {
        ensure!(Self::enabled(snapshot), "角色選取尚未就緒");
        ensure!(
            self.owner == Some((epoch, snapshot.directory_generation))
                && self.count == snapshot.characters.len(),
            "角色清單已更新，請重新選取"
        );
        ensure!(visible < self.range().len(), "請選擇本頁角色");
        Ok(self.range().start + visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        native_game::{self, Packet},
        native_login::LoginController,
    };

    fn snapshot(count: usize) -> Snapshot {
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-game-current.json"
        ))
        .unwrap();
        let hex = fixture["packets"]["row"].as_str().unwrap();
        let body: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let Packet::Row(row) = native_game::parse(&body).unwrap() else {
            panic!()
        };
        let mut directory = native_game::Directory::default();
        directory.start(1).unwrap();
        directory.push(row).unwrap();
        let character = directory.view().unwrap().remove(0);
        let mut s = LoginController::default().snapshot().clone();
        s.phase = LoginPhase::CharactersReady;
        s.directory_generation = 1;
        s.characters = (0..count)
            .map(|i| {
                let mut c = character.clone();
                c.name = format!("角色{i}");
                c
            })
            .collect();
        s
    }

    #[test]
    fn all_zero_to_eight_rosters_have_four_rows_per_page_and_no_phantom_slots() {
        for count in 0..=8 {
            let s = snapshot(count);
            let mut p = Pages::default();
            p.sync(2, &s);
            assert!(!p.turn(Turn::Previous));
            let mut indices = Vec::new();
            loop {
                assert!(p.range().len() <= PAGE_SIZE);
                for visible in 0..p.range().len() {
                    indices.push(p.index(2, &s, visible).unwrap());
                }
                assert!(p.index(2, &s, p.range().len()).is_err());
                assert!(p.index(2, &s, usize::MAX).is_err());
                if !p.turn(Turn::Next) {
                    break;
                }
            }
            assert_eq!(indices, (0..count).collect::<Vec<_>>());
            assert!(!p.turn(Turn::Next));
            if count > 4 {
                assert!(p.turn(Turn::Previous));
                assert_eq!(p.range(), 0..4);
            }
        }
    }

    #[test]
    fn wheel_accumulates_fractional_notches_and_clamps_both_ends() {
        let s = snapshot(8);
        let mut p = Pages::default();
        p.sync(2, &s);
        assert!(!p.wheel(-60));
        assert!(p.wheel(-60));
        assert_eq!(p.range(), 4..8);
        assert_eq!(p.label(), "角色 2/2 頁（5–8／8）");
        assert!(!p.wheel(-240));
        assert!(p.wheel(240));
        assert_eq!(p.range(), 0..4);
        assert!(!p.wheel(120));
        assert!(!p.wheel(0));
        assert!(!p.wheel(-60));
        assert!(p.turn(Turn::Next));
        assert!(!p.wheel(60)); // key/button clears the previous fractional delta
        assert!(p.wheel(60));
    }

    #[test]
    fn pending_and_notice_block_selection_but_refusal_preserves_second_page() {
        let mut s = snapshot(8);
        let mut p = Pages::default();
        p.sync(2, &s);
        p.turn(Turn::Next);
        s.phase = LoginPhase::CharacterPending;
        p.sync(2, &s);
        assert!(!Pages::enabled(&s));
        assert!(p.index(2, &s, 3).is_err());
        s.phase = LoginPhase::CharactersReady;
        s.character_result = Some(7);
        p.sync(2, &s);
        assert_eq!(p.index(2, &s, 3).unwrap(), 7);
        s.notice = Some(native_game::Notice {
            message_id: None,
            text: Some("公告".into()),
        });
        assert!(!Pages::enabled(&s));
        assert!(p.index(2, &s, 3).is_err());
    }

    #[test]
    fn refreshed_same_size_roster_new_connection_and_return_retire_old_page() {
        let mut s = snapshot(8);
        let mut p = Pages::default();
        p.sync(2, &s);
        p.turn(Turn::Next);
        s.directory_generation += 1;
        assert!(p.index(2, &s, 0).is_err());
        p.sync(2, &s);
        assert_eq!(p.range(), 0..4);
        p.turn(Turn::Next);
        assert!(p.index(3, &s, 0).is_err());
        p.sync(3, &s);
        assert_eq!(p.range(), 0..4);
        p.turn(Turn::Next);
        s.phase = LoginPhase::ReturningCharacters;
        p.sync(3, &s);
        assert!(p.index(3, &s, 0).is_err());
        s.phase = LoginPhase::CharactersReady;
        p.sync(3, &s);
        assert_eq!(p.index(3, &s, 0).unwrap(), 0);
        p.turn(Turn::Next);
        s.characters.truncate(1);
        assert!(p.index(3, &s, 0).is_err());
        p.sync(3, &s);
        assert_eq!(p.range(), 0..1);
        assert!(!p.can_turn(Turn::Next));
    }
}
