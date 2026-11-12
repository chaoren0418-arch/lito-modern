//! G07.4：缺件與未知格式的明確失敗與來源診斷（T057）。
//!
//! 世界畫面原本把每個資源失敗寫在影格 receipt 的各個角落（`missing_attachments`、
//! `layer_error`、`presentation.reason`、音效的 `prepared:false`…），玩家看不到，報告也要
//! 逐欄去找。本模組把它們收成一種紀錄：**誰要用（consumer）、哪一類失敗（kind）、
//! 來源在哪裡（source／member）、原始原因（reason）**，並提供：
//!
//! - [`collect`]：從一張影格 receipt 取出所有資源失敗；
//! - [`Ledger`]：跨影格去重、有上限的清單，給 HUD 一行摘要、給 receipt 完整清單；
//! - [`classify`]：把解碼器與封存讀取的錯誤型別分成缺件／格式不支援／損毀／超過上限；
//! - [`explain_terrain`]：地圖載入失敗時說明這張地圖該讀哪個域、兩個域各有沒有資料。
//!
//! 診斷只描述呈現，不改任何權威狀態，也不放寬任何解碼器的檢查。
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

/// 客戶端唯一走經典域（共用 `map/` ＋ `Tile`）的地圖；其餘一律走競技場域
/// （`LiTo/LTmap` ＋ `LTtile`），缺檔時不改讀另一個域。對應 `native_world_scene.rs`
/// `load_terrain` 的 `world.map_id!=Some(4)`。
pub const CLASSIC_DOMAIN_MAPS: [u16; 1] = [4];
/// 伺服器更新過通行資料的 11 張經典地圖（lito-launcher-server
/// `native-de-terrain-current.zh-TW.md` 第 11 段）。客戶端只有經典域的資料，競技場域沒有
/// （`docs/MAP-DOMAINS.md` 第 3 節）。
pub const CLASSIC_REFRESHED_MAPS: [u16; 11] = [77, 78, 79, 80, 81, 82, 104, 105, 108, 109, 110];
/// 同一個編號在兩個域的物理不同、呼叫端必須明確選域的代表例（同一份伺服器文件）。
/// 客戶端資源裡兩個域都有的編號共 35 個，見 `docs/MAP-DOMAINS.md` 第 2 節。
pub const DUAL_DOMAIN_REFERENCE_MAP: u16 = 504;
pub const LEDGER_LIMIT: usize = 128;
const HUD_REASON_CHARS: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// 封存裡沒有這個成員，或檔案／資料夾不存在。
    Missing,
    /// 解碼器不認得的格式、型別或編碼。
    UnknownFormat,
    /// 格式認得，但內容截斷或不合法。
    Damaged,
    /// 超過本客戶端的大小或數量上限。
    OverBudget,
    /// 同名成員不只一筆，不能決定用哪一筆。
    Ambiguous,
    /// 資源存在，但不符合這個功能要求的版本資格，或這個功能還沒有接上。
    Unavailable,
    /// 其他失敗；原因見 reason。
    Other,
}
impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::Missing,
        Kind::UnknownFormat,
        Kind::Damaged,
        Kind::OverBudget,
        Kind::Ambiguous,
        Kind::Unavailable,
        Kind::Other,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Kind::Missing => "缺件",
            Kind::UnknownFormat => "格式不支援",
            Kind::Damaged => "內容損毀",
            Kind::OverBudget => "超過上限",
            Kind::Ambiguous => "同名多筆",
            Kind::Unavailable => "不可用",
            Kind::Other => "其他失敗",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Consumer {
    ActorBody,
    ActorLayers,
    ActorAttachment,
    ActorShadow,
    Sound,
    EffectPresentation,
    ZoneTable,
    MapPage,
    Terrain,
}
impl Consumer {
    pub fn label(self) -> &'static str {
        match self {
            Consumer::ActorBody => "角色本體",
            Consumer::ActorLayers => "角色附加層",
            Consumer::ActorAttachment => "角色附件",
            Consumer::ActorShadow => "角色陰影",
            Consumer::Sound => "音效",
            Consumer::EffectPresentation => "技能特效",
            Consumer::ZoneTable => "區域名稱表",
            Consumer::MapPage => "地圖頁",
            Consumer::Terrain => "地圖",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Diagnostic {
    pub consumer: Consumer,
    pub kind: Kind,
    /// 封存、檔案或查找路徑，例如 `Sprite07→Sprite`、`launcher/TW13081901.pak`。
    pub source: String,
    /// 成員或識別，例如 `6262-8.spr`、`gfx 727 action 4`。
    pub member: String,
    pub reason: String,
}
impl Diagnostic {
    /// 給玩家與紀錄看的一行。
    pub fn line(&self) -> String {
        format!(
            "{}{}：{}（{}）{}",
            self.consumer.label(),
            self.kind.label(),
            self.member,
            self.source,
            if self.reason.is_empty() { String::new() } else { format!("—{}", clip(&self.reason, HUD_REASON_CHARS)) }
        )
    }
}
fn clip(text: &str, limit: usize) -> String {
    let single = text.replace(['\r', '\n'], " ");
    if single.chars().count() <= limit {
        return single;
    }
    let mut out: String = single.chars().take(limit).collect();
    out.push('…');
    out
}

fn resource_kind(error: &super::resources::ResourceError) -> Kind {
    use super::resources::ResourceError as E;
    match error {
        E::MissingEntry(_) => Kind::Missing,
        E::Io(io) if io.kind() == std::io::ErrorKind::NotFound => Kind::Missing,
        E::AmbiguousEntry { .. } | E::DuplicateEntry(_) => Kind::Ambiguous,
        E::EntryTooLarge { .. } | E::SectorTooLarge { .. } | E::SoundTooLarge { .. } | E::TooManyEntries(_) => {
            Kind::OverBudget
        }
        E::MalformedIndex(_) | E::EntryOutOfBounds { .. } => Kind::Damaged,
        _ => Kind::Other,
    }
}
fn cause_kind(cause: &(dyn std::error::Error + 'static)) -> Option<Kind> {
    use super::{audio::WaveError, map::MapError, sprite::SpriteError, tile::TileError};
    if let Some(e) = cause.downcast_ref::<super::resources::ResourceError>() {
        return Some(resource_kind(e));
    }
    if let Some(e) = cause.downcast_ref::<SpriteError>() {
        return Some(match e {
            SpriteError::UnsupportedFormat(_) => Kind::UnknownFormat,
            SpriteError::Truncated { .. } | SpriteError::InvalidData(_) => Kind::Damaged,
            SpriteError::LimitExceeded(_) => Kind::OverBudget,
            _ => Kind::Other,
        });
    }
    if let Some(e) = cause.downcast_ref::<TileError>() {
        return Some(match e {
            TileError::UnsupportedFormat(_) | TileError::UnsupportedType(_) => Kind::UnknownFormat,
            TileError::Truncated { .. } | TileError::InvalidData(_) => Kind::Damaged,
            TileError::LimitExceeded(_) => Kind::OverBudget,
            _ => Kind::Other,
        });
    }
    if let Some(e) = cause.downcast_ref::<WaveError>() {
        return Some(match e {
            WaveError::UnsupportedContainer(_) | WaveError::UnsupportedEncoding(_) | WaveError::UnsupportedPcm { .. } => {
                Kind::UnknownFormat
            }
            WaveError::Truncated { .. }
            | WaveError::MissingChunk(_)
            | WaveError::DuplicateChunk(_)
            | WaveError::InvalidData(_) => Kind::Damaged,
            WaveError::LimitExceeded(_) => Kind::OverBudget,
            _ => Kind::Other,
        });
    }
    if let Some(e) = cause.downcast_ref::<MapError>() {
        return Some(match e {
            MapError::TooLarge { .. } | MapError::TooManyRecords { .. } => Kind::OverBudget,
            MapError::Truncated { .. } | MapError::NegativeCount { .. } => Kind::Damaged,
        });
    }
    if let Some(e) = cause.downcast_ref::<std::io::Error>() {
        return (e.kind() == std::io::ErrorKind::NotFound).then_some(Kind::Missing);
    }
    None
}
/// 依錯誤鏈裡第一個認得的型別分類；都不認得就是 [`Kind::Other`]（原因文字仍會保留）。
pub fn classify(error: &anyhow::Error) -> Kind {
    error.chain().find_map(cause_kind).unwrap_or(Kind::Other)
}

/// 角色 SPR 的查找路徑：先查分片，再查 `Sprite`（`native_actor_layers::Bank::ensure`）。
pub fn sprite_route(member: &str) -> String {
    format!("Sprite{:02}→Sprite", super::native_actor_layers::shard_of(member))
}

const TW_EFFECTS: [(&str, &str); 3] = [
    ("disintegrate_effects", "技能 77 究極光裂術"),
    ("call_lightning_effects", "技能 34 極道落雷"),
    ("cone_cold_effects", "技能 38 冰錐"),
];
/// 三個 TW 特效共用的資源（`shared_action_catalog::load_selected` 讀的 pack）。
pub const TW_EFFECT_SOURCE: &str = "launcher/TW13081901.pak";

fn kind_of(value: &Value) -> Kind {
    serde_json::from_value(value.clone()).unwrap_or(Kind::Other)
}
/// 從一張世界影格的 receipt 取出所有資源失敗。隱藏的角色（`hidden:true`）不是失敗。
pub fn collect(receipt: &Value) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for row in receipt["actors"].as_array().into_iter().flatten() {
        let graphics = row["logical_graphics"].as_u64().unwrap_or(0);
        if row["rendered"] == false {
            if let Some(error) = row["error"].as_str() {
                out.push(Diagnostic {
                    consumer: Consumer::ActorBody,
                    kind: kind_of(&row["error_kind"]),
                    source: "Sprite00–15→Sprite".into(),
                    member: format!("gfx {graphics} action {}", row["action"].as_u64().unwrap_or(0)),
                    reason: error.into(),
                });
            }
            continue;
        }
        for member in row["missing_attachments"].as_array().into_iter().flatten().filter_map(Value::as_str) {
            out.push(Diagnostic {
                consumer: Consumer::ActorAttachment,
                kind: Kind::Missing,
                source: sprite_route(member),
                member: member.into(),
                reason: format!("gfx {graphics} 的附件 bank 不在分片與 Sprite；依原版 4E6EA0 整組附件不畫"),
            });
        }
        if let Some(member) = row["missing_shadow"].as_str() {
            out.push(Diagnostic {
                consumer: Consumer::ActorShadow,
                kind: Kind::Missing,
                source: sprite_route(member),
                member: member.into(),
                reason: format!("gfx {graphics} 的陰影 bank 不在分片與 Sprite；這個姿勢不畫陰影"),
            });
        }
        if let Some(error) = row["layer_error"].as_str() {
            out.push(Diagnostic {
                consumer: Consumer::ActorLayers,
                kind: kind_of(&row["layer_error_kind"]),
                source: "Sprite00–15→Sprite".into(),
                member: format!("gfx {graphics}"),
                reason: error.into(),
            });
        }
    }
    for row in receipt["audio"].as_array().into_iter().flatten() {
        if row["prepared"] == false {
            out.push(Diagnostic {
                consumer: Consumer::Sound,
                kind: kind_of(&row["error_kind"]),
                source: "Sound".into(),
                member: match row["sound_id"].as_u64() {
                    Some(id) => format!("sound {id}"),
                    None => "sound ?".into(),
                },
                reason: row["error"].as_str().unwrap_or("").into(),
            });
        }
    }
    for (key, label) in TW_EFFECTS {
        let presentation = &receipt[key]["presentation"];
        if presentation["state"] == "unavailable" {
            out.push(Diagnostic {
                consumer: Consumer::EffectPresentation,
                kind: Kind::Unavailable,
                source: TW_EFFECT_SOURCE.into(),
                member: label.into(),
                reason: presentation["reason"].as_str().unwrap_or("").into(),
            });
        }
    }
    if let Some(error) = receipt["zone"]["table_error"].as_str() {
        out.push(Diagnostic {
            consumer: Consumer::ZoneTable,
            kind: kind_of(&receipt["zone"]["table_error_kind"]),
            source: super::native_zone::ARCHIVE.into(),
            member: super::native_zone::MEMBER.into(),
            reason: error.into(),
        });
    }
    out
}

/// 跨影格去重的診斷清單。同一個（consumer、kind、source、member）只記第一次的原因。
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    keys: BTreeSet<(Consumer, Kind, String, String)>,
    rows: Vec<Diagnostic>,
    dropped: u64,
}
impl Ledger {
    /// 新的一筆回傳 true；重複或清單已滿回傳 false（清單已滿會計入 `dropped`）。
    pub fn record(&mut self, diagnostic: Diagnostic) -> bool {
        let key = (diagnostic.consumer, diagnostic.kind, diagnostic.source.clone(), diagnostic.member.clone());
        if self.keys.contains(&key) {
            return false;
        }
        if self.rows.len() >= LEDGER_LIMIT {
            self.dropped = self.dropped.saturating_add(1);
            return false;
        }
        self.keys.insert(key);
        self.rows.push(diagnostic);
        true
    }
    /// 收進一張影格的失敗；每個新的失敗在 stderr 記一行（同一個失敗只記一次）。
    pub fn absorb(&mut self, found: Vec<Diagnostic>) -> usize {
        let mut added = 0;
        for diagnostic in found {
            let line = diagnostic.line();
            if self.record(diagnostic) {
                eprintln!("資源診斷：{line}");
                added += 1;
            }
        }
        added
    }
    pub fn rows(&self) -> &[Diagnostic] {
        &self.rows
    }
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
    pub fn count(&self, kind: Kind) -> usize {
        self.rows.iter().filter(|row| row.kind == kind).count()
    }
    /// HUD 的一行摘要；沒有任何失敗時是 None。
    pub fn hud(&self) -> Option<String> {
        let latest = self.rows.last()?;
        let kinds: Vec<String> = Kind::ALL
            .iter()
            .filter_map(|kind| match self.count(*kind) {
                0 => None,
                n => Some(format!("{} {n}", kind.label())),
            })
            .collect();
        let more = if self.dropped > 0 { format!("，另有 {} 筆超過清單上限", self.dropped) } else { String::new() };
        Some(format!("資源診斷 {} 項（{}{more}）；最新：{}", self.rows.len(), kinds.join("、"), latest.line()))
    }
    /// 影格 receipt 的 `resource_diagnostics`：完整清單與 HUD 摘要。
    pub fn receipt(&self) -> Value {
        serde_json::json!({"count":self.rows.len(),"dropped":self.dropped,"limit":LEDGER_LIMIT,
            "rows":self.rows,"hud":self.hud()})
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// 共用 `map/<id>` ＋ `Tile`。
    Classic,
    /// `LiTo/LTmap/<id>` ＋ `LTtile`。
    Arena,
}
impl Domain {
    pub fn of(map_id: u16) -> Self {
        if CLASSIC_DOMAIN_MAPS.contains(&map_id) {
            Domain::Classic
        } else {
            Domain::Arena
        }
    }
    pub fn directory(self, map_id: u16) -> String {
        match self {
            Domain::Classic => format!("map/{map_id}"),
            Domain::Arena => format!("LiTo/LTmap/{map_id}"),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Domain::Classic => "經典域",
            Domain::Arena => "競技場域",
        }
    }
}
/// 兩個域各有沒有這張地圖的資料夾。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TerrainPresence {
    pub arena: bool,
    pub classic: bool,
}
impl TerrainPresence {
    /// 只查資料夾是否存在，不讀內容。
    pub fn probe(root: &Path, map_id: u16) -> Self {
        let present =
            |domain: Domain| std::fs::metadata(root.join(domain.directory(map_id))).is_ok_and(|m| m.is_dir());
        Self { arena: present(Domain::Arena), classic: present(Domain::Classic) }
    }
}
/// 地圖載入失敗時的來源說明：這張地圖該讀哪個域、兩個域各有沒有資料、為什麼不改讀另一個域。
pub fn explain_terrain(map_id: u16, presence: TerrainPresence) -> String {
    let domain = Domain::of(map_id);
    let (own, other, other_domain) = match domain {
        Domain::Arena => (presence.arena, presence.classic, Domain::Classic),
        Domain::Classic => (presence.classic, presence.arena, Domain::Arena),
    };
    let mut text = format!("地圖 {map_id} 讀{}（{}）", domain.label(), domain.directory(map_id));
    match (own, other) {
        (true, false) => text.push_str("；資料夾存在，失敗原因是上面的訊息。"),
        (true, true) => text.push_str(&format!(
            "；資料夾存在，失敗原因是上面的訊息。{}（{}）也有同編號的資料，但內容不同，不會拿來代替。",
            other_domain.label(),
            other_domain.directory(map_id)
        )),
        (false, true) => text.push_str(&format!(
            "，但這個資料夾不存在。同編號只有{}（{}）的資料；兩個域的地形與通行不同，不會拿來代替。",
            other_domain.label(),
            other_domain.directory(map_id)
        )),
        (false, false) => text.push_str("，但這個資料夾不存在；另一個域也沒有這張地圖。"),
    }
    if CLASSIC_REFRESHED_MAPS.contains(&map_id) {
        text.push_str("這是伺服器更新過通行資料的 11 張經典地圖之一，客戶端沒有競技場域的版本。");
    }
    text
}
/// 把載入失敗包成一筆診斷（給 HUD 與紀錄）。
pub fn terrain_diagnostic(map_id: u16, presence: TerrainPresence, error: &str) -> Diagnostic {
    let domain = Domain::of(map_id);
    let own = match domain {
        Domain::Arena => presence.arena,
        Domain::Classic => presence.classic,
    };
    Diagnostic {
        consumer: Consumer::Terrain,
        kind: if own { Kind::Other } else { Kind::Missing },
        source: domain.directory(map_id),
        member: format!("map {map_id}"),
        reason: format!("{error}｜{}", explain_terrain(map_id, presence)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{audio::WaveError, map::MapError, resources::ResourceError, sprite::SpriteError, tile::TileError};

    fn any<E: std::error::Error + Send + Sync + 'static>(error: E) -> anyhow::Error {
        anyhow::Error::from(error).context("outer context")
    }

    #[test]
    fn classify_separates_missing_unknown_damaged_and_budget() {
        assert_eq!(classify(&any(ResourceError::MissingEntry("6262-8.spr".into()))), Kind::Missing);
        assert_eq!(classify(&any(ResourceError::AmbiguousEntry { name: "a".into(), count: 2 })), Kind::Ambiguous);
        assert_eq!(classify(&any(ResourceError::EntryTooLarge { name: "a".into(), size: 2, limit: 1 })), Kind::OverBudget);
        assert_eq!(classify(&any(ResourceError::MalformedIndex("x"))), Kind::Damaged);
        assert_eq!(classify(&any(ResourceError::InvalidLimits)), Kind::Other);
        assert_eq!(classify(&any(SpriteError::UnsupportedFormat("SP2"))), Kind::UnknownFormat);
        assert_eq!(classify(&any(SpriteError::InvalidData("reversed declared frame bounds"))), Kind::Damaged);
        assert_eq!(classify(&any(SpriteError::Truncated { offset: 1, needed: 2 })), Kind::Damaged);
        assert_eq!(classify(&any(SpriteError::LimitExceeded("blocks"))), Kind::OverBudget);
        assert_eq!(classify(&any(TileError::UnsupportedType(5))), Kind::UnknownFormat);
        assert_eq!(classify(&any(WaveError::UnsupportedEncoding(17))), Kind::UnknownFormat);
        assert_eq!(classify(&any(WaveError::MissingChunk(*b"data"))), Kind::Damaged);
        assert_eq!(classify(&any(MapError::TooLarge { size: 2, limit: 1 })), Kind::OverBudget);
        assert_eq!(classify(&any(std::io::Error::from(std::io::ErrorKind::NotFound))), Kind::Missing);
        assert_eq!(classify(&any(std::io::Error::from(std::io::ErrorKind::PermissionDenied))), Kind::Other);
        assert_eq!(classify(&anyhow::anyhow!("actor pose absent")), Kind::Other);
    }

    fn receipt() -> Value {
        serde_json::json!({
            "actors":[
                {"rendered":false,"hidden":true,"id":1,"logical_graphics":61},
                {"rendered":false,"id":2,"logical_graphics":727,"action":4,
                 "error":"entry 727-33.spr is absent","error_kind":"missing"},
                {"rendered":true,"id":3,"logical_graphics":0,"missing_attachments":["6262-8.spr","6262-9.spr"],
                 "missing_shadow":"3226-8.spr","layer_error":null},
                {"rendered":true,"id":4,"logical_graphics":9,"missing_attachments":[],"missing_shadow":null,
                 "layer_error":"unsupported SPR format: SP2","layer_error_kind":"unknown_format"},
                {"rendered":true,"id":5,"logical_graphics":0,"missing_attachments":["6262-8.spr"],"missing_shadow":null}
            ],
            "audio":[
                {"prepared":true,"bytes":10},
                {"prepared":false,"sound_id":145,"error":"unsupported WAVE encoding 0x0011","error_kind":"unknown_format"}
            ],
            "disintegrate_effects":{"presentation":{"state":"unavailable","reason":"TW pack SHA mismatch"}},
            "call_lightning_effects":{"presentation":{"state":"ready"}},
            "cone_cold_effects":{"presentation":{"state":"pending"}},
            "zone":{"name":null}
        })
    }

    #[test]
    fn collect_reads_every_failure_with_its_source_and_skips_hidden_actors() {
        let found = collect(&receipt());
        let brief: Vec<_> = found.iter().map(|d| (d.consumer, d.kind, d.source.as_str(), d.member.as_str())).collect();
        assert_eq!(
            brief,
            vec![
                (Consumer::ActorBody, Kind::Missing, "Sprite00–15→Sprite", "gfx 727 action 4"),
                (Consumer::ActorAttachment, Kind::Missing, sprite_route("6262-8.spr").as_str(), "6262-8.spr"),
                (Consumer::ActorAttachment, Kind::Missing, sprite_route("6262-9.spr").as_str(), "6262-9.spr"),
                (Consumer::ActorShadow, Kind::Missing, sprite_route("3226-8.spr").as_str(), "3226-8.spr"),
                (Consumer::ActorLayers, Kind::UnknownFormat, "Sprite00–15→Sprite", "gfx 9"),
                (Consumer::ActorAttachment, Kind::Missing, sprite_route("6262-8.spr").as_str(), "6262-8.spr"),
                (Consumer::Sound, Kind::UnknownFormat, "Sound", "sound 145"),
                (Consumer::EffectPresentation, Kind::Unavailable, TW_EFFECT_SOURCE, "技能 77 究極光裂術"),
            ]
        );
        assert_eq!(found[0].reason, "entry 727-33.spr is absent");
        assert_eq!(found[7].reason, "TW pack SHA mismatch");
        assert_eq!(sprite_route("0.spr"), "Sprite03→Sprite");
        assert!(collect(&serde_json::json!({})).is_empty());
        // 沒有 error_kind 的舊 receipt 仍會被收進來，分類是「其他失敗」。
        let legacy = collect(&serde_json::json!({"actors":[{"rendered":false,"logical_graphics":5,"error":"x"}],
            "zone":{"table_error":"LTzone header rows missing"}}));
        assert_eq!(legacy.iter().map(|d| (d.consumer, d.kind)).collect::<Vec<_>>(),
            vec![(Consumer::ActorBody, Kind::Other), (Consumer::ZoneTable, Kind::Other)]);
        assert_eq!(legacy[1].member, "ltzone-c.tbl");
    }

    #[test]
    fn ledger_deduplicates_across_frames_and_reports_a_player_line() {
        let mut ledger = Ledger::default();
        assert_eq!(ledger.hud(), None);
        assert_eq!(ledger.receipt()["count"], 0);
        assert_eq!(ledger.absorb(collect(&receipt())), 7, "the repeated attachment is one entry");
        assert_eq!(ledger.absorb(collect(&receipt())), 0, "the next frame adds nothing");
        assert_eq!((ledger.count(Kind::Missing), ledger.count(Kind::UnknownFormat), ledger.count(Kind::Unavailable)), (4, 2, 1));
        let hud = ledger.hud().unwrap();
        assert!(hud.starts_with("資源診斷 7 項（缺件 4、格式不支援 2、不可用 1）；最新：技能特效不可用：技能 77 究極光裂術（launcher/TW13081901.pak）—TW pack SHA mismatch"), "{hud}");
        let value = ledger.receipt();
        assert_eq!(value["rows"].as_array().unwrap().len(), 7);
        assert_eq!(value["rows"][1]["consumer"], "actor_attachment");
        assert_eq!(value["rows"][1]["kind"], "missing");
        assert_eq!(value["hud"], hud);
    }

    #[test]
    fn ledger_is_bounded_and_counts_what_it_could_not_keep() {
        let mut ledger = Ledger::default();
        for i in 0..LEDGER_LIMIT + 3 {
            ledger.record(Diagnostic { consumer: Consumer::Sound, kind: Kind::Missing, source: "Sound".into(),
                member: format!("sound {i}"), reason: String::new() });
        }
        assert_eq!((ledger.rows().len(), ledger.dropped()), (LEDGER_LIMIT, 3));
        assert!(ledger.hud().unwrap().contains("另有 3 筆超過清單上限"));
        let long = Diagnostic { consumer: Consumer::Sound, kind: Kind::Other, source: "Sound".into(), member: "m".into(),
            reason: "字".repeat(200) + "\r\n尾" };
        assert_eq!(long.line().chars().filter(|c| *c == '字').count(), HUD_REASON_CHARS);
        assert!(long.line().ends_with('…') && !long.line().contains('\n'));
    }

    #[test]
    fn terrain_explanation_names_the_domain_and_never_offers_the_other_one() {
        assert_eq!(Domain::of(4), Domain::Classic);
        assert_eq!(Domain::of(DUAL_DOMAIN_REFERENCE_MAP), Domain::Arena);
        assert!(CLASSIC_REFRESHED_MAPS.iter().all(|id| Domain::of(*id) == Domain::Arena));
        let both = explain_terrain(504, TerrainPresence { arena: true, classic: true });
        assert!(both.starts_with("地圖 504 讀競技場域（LiTo/LTmap/504）；資料夾存在"), "{both}");
        assert!(both.contains("經典域（map/504）也有同編號的資料，但內容不同，不會拿來代替"), "{both}");
        let classic_only = explain_terrain(77, TerrainPresence { arena: false, classic: true });
        assert!(classic_only.contains("LiTo/LTmap/77），但這個資料夾不存在"), "{classic_only}");
        assert!(classic_only.contains("同編號只有經典域（map/77）的資料"), "{classic_only}");
        assert!(classic_only.ends_with("客戶端沒有競技場域的版本。"), "{classic_only}");
        let absent = explain_terrain(999, TerrainPresence { arena: false, classic: false });
        assert!(absent.ends_with("另一個域也沒有這張地圖。"), "{absent}");
        let four = explain_terrain(4, TerrainPresence { arena: false, classic: true });
        assert_eq!(four, "地圖 4 讀經典域（map/4）；資料夾存在，失敗原因是上面的訊息。");
        let missing = terrain_diagnostic(77, TerrainPresence { arena: false, classic: true }, "no Native S32 sectors");
        assert_eq!((missing.consumer, missing.kind, missing.source.as_str()), (Consumer::Terrain, Kind::Missing, "LiTo/LTmap/77"));
        assert!(missing.reason.starts_with("no Native S32 sectors｜地圖 77"));
        assert_eq!(terrain_diagnostic(504, TerrainPresence { arena: true, classic: true }, "x").kind, Kind::Other);
        // 不存在的根目錄：兩個域都查不到，不會出錯。
        assert_eq!(TerrainPresence::probe(Path::new("Z:/lito-no-such-root"), 504), TerrainPresence { arena: false, classic: false });
    }
}
