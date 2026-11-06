//! T063 companion controls. Source settings, capacity and expiry remain server authority.
use super::{
    native_login::{LoginPhase, Snapshot},
    native_navigation::InputStamp,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Aggressive,
    Defensive,
    Stay,
    Extend,
    Alert,
    Attack,
    Collect,
    Rename,
    Dismiss,
}
impl Action {
    pub const ALL: [Self; 9] = [
        Self::Aggressive,
        Self::Defensive,
        Self::Stay,
        Self::Extend,
        Self::Alert,
        Self::Attack,
        Self::Collect,
        Self::Rename,
        Self::Dismiss,
    ];
    pub fn wire(self) -> &'static str {
        match self {
            Self::Aggressive => "aggressive",
            Self::Defensive => "defensive",
            Self::Stay => "stay",
            Self::Extend => "extend",
            Self::Alert => "alert",
            Self::Attack => "attackchr",
            Self::Collect => "getitem",
            Self::Rename => "changename",
            Self::Dismiss => "dismiss",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Aggressive => "攻擊態勢",
            Self::Defensive => "防禦態勢",
            Self::Stay => "休息",
            Self::Extend => "散開",
            Self::Alert => "警戒",
            Self::Attack => "指定攻擊目標",
            Self::Collect => "收取物品",
            Self::Rename => "申請改名",
            Self::Dismiss => "解放",
        }
    }
}

pub fn available(s: &Snapshot) -> bool {
    s.game_connected
        && matches!(s.phase, LoginPhase::BattlePending | LoginPhase::WorldReady)
        && s.world.ready()
        && !s.observer
        && !s.world.observer_mode
        && s.world.battle_result.is_none()
        && s.world.portal.is_none()
        && s.world.movement_blocks.is_empty()
        && s.world.hp.is_some_and(|v| v[0] > 0)
        && s.world.own_actor().is_some_and(|a| !a.dead)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub id: u32,
    stamp: InputStamp,
    incarnation: u64,
    revision: u64,
    master: Vec<u8>,
}
impl Choice {
    pub fn capture(s: &Snapshot, epoch: u64, id: u32) -> Result<Self> {
        ensure!(available(s), "目前不能操作寵物或召喚物");
        let own = s.world.own_actor().context("沒有目前角色")?;
        let a = s.world.actors.get(&id).context("同伴已離開場景")?;
        ensure!(
            id != own.id
                && !a.dead
                && a.draw_state.visible(false)
                && super::native_skills::in_native_screen([own.x, own.y], [a.x, a.y]),
            "同伴不在可操作範圍"
        );
        ensure!(
            a.relations.master_kind(Some(&own.relations.name))
                == super::native_light_sources::Master::Own,
            "不是目前角色的同伴"
        );
        Ok(Self {
            id,
            stamp: InputStamp::capture(epoch, &s.world),
            incarnation: a.incarnation,
            revision: a.target_revision,
            master: a.relations.master.clone(),
        })
    }
    pub fn validate(&self, s: &Snapshot, epoch: u64) -> Result<()> {
        ensure!(
            Self::capture(s, epoch, self.id).as_ref().ok() == Some(self),
            "同伴或場景已變更，請重新選取"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Menu {
    pub id: u32,
    pub posture: String,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub level: u8,
    pub name: String,
    pub carrying: String,
    pub experience_percent: u8,
    pub lawful: i16,
}
impl Menu {
    pub fn parse(d: &super::native_merchant::Dialog) -> Result<Self> {
        ensure!(
            matches!(d.html.as_str(), "anicom" | "moncom")
                && d.parameter.is_empty()
                && d.arguments.len() == 10,
            "不支援的同伴選單格式"
        );
        let a = &d.arguments;
        let v = Self {
            id: d.npc,
            posture: a[0].clone(),
            hp: a[1].parse()?,
            max_hp: a[2].parse()?,
            mp: a[3].parse()?,
            max_mp: a[4].parse()?,
            level: a[5].parse()?,
            name: a[6].clone(),
            carrying: a[7].clone(),
            experience_percent: a[8].parse()?,
            lawful: a[9].parse()?,
        };
        ensure!(
            v.id != 0
                && v.max_hp > 0
                && v.hp <= v.max_hp
                && v.mp <= v.max_mp
                && v.experience_percent <= 100,
            "同伴選單數值範圍錯誤"
        );
        Ok(v)
    }
    pub fn text(&self) -> String {
        let posture = match self.posture.as_str() {
            "$469" => "攻擊",
            "$470" => "防禦",
            "$471" => "休息／散開",
            "$472" => "警戒",
            other => other,
        };
        format!("{}　等級 {}　{}\r\nHP {}/{}　MP {}/{}　EXP {}%　正義值 {}\r\n攜帶狀態：{}（伺服器資料）",
            self.name,self.level,posture,self.hp,self.max_hp,self.mp,self.max_mp,self.experience_percent,
            self.lawful,if self.carrying=="$611"{"道具"}else{&self.carrying})
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wait {
    Menu,
    Target,
    Rename,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Session {
    pub menu: Option<Menu>,
    pub feedback: String,
    #[serde(skip)]
    choice: Option<Choice>,
    #[serde(skip)]
    waiting: Option<(Wait, Instant)>,
    #[serde(skip)]
    target: Option<(u8, Instant)>,
    #[serde(skip)]
    rename: Option<Instant>,
}
impl Session {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn tick(&mut self, s: &Snapshot, epoch: u64, now: Instant) {
        if self
            .choice
            .as_ref()
            .is_some_and(|c| c.validate(s, epoch).is_err())
        {
            self.clear();
            return;
        }
        if self.waiting.is_some_and(|(_, until)| now >= until) {
            self.waiting = None;
            self.feedback = "未收到同伴回覆；請重新開啟選單。".into();
        }
        if self.target.is_some_and(|(_, until)| now >= until) {
            self.target = None;
        }
        if self.rename.is_some_and(|until| now >= until) {
            self.rename = None;
        }
    }
    pub fn can_request(&self) -> bool {
        self.waiting.is_none()
    }
    pub fn can_target(&self, now: Instant) -> bool {
        self.target.is_some_and(|(_, until)| now < until)
    }
    pub fn can_rename(&self, now: Instant) -> bool {
        self.rename.is_some_and(|until| now < until)
    }
    pub fn opened(&mut self, c: Choice, now: Instant) {
        self.clear();
        self.choice = Some(c);
        self.waiting = Some((Wait::Menu, now + Duration::from_secs(8)));
        self.feedback = "正在等待伺服器同伴選單…".into();
    }
    pub fn action(&self, s: &Snapshot, epoch: u64, c: &Choice, a: Action) -> Result<Vec<u8>> {
        c.validate(s, epoch)?;
        ensure!(
            self.choice.as_ref() == Some(c) && self.menu.is_some() && self.can_request(),
            "請先開啟此同伴選單"
        );
        super::native_merchant::action(c.id, a.wire())
    }
    pub fn sent(&mut self, a: Action, now: Instant) {
        self.target = None;
        self.rename = None;
        self.waiting = match a {
            Action::Attack => Some((Wait::Target, now + Duration::from_secs(8))),
            Action::Rename => Some((Wait::Rename, now + Duration::from_secs(8))),
            _ => None,
        };
        // C_HACTION has no success ACK. Never predict posture, HP, capacity or deletion.
        self.feedback = "指令已送出；結果由伺服器裁定，選單數值為上次收到的資料。".into();
    }
    pub fn receive(&mut self, s: &Snapshot, epoch: u64, b: &[u8], now: Instant) -> Result<bool> {
        self.tick(s, epoch, now);
        if b.first() == Some(&0x2a) {
            let super::native_merchant::Packet::Dialog(d) = super::native_merchant::parse(b)?
            else {
                unreachable!()
            };
            if !matches!(d.html.as_str(), "anicom" | "moncom") {
                self.clear();
                return Ok(false);
            }
            let menu = Menu::parse(&d)?;
            if self.choice.as_ref().is_some_and(|c| c.id == d.npc)
                && self.waiting.is_some_and(|(w, _)| w == Wait::Menu)
            {
                self.menu = Some(menu);
                self.waiting = None;
                self.feedback = "已收到伺服器同伴資料。".into();
            }
            return Ok(true); // stale/unsolicited companion menus do not enter merchant state
        }
        if b.first() == Some(&0x57) {
            ensure!(b.len() == 6, "S_SLAVE_CONTROL 長度錯誤");
            let id = u32::from_le_bytes(b[1..5].try_into()?);
            ensure!(id != 0, "同伴 ID 為零");
            if self.choice.as_ref().is_some_and(|c| c.id == id)
                && self.waiting.is_some_and(|(w, _)| w == Wait::Target)
            {
                self.waiting = None;
                self.target = Some((b[5], now + Duration::from_secs(15)));
                self.feedback = "請選取攻擊對象並送出目標。".into();
            }
            return Ok(true);
        }
        if b.starts_with(&[0x24, 0x45, 0x01]) {
            ensure!(b.len() == 3, "寵物改名提示長度錯誤");
            if self.waiting.is_some_and(|(w, _)| w == Wait::Rename) {
                self.waiting = None;
                self.rename = Some(now + Duration::from_secs(30));
                self.feedback = "請輸入新名稱並送出；是否可改名由伺服器裁定。".into();
            }
            return Ok(true);
        }
        if b.first() == Some(&0x24) {
            // Native S_ASK has one callback; a newer question supersedes rename.
            self.rename = None;
            if self.waiting.is_some_and(|(w, _)| w == Wait::Rename) {
                self.waiting = None;
            }
        }
        Ok(false)
    }
    pub fn target_request(
        &self,
        s: &Snapshot,
        epoch: u64,
        target: u32,
        now: Instant,
    ) -> Result<Vec<u8>> {
        let c = self.choice.as_ref().context("沒有待決同伴")?;
        c.validate(s, epoch)?;
        let (mode, until) = self.target.context("尚未收到選擇目標提示")?;
        ensure!(now < until, "目標提示已過期");
        let a = s.world.actors.get(&target).context("目標已離開場景")?;
        let own = s.world.own_actor().context("沒有目前角色")?;
        ensure!(
            target != own.id
                && target != c.id
                && !a.dead
                && a.draw_state.visible(false)
                && super::native_skills::in_native_screen([own.x, own.y], [a.x, a.y]),
            "目標不可用"
        );
        let mut b = vec![0x4f];
        b.extend(c.id.to_le_bytes());
        b.push(mode);
        b.extend(target.to_le_bytes());
        Ok(b)
    }
    pub fn rename_request(
        &self,
        s: &Snapshot,
        epoch: u64,
        name: &str,
        now: Instant,
    ) -> Result<Vec<u8>> {
        self.choice
            .as_ref()
            .context("沒有待決同伴")?
            .validate(s, epoch)?;
        ensure!(self.can_rename(now), "尚未收到改名提示或提示已過期");
        let bytes = super::native_login::encode_text(name, 27)?;
        ensure!(bytes.len() >= 2, "寵物名稱至少兩個編碼位元組");
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            if c.is_ascii_alphanumeric() || c == b' ' {
                i += 1;
            } else {
                ensure!(
                    i + 1 < bytes.len()
                        && (0xb0a1..=0xc8fe).contains(&u16::from_be_bytes([c, bytes[i + 1]])),
                    "名稱不符合原生寵物命名範圍"
                );
                i += 2;
            }
        }
        let mut b = vec![0x23, 0x45, 0x01, 1];
        b.extend_from_slice(&bytes);
        b.push(0);
        Ok(b)
    }
    pub fn replied(&mut self) {
        self.target = None;
        self.rename = None;
        self.feedback = "已送出回覆，等待伺服器更新。".into();
    }
}

pub fn give(
    s: &Snapshot,
    epoch: u64,
    c: &Choice,
    item: &super::native_item_actions::Choice,
    quantity: u32,
) -> Result<Vec<u8>> {
    c.validate(s, epoch)?;
    item.validate(s)?;
    ensure!(item.owner.connection == epoch, "物品連線已變更");
    ensure!(
        quantity > 0 && quantity <= item.item.count && quantity <= i32::MAX as u32,
        "交付數量超過持有數量"
    );
    let a = &s.world.actors[&c.id];
    let own = s.world.own_actor().context("沒有目前角色")?;
    ensure!(
        a.x.abs_diff(own.x).max(a.y.abs_diff(own.y)) < 3,
        "請靠近同伴再交付物品"
    );
    let mut b = vec![0x11];
    b.extend(c.id.to_le_bytes());
    b.extend(a.x.to_le_bytes());
    b.extend(a.y.to_le_bytes());
    b.extend(item.item.id.to_le_bytes());
    b.extend(quantity.to_le_bytes());
    Ok(b)
}

pub fn summary(s: &Snapshot) -> String {
    let cha = s
        .world
        .status
        .as_ref()
        .map(|v| v.charisma.to_string())
        .unwrap_or_else(|| "未收到".into());
    format!("CHA：{cha}\r\n攜帶容量與職業加成：由伺服器裁定；尚未收到剩餘容量。\r\n召喚期限：尚未收到到期時間，由伺服器決定到期與解放。\r\n項圈：選取背包物品後使用；召回、攜帶限制與解放結果依伺服器回覆。")
}
