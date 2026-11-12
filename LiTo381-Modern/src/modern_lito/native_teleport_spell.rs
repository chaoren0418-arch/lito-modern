//! Skill 5 destinations: authoritative S30 bookmarks or explicit random travel.
use super::{
    native_inventory::Owner, native_login::Snapshot, native_navigation::InputStamp,
    native_portal::Destination,
};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Character {
    inventory: Owner,
    source: String,
}
fn character(s: &Snapshot, epoch: u64) -> Option<Character> {
    let inventory = s.inventory.owner.as_ref()?;
    let source = s.selected_source.clone()?;
    (s.game_connected
        && inventory.connection == epoch
        && s.selected_character.as_deref() == Some(inventory.character.as_str()))
    .then(|| Character {
        inventory: inventory.clone(),
        source,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Bookmark {
    name: Vec<u8>,
    label: String,
    destination: Destination,
}
#[derive(Default)]
pub struct Bookmarks {
    owner: Option<Character>,
    rows: Vec<Bookmark>,
    revision: u64,
}
impl Bookmarks {
    pub fn receive(&mut self, s: &Snapshot, epoch: u64, body: &[u8]) -> Result<bool> {
        if body.first() != Some(&0x30) {
            return Ok(false);
        }
        let end = body[1..]
            .iter()
            .position(|v| *v == 0)
            .map(|i| i + 1)
            .context("書籤名稱缺少結尾")?;
        ensure!(
            (1..=27).contains(&(end - 1)) && body.len() == end + 7,
            "書籤封包長度錯誤"
        );
        let name = &body[1..end];
        ensure!(
            name.iter().all(|v| *v >= 32 && *v != 127),
            "書籤名稱含控制字元"
        );
        let label = super::native_login::decode_text(name)?;
        ensure!(!label.trim().is_empty(), "空白書籤");
        let h = |at| u16::from_le_bytes([body[at], body[at + 1]]);
        let row = Bookmark {
            name: name.to_vec(),
            label,
            destination: Destination {
                x: h(end + 1),
                y: h(end + 3),
                map: h(end + 5),
            },
        };
        let Some(owner) = character(s, epoch) else {
            return Ok(true);
        };
        let same = self.owner.as_ref() == Some(&owner);
        let index = if same {
            self.rows.iter().position(|r| r.name == row.name)
        } else {
            None
        };
        if index.is_some_and(|i| self.rows[i] == row) {
            return Ok(true);
        }
        ensure!(
            !same || index.is_some() || self.rows.len() < 50,
            "書籤超過 50 筆"
        );
        let revision = self.revision.checked_add(1).context("書籤版本耗盡")?;
        if !same {
            self.rows.clear();
            self.owner = Some(owner);
        }
        if let Some(i) = index {
            self.rows[i] = row;
        } else {
            self.rows.push(row);
        }
        self.revision = revision;
        Ok(true)
    }
    fn current(&self, s: &Snapshot, epoch: u64) -> bool {
        self.owner.is_some() && self.owner == character(s, epoch)
    }
    pub fn labels(&self, s: &Snapshot, epoch: u64) -> Vec<String> {
        let mut labels = vec!["隨機傳送（目前地圖）".into()];
        if self.current(s, epoch) {
            labels.extend(self.rows.iter().map(|r| {
                format!(
                    "{}（地圖 {}：{}, {}）",
                    r.label, r.destination.map, r.destination.x, r.destination.y
                )
            }));
        }
        labels
    }
    pub fn choose(&self, s: &Snapshot, epoch: u64, index: usize) -> Result<Choice> {
        let owner = character(s, epoch).context("尚未取得角色背包")?;
        let (destination, bookmark) = if index == 0 {
            (
                Destination {
                    map: s.world.map_id.context("尚未收到地圖")?,
                    x: 0,
                    y: 0,
                },
                None,
            )
        } else {
            ensure!(self.current(s, epoch), "書籤角色已失效");
            let row = self.rows.get(index - 1).context("書籤已更新")?.clone();
            ensure!(
                row.destination.x != 0 && row.destination.y != 0,
                "書籤座標不可用"
            );
            (row.destination, Some((self.revision, row)))
        };
        let choice = Choice {
            owner,
            stamp: InputStamp::capture(epoch, &s.world),
            source_map: s.world.map_id,
            destination,
            bookmark,
        };
        choice.request(s, epoch, self)?;
        Ok(choice)
    }
}
#[derive(Clone, Debug)]
pub struct Choice {
    owner: Character,
    stamp: InputStamp,
    source_map: Option<u16>,
    destination: Destination,
    bookmark: Option<(u64, Bookmark)>,
}
impl Choice {
    pub fn destination(&self) -> Destination {
        self.destination
    }
    pub fn request(&self, s: &Snapshot, epoch: u64, book: &Bookmarks) -> Result<Vec<u8>> {
        ensure!(
            s.phase.active_world()
                && !s.observer
                && !s.world.observer_mode
                && character(s, epoch).as_ref() == Some(&self.owner)
                && self.source_map == s.world.map_id
                && self.stamp == InputStamp::capture(epoch, &s.world),
            "角色或場景已更新，請重新選擇傳送目的地"
        );
        if let Some((revision, row)) = &self.bookmark {
            ensure!(
                book.current(s, epoch) && *revision == book.revision && book.rows.contains(row),
                "書籤已更新，請重新選擇"
            );
        }
        super::native_skills::teleport_request(&s.world, self.destination)
    }
}
