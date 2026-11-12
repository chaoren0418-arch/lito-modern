//! 擂台（Tournament）點數商店：大廳／頻道／房間內的 C_DA／C_DE 開單、S_DF／S_E0 清單與 C_DB 交易。
//!
//! 依據（唯讀靜態證據，正式 JAR SHA256 `E1498871…D498`）：
//! - `Protocol`：`C_LITO_BUY_LIST=0xDA`、`C_LITO_BUY_SELL=0xDB`、`C_LITO_SELL_LIST=0xDE`、
//!   `S_LITO_POINT=0xDB`、`S_LITO_BUY_LIST=0xDF`、`S_LITO_SELL_LIST=0xE0`。
//! - `ClientSession.sendBuyList`：`DF h(n) n×[d(catalog) h(icon) d(buy) s(name) c(len) raw]`；
//!   `sendSellList`：`E0 h(n) n×[d(inventory objid) d(sell)]`；`openTournamentShop` 只在
//!   GAME 端點、已選角色、LOBBY／CHANNEL／ROOM 時開單，交易上下文 60 秒到期，並綁定
//!   phase、channel、room；`validTournamentShopConfirmation` 拒絕不符的確認。
//! - `Protocol.shopTransaction`：一般形式 `DB c(op) h(n) n×[d(id) d(qty)]`，op 0 買、1 賣；
//!   原生尾端的 8 個不透明 stack bytes 可省略（伺服器兩種長度都接受）。
//! - `TournamentShopCatalog.SAME_FAMILY_NATIVE`：16 列，買價＝賣價（`offer(id, price)`）；
//!   資料表 `lito_tournament_shop` 16 列與種子相同（`data-sources-0.6.60.json` L402–439）。
//!
//! 價格、點數、背包與重量都是伺服器權威；本模組不預扣點數、不預加物品。
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

pub const C_BUY_LIST: u8 = 0xda;
pub const C_BUY_SELL: u8 = 0xdb;
pub const C_SELL_LIST: u8 = 0xde;
pub const S_BUY_LIST: u8 = 0xdf;
pub const S_SELL_LIST: u8 = 0xe0;
/// 正式 `PendingTournamentShop` 期限（`nanoTime + 60_000_000_000`）。
pub const CONTEXT_SECONDS: u64 = 60;
/// 一般形式的 `Protocol.shopTransaction` 列數上限。
pub const MAX_LINES: usize = 510;

/// 正式 E149 `TournamentShopCatalog.SAME_FAMILY_NATIVE` 的 16 列（catalog id、價格；買價＝賣價）。
/// 只作為對照與顯示；實際可買清單與價格以伺服器送來的 S_DF／S_E0 為準。
pub const E149_OFFERS: [(u32, u32); 16] = [
    (40010, 40), (40011, 200), (40013, 200), (40007, 5000),
    (40744, 20), (40068, 1000), (40017, 70), (40012, 600),
    (40318, 900), (40746, 30), (40014, 800), (40016, 600),
    (40088, 1300), (40317, 150), (40015, 700), (40319, 700),
];
/// 與正式 16 列對照；回傳 (買價, 賣價)。
pub fn reference_price(catalog: u32) -> Option<(u32, u32)> {
    E149_OFFERS.iter().find(|(id, _)| *id == catalog).map(|(_, p)| (*p, *p))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuyRow { pub catalog: u32, pub icon: u16, pub price: u32, pub name: String, pub detail: Vec<u8> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SellRow { pub object: u32, pub price: u32 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Listing { Buy(Vec<BuyRow>), Sell(Vec<SellRow>) }
impl Listing {
    pub fn sell(&self) -> bool { matches!(self, Self::Sell(_)) }
    pub fn len(&self) -> usize { match self { Self::Buy(r) => r.len(), Self::Sell(r) => r.len() } }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

struct Reader<'a> { b: &'a [u8], at: usize }
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.at.checked_add(n).context("shop offset")?;
        let b = self.b.get(self.at..end).context("truncated tournament shop field")?;
        self.at = end;
        Ok(b)
    }
    fn c(&mut self) -> Result<u8> { Ok(self.take(1)?[0]) }
    fn h(&mut self) -> Result<u16> { Ok(u16::from_le_bytes(self.take(2)?.try_into()?)) }
    fn d(&mut self) -> Result<u32> { Ok(u32::from_le_bytes(self.take(4)?.try_into()?)) }
    fn text(&mut self) -> Result<String> {
        let len = self.b[self.at..].iter().take(257).position(|v| *v == 0)
            .context("shop item name exceeds 256-byte budget or lacks NUL")?;
        let s = if len == 0 { String::new() } else { super::native_login::decode_text(self.take(len)?)? };
        self.take(1)?;
        Ok(s)
    }
}

/// 解析 S_DF／S_E0。其他 opcode 回傳 `Ok(None)`。物件 ID 與價格一律為 unsigned dword。
pub fn parse(b: &[u8]) -> Result<Option<Listing>> {
    let Some(&op) = b.first() else { return Ok(None) };
    if op != S_BUY_LIST && op != S_SELL_LIST { return Ok(None); }
    ensure!(b.len() <= 65533, "tournament shop frame budget");
    let mut r = Reader { b, at: 1 };
    let n = usize::from(r.h()?);
    let mut ids = BTreeSet::new();
    let listing = if op == S_BUY_LIST {
        let mut rows = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            let catalog = r.d()?;
            ensure!(catalog != 0 && ids.insert(catalog), "invalid or duplicate tournament buy row");
            let icon = r.h()?;
            let price = r.d()?;
            ensure!(price != 0, "zero tournament buy price");
            let name = r.text()?;
            let size = usize::from(r.c()?);
            let detail = r.take(size)?.to_vec();
            rows.push(BuyRow { catalog, icon, price, name, detail });
        }
        Listing::Buy(rows)
    } else {
        let mut rows = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            let object = r.d()?;
            ensure!(object != 0 && ids.insert(object), "invalid or duplicate tournament sell object");
            let price = r.d()?;
            ensure!(price != 0, "zero tournament sell price");
            rows.push(SellRow { object, price });
        }
        Listing::Sell(rows)
    };
    ensure!(r.at == b.len(), "trailing tournament shop bytes");
    Ok(Some(listing))
}

/// C_DA（買單）或 C_DE（賣單）：正式 dispatch 不讀 payload，只送 opcode。
pub fn open_request(sell: bool) -> Vec<u8> { vec![if sell { C_SELL_LIST } else { C_BUY_LIST }] }

/// C_DB 一般形式 `DB c(op) h(n) n×[d(id) d(qty)]`。買單 id 是 S_DF 的 catalog id，
/// 賣單 id 是 S_E0 的背包物件 ID；同一張單不可重複列，數量 1..=i32::MAX。
pub fn transaction(sell: bool, lines: &[(u32, u32)]) -> Result<Vec<u8>> {
    ensure!(!lines.is_empty() && lines.len() <= MAX_LINES, "商店交易列數無效");
    let mut b = vec![C_BUY_SELL, u8::from(sell)];
    b.extend((lines.len() as u16).to_le_bytes());
    let mut ids = BTreeSet::new();
    for &(id, quantity) in lines {
        ensure!(id != 0 && ids.insert(id) && quantity > 0 && quantity <= i32::MAX as u32,
            "商店交易物品或數量無效");
        b.extend(id.to_le_bytes());
        b.extend(quantity.to_le_bytes());
    }
    Ok(b)
}

/// 伺服器 `PendingTournamentShop` 綁定的範圍；客戶端用同一組條件判斷清單是否仍可確認。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Scope {
    pub connection: u64,
    pub character: String,
    pub phase: super::native_login::LoginPhase,
    pub channel: Option<u8>,
    pub room: Option<u32>,
    pub inventory: Option<super::native_inventory::Owner>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Session {
    pub scope: Option<Scope>,
    pub requested_sell: Option<bool>,
    pub listing: Option<Listing>,
    pub revision: u64,
    pub waiting: bool,
    pub timed_out: bool,
    pub submitted: bool,
    /// 送出交易時的點數；伺服器之後的 S_DB 點數與 S_0F 訊息才是結果。
    pub points_at_submit: Option<u32>,
    /// 沒有對應開單而忽略的清單數（控制器在 `accept` 失敗時累加）。
    pub unsolicited: u64,
    #[serde(skip)] deadline: Option<Instant>,
    #[serde(skip)] expires: Option<Instant>,
}
impl Session {
    pub fn clear(&mut self) {
        let revision = self.revision.wrapping_add(1);
        *self = Self { revision, unsolicited: self.unsolicited, ..Self::default() };
    }
    /// 送出 C_DA／C_DE 之前呼叫；同時只允許一個未回覆的開單。
    pub fn begin(&mut self, scope: Scope, sell: bool, now: Instant) -> Result<Vec<u8>> {
        ensure!(!self.waiting, "等待商店清單");
        self.clear();
        self.scope = Some(scope);
        self.requested_sell = Some(sell);
        self.waiting = true;
        self.deadline = Some(now + Duration::from_secs(8));
        Ok(open_request(sell))
    }
    /// 只接受與目前開單同種類、同範圍的清單；未開單的清單被拒絕，不改狀態。
    pub fn accept(&mut self, listing: Listing, scope: Option<&Scope>, now: Instant) -> Result<()> {
        ensure!(self.scope.is_some() && self.scope.as_ref() == scope, "商店清單沒有對應的開單");
        ensure!(self.requested_sell == Some(listing.sell()), "商店清單種類與開單不符");
        self.waiting = false;
        self.timed_out = false;
        self.deadline = None;
        self.listing = Some(listing);
        self.expires = Some(now + Duration::from_secs(CONTEXT_SECONDS));
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    pub fn tick(&mut self, scope: Option<&Scope>, now: Instant) {
        if self.scope.is_some() && self.scope.as_ref() != scope { self.clear(); return; }
        if self.deadline.is_some_and(|at| now >= at) { self.waiting = false; self.timed_out = true; self.deadline = None; }
        if self.expires.is_some_and(|at| now >= at) {
            self.listing = None;
            self.expires = None;
            self.revision = self.revision.wrapping_add(1);
        }
    }
    /// 依清單列建 C_DB。賣單另檢查物件仍在目前背包且數量不超過持有量。
    pub fn order(&self, revision: u64, scope: Option<&Scope>, inventory: &super::native_inventory::Inventory,
        index: usize, quantity: u32, now: Instant) -> Result<Vec<u8>> {
        ensure!(!self.waiting && self.revision == revision && self.expires.is_some_and(|at| now < at)
            && self.scope.is_some() && self.scope.as_ref() == scope, "商店清單已變更或過期，請重新開啟");
        match self.listing.as_ref().context("沒有商店清單")? {
            Listing::Buy(rows) => {
                let row = rows.get(index).context("買單列無效")?;
                ensure!(u64::from(row.price).checked_mul(u64::from(quantity)).is_some_and(|t| t <= u64::from(u32::MAX)),
                    "購買總價超過上限");
                transaction(false, &[(row.catalog, quantity)])
            }
            Listing::Sell(rows) => {
                let row = rows.get(index).context("賣單列無效")?;
                let item = inventory.items.get(&row.object).context("物品已不在背包")?;
                ensure!(quantity <= item.count, "出售數量超過持有量");
                transaction(true, &[(row.object, quantity)])
            }
        }
    }
    pub fn submitted(&mut self, points: Option<u32>) {
        self.listing = None;
        self.expires = None;
        self.submitted = true;
        self.points_at_submit = points;
        self.revision = self.revision.wrapping_add(1);
    }
    /// 顯示用：S_DF 列與正式 16 列相同時標示，不同時照伺服器顯示，不改價格。
    pub fn buy_label(row: &BuyRow) -> String {
        let name = if row.name.is_empty() { format!("物品 {}", row.catalog) } else {
            row.name.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
        };
        format!("{name}　{} 點", row.price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_inventory::{Inventory, Item};
    use crate::modern_lito::native_login::LoginPhase;

    fn scope() -> Scope {
        Scope { connection: 3, character: "A".into(), phase: LoginPhase::LobbyReady, channel: Some(1), room: None,
            inventory: Some(crate::modern_lito::native_inventory::Owner { connection: 3, selection: 1, character: "A".into() }) }
    }
    fn buy_frame(rows: &[(u32, u16, u32, &[u8], &[u8])]) -> Vec<u8> {
        let mut b = vec![S_BUY_LIST];
        b.extend((rows.len() as u16).to_le_bytes());
        for (id, icon, price, name, detail) in rows {
            b.extend(id.to_le_bytes());
            b.extend(icon.to_le_bytes());
            b.extend(price.to_le_bytes());
            b.extend_from_slice(name);
            b.push(0);
            b.push(detail.len() as u8);
            b.extend_from_slice(detail);
        }
        b
    }

    #[test]
    fn e149_reference_table_has_sixteen_unique_rows_with_equal_buy_and_sell() {
        let ids: BTreeSet<_> = E149_OFFERS.iter().map(|r| r.0).collect();
        assert_eq!(ids.len(), 16);
        assert_eq!(reference_price(40318), Some((900, 900)));
        assert_eq!(reference_price(40007), Some((5000, 5000)));
        assert_eq!(reference_price(40743), None);
        assert_eq!(E149_OFFERS.iter().map(|r| u64::from(r.1)).sum::<u64>(), 12310);
    }

    #[test]
    fn buy_and_sell_lists_parse_exactly_and_reject_malformed_frames() {
        let frame = buy_frame(&[(40010, 1, 40, b"$233", &[0x17, 1]), (0xfedc_ba98, 0xffff, u32::MAX, b"", &[])]);
        let Listing::Buy(rows) = parse(&frame).unwrap().unwrap() else { panic!() };
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].catalog, rows[0].price, rows[0].detail.as_slice()), (40010, 40, &[0x17u8, 1][..]));
        assert_eq!((rows[1].catalog, rows[1].icon, rows[1].price), (0xfedc_ba98, 0xffff, u32::MAX));
        for cut in 1..frame.len() { assert!(parse(&frame[..cut]).is_err(), "cut {cut}"); }
        let mut extra = frame.clone(); extra.push(0);
        assert!(parse(&extra).is_err());
        assert!(parse(&buy_frame(&[(40010, 1, 0, b"x", &[])])).is_err());
        assert!(parse(&buy_frame(&[(40010, 1, 1, b"x", &[]), (40010, 1, 1, b"y", &[])])).is_err());
        let sell = [S_SELL_LIST, 2, 0, 0x98, 0xba, 0xdc, 0xfe, 30, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 1, 0, 0, 0];
        let Listing::Sell(rows) = parse(&sell).unwrap().unwrap() else { panic!() };
        assert_eq!(rows, vec![SellRow { object: 0xfedc_ba98, price: 30 }, SellRow { object: u32::MAX, price: 1 }]);
        for cut in 1..sell.len() { assert!(parse(&sell[..cut]).is_err()); }
        assert!(parse(&[S_SELL_LIST, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0]).is_err());
        assert!(parse(&[S_SELL_LIST, 0, 0]).unwrap().unwrap().is_empty());
        assert!(parse(&[0xdb, 0]).unwrap().is_none());
    }

    #[test]
    fn transaction_matches_formal_ordinary_c_db_layout() {
        assert_eq!(open_request(false), [0xda]);
        assert_eq!(open_request(true), [0xde]);
        assert_eq!(transaction(false, &[(40318, 4)]).unwrap(),
            [0xdb, 0, 1, 0, 0x7e, 0x9d, 0, 0, 4, 0, 0, 0]);
        let sell = transaction(true, &[(0xfedc_ba98, 1), (u32::MAX, 2)]).unwrap();
        assert_eq!(sell.len() & 7, 4, "formal parser selects the context-free form by length&7==4");
        assert_eq!(&sell[..4], &[0xdb, 1, 2, 0]);
        assert!(transaction(false, &[]).is_err());
        assert!(transaction(false, &[(1, 0)]).is_err());
        assert!(transaction(false, &[(0, 1)]).is_err());
        assert!(transaction(false, &[(1, 1), (1, 2)]).is_err());
        assert!(transaction(false, &[(1, i32::MAX as u32 + 1)]).is_err());
    }

    #[test]
    fn session_binds_listing_to_request_scope_kind_and_sixty_second_context() {
        let now = Instant::now();
        let mut s = Session::default();
        assert!(s.accept(Listing::Buy(vec![]), Some(&scope()), now).is_err(), "unsolicited list");
        assert_eq!(s.begin(scope(), false, now).unwrap(), [0xda]);
        assert!(s.begin(scope(), false, now).is_err(), "one request at a time");
        assert!(s.accept(Listing::Sell(vec![]), Some(&scope()), now).is_err(), "wrong kind");
        let mut other = scope(); other.room = Some(9);
        assert!(s.accept(Listing::Buy(vec![]), Some(&other), now).is_err(), "wrong scope");
        let row = BuyRow { catalog: 40318, icon: 1, price: 900, name: "$1".into(), detail: vec![] };
        s.accept(Listing::Buy(vec![row]), Some(&scope()), now).unwrap();
        let inv = Inventory::default();
        let rev = s.revision;
        assert_eq!(s.order(rev, Some(&scope()), &inv, 0, 2, now).unwrap(), transaction(false, &[(40318, 2)]).unwrap());
        assert!(s.order(rev, Some(&scope()), &inv, 1, 1, now).is_err());
        assert!(s.order(rev + 1, Some(&scope()), &inv, 0, 1, now).is_err());
        assert!(s.order(rev, Some(&other), &inv, 0, 1, now).is_err());
        assert!(s.order(rev, Some(&scope()), &inv, 0, u32::MAX / 900 + 1, now).is_err(), "u32 total overflow");
        let late = now + Duration::from_secs(CONTEXT_SECONDS);
        assert!(s.order(rev, Some(&scope()), &inv, 0, 1, late).is_err());
        s.tick(Some(&scope()), late);
        assert!(s.listing.is_none());
        s.tick(Some(&other), late);
        assert!(s.scope.is_none(), "scope change clears the session");
    }

    #[test]
    fn sell_order_requires_current_inventory_object_and_count() {
        let now = Instant::now();
        let mut s = Session::default();
        s.begin(scope(), true, now).unwrap();
        s.accept(Listing::Sell(vec![SellRow { object: 0xfedc_ba98, price: 30 }]), Some(&scope()), now).unwrap();
        let mut inv = Inventory::default();
        assert!(s.order(s.revision, Some(&scope()), &inv, 0, 1, now).is_err());
        inv.items.insert(0xfedc_ba98, Item { id: 0xfedc_ba98, use_type: 0xfe, charge: 0, icon: 96, blessing: 1,
            count: 5, identified: 1, name: "$773 (5)".into(), detail: vec![] });
        assert_eq!(s.order(s.revision, Some(&scope()), &inv, 0, 5, now).unwrap(),
            [0xdb, 1, 1, 0, 0x98, 0xba, 0xdc, 0xfe, 5, 0, 0, 0]);
        assert!(s.order(s.revision, Some(&scope()), &inv, 0, 6, now).is_err());
        s.submitted(Some(10));
        assert!(s.listing.is_none() && s.submitted && s.points_at_submit == Some(10));
    }

    #[test]
    fn open_request_times_out_without_inventing_a_list() {
        let now = Instant::now();
        let mut s = Session::default();
        s.begin(scope(), false, now).unwrap();
        s.tick(Some(&scope()), now + Duration::from_secs(8));
        assert!(!s.waiting && s.timed_out && s.listing.is_none());
    }
}
