//! LTzone 區域表：區域名稱與名稱的顯示色（G07.4，T057）。
//!
//! 來源是 `LiTo/LTtext` 的 `LTzone-c.tbl`（解碼後 125 bytes，SHA-256 7DA1B04C…，見
//! `docs/MAP-DOMAINS.md` 第 4 節）。格式：
//!
//! ```text
//! 5                       總列數（不含本列）
//! 2                       色彩列數
//! 45 95 209 ; 註解        色彩 0
//! 10 173 90               色彩 1
//! "LODUN" 0 500 0 0 0 0   "名稱" 色彩索引 地圖 x1 y1 x2 y2；四個 0 代表整張地圖
//! "LENOA" 1 560 0 0 0 0
//! ```
//!
//! **這張表只有名稱與顏色。** 沒有任何欄位表示安全區、禁戰區或可否攻擊，所以本模組不提供
//! 這類判斷，呼叫端也不能用顏色推導；能不能攻擊只由伺服器封包決定。
//!
//! 伺服器端的同一份投影是 lito-launcher-server `NativeLitoZoneCatalog`（唯讀參考）。
use anyhow::{bail, ensure, Context, Result};
use serde::Serialize;
use std::{path::Path, sync::atomic::AtomicBool};

pub const MEMBER: &str = "ltzone-c.tbl";
pub const ARCHIVE: &str = "LiTo/LTtext";
pub const MAX_TABLE_BYTES: usize = 64 * 1024;
pub const MAX_COLORS: usize = 64;
pub const MAX_ZONES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Zone {
    /// 原始位元組（各語系表的編碼不同，不在這裡猜）。
    pub name: Vec<u8>,
    pub color: u8,
    pub map_id: u32,
    /// `x1 y1 x2 y2`；四個 0 代表整張地圖。
    pub bounds: [u32; 4],
}
impl Zone {
    pub fn whole_map(&self) -> bool {
        self.bounds == [0; 4]
    }
    /// 可列印 ASCII 的名稱原樣顯示；其他位元組以 `\xNN` 表示，不做編碼猜測。
    pub fn display_name(&self) -> String {
        let mut out = String::new();
        for &b in &self.name {
            if (0x20..0x7f).contains(&b) && b != b'\\' {
                out.push(char::from(b));
            } else {
                out.push_str(&format!("\\x{b:02X}"));
            }
        }
        out
    }
    fn contains(&self, x: u16, y: u16) -> bool {
        let (x, y) = (u32::from(x), u32::from(y));
        self.whole_map()
            || (self.bounds[0].min(self.bounds[2])..=self.bounds[0].max(self.bounds[2])).contains(&x)
                && (self.bounds[1].min(self.bounds[3])..=self.bounds[1].max(self.bounds[3])).contains(&y)
    }
}

/// 畫面要用的結果：名稱與名稱的顯示色，沒有別的語意。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Label {
    pub name: String,
    pub rgb: [u8; 3],
    pub color_index: u8,
    pub whole_map: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Table {
    pub colors: Vec<[u8; 3]>,
    pub zones: Vec<Zone>,
    pub declared_rows: u32,
}

/// 去掉 `;` 之後的註解（引號內的 `;` 保留）與前後空白。
fn content(line: &[u8]) -> &[u8] {
    let mut quoted = false;
    let mut end = line.len();
    for (i, &b) in line.iter().enumerate() {
        match b {
            b'"' => quoted = !quoted,
            b';' if !quoted => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    line[..end].trim_ascii()
}
fn number<T: std::str::FromStr>(field: &[u8], what: &'static str) -> Result<T> {
    ensure!(!field.is_empty() && field.iter().all(u8::is_ascii_digit), "LTzone {what} is not a decimal number");
    std::str::from_utf8(field)?.parse().ok().with_context(|| format!("LTzone {what} out of range"))
}

impl Table {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_TABLE_BYTES, "LTzone table exceeds {MAX_TABLE_BYTES} bytes");
        ensure!(!bytes.contains(&0), "LTzone table contains NUL");
        let rows: Vec<&[u8]> = bytes.split(|b| *b == b'\n').map(content).filter(|row| !row.is_empty()).collect();
        ensure!(rows.len() >= 2, "LTzone header rows missing");
        let declared_rows: u32 = number(rows[0], "row count")?;
        let colors_declared: usize = number(rows[1], "color count")?;
        ensure!(colors_declared <= MAX_COLORS && rows.len() >= 2 + colors_declared, "LTzone color count invalid");
        // 原始註解：總列數不含第 1 列。不符就是表被截斷或格式不同，明確拒絕。
        ensure!(
            declared_rows as usize == rows.len() - 1,
            "LTzone declares {declared_rows} rows but has {}",
            rows.len() - 1
        );
        let mut colors = Vec::with_capacity(colors_declared);
        for row in &rows[2..2 + colors_declared] {
            let fields: Vec<&[u8]> = row.split(u8::is_ascii_whitespace).filter(|f| !f.is_empty()).collect();
            ensure!(fields.len() == 3, "LTzone color row needs r g b");
            colors.push([number(fields[0], "red")?, number(fields[1], "green")?, number(fields[2], "blue")?]);
        }
        let zone_rows = &rows[2 + colors_declared..];
        ensure!(zone_rows.len() <= MAX_ZONES, "LTzone zone count exceeds {MAX_ZONES}");
        let mut zones = Vec::with_capacity(zone_rows.len());
        for row in zone_rows {
            ensure!(row.first() == Some(&b'"'), "LTzone zone row must start with a quoted name");
            let close = row[1..].iter().position(|b| *b == b'"').context("LTzone zone name is not closed")? + 1;
            let name = row[1..close].to_vec();
            ensure!(!name.is_empty() && name.len() <= 64, "LTzone zone name length");
            let fields: Vec<&[u8]> =
                row[close + 1..].split(u8::is_ascii_whitespace).filter(|f| !f.is_empty()).collect();
            ensure!(fields.len() == 6, "LTzone zone row needs color, map and four bounds");
            let color: u8 = number(fields[0], "color index")?;
            ensure!(usize::from(color) < colors.len(), "LTzone color index {color} is not defined");
            let mut bounds = [0u32; 4];
            for (slot, field) in bounds.iter_mut().zip(&fields[2..]) {
                *slot = number(field, "bound")?;
            }
            zones.push(Zone { name, color, map_id: number(fields[1], "map")?, bounds });
        }
        // 同一張地圖有兩筆「整張地圖」就無法決定名稱；不默默挑一筆。
        for (i, zone) in zones.iter().enumerate() {
            if zone.whole_map() && zones[..i].iter().any(|z| z.whole_map() && z.map_id == zone.map_id) {
                bail!("LTzone map {} has two whole-map zones", zone.map_id);
            }
        }
        Ok(Self { colors, zones, declared_rows })
    }
    /// 讀本機 `LiTo/LTtext` 的 `LTzone-c.tbl`。成員不存在是明確錯誤（`ResourceError::MissingEntry`）。
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        let lt = root.join("LiTo");
        let index = super::native_world_scene::read_pin(&lt.join("LTtext.idx"), 1024 * 1024, cancel)?;
        let pak = super::native_world_scene::read_pin(&lt.join("LTtext.pak"), 1024 * 1024, cancel)?;
        let bytes = super::native_resources::read_native_text_optional(&lt, index, pak, MEMBER, cancel)?
            .ok_or_else(|| super::resources::ResourceError::MissingEntry(MEMBER.into()))?;
        Self::parse(&bytes)
    }
    /// 座標所在的區域。有範圍的列優先於整張地圖的列（推測：原版的比對順序沒有查；
    /// 目前的表只有整張地圖的列，兩種順序結果相同）。
    pub fn label(&self, map_id: u16, x: u16, y: u16) -> Option<Label> {
        let on_map = || self.zones.iter().filter(|z| z.map_id == u32::from(map_id) && z.contains(x, y));
        let zone = on_map().find(|z| !z.whole_map()).or_else(|| on_map().next())?;
        Some(Label {
            name: zone.display_name(),
            rgb: self.colors[usize::from(zone.color)],
            color_index: zone.color,
            whole_map: zone.whole_map(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `LiTo/LTtext` `LTzone-c.tbl` 解碼後的完整內容（125 bytes）。
    const LTZONE_C: &[u8] = b"5 \r\n2\r\n45 95 209 ; field zone name 0 blue \r\n10 173 90 ; 1-19 zone 1 green \r\n\"LODUN\" 0 500 0 0 0 0 \r\n\"LENOA\" 1 560 0 0 0 0\r\n\r\n";

    #[test]
    fn actual_table_has_two_colors_and_two_whole_map_zones() {
        assert_eq!(LTZONE_C.len(), 125);
        let table = Table::parse(LTZONE_C).unwrap();
        assert_eq!(table.declared_rows, 5);
        assert_eq!(table.colors, vec![[45, 95, 209], [10, 173, 90]]);
        assert_eq!(table.zones.len(), 2);
        assert!(table.zones.iter().all(Zone::whole_map));
        let lodun = table.label(500, 32768, 32768).unwrap();
        assert_eq!((lodun.name.as_str(), lodun.rgb, lodun.color_index, lodun.whole_map), ("LODUN", [45, 95, 209], 0, true));
        assert_eq!(table.label(560, 1, 1).unwrap().name, "LENOA");
        // 504 等其餘地圖沒有區域列：沒有名稱，不是錯誤，也不代表任何戰鬥規則。
        assert_eq!(table.label(504, 32768, 32768), None);
        assert_eq!(table.label(4, 32768, 32768), None);
    }

    #[test]
    fn bounded_rows_win_over_whole_map_and_names_keep_raw_bytes() {
        let table = Table::parse(
            b"5\n1\n1 2 3\n\"ALL;X\" 0 7 0 0 0 0 ; whole\n\"\xb7\xce\" 0 7 20 10 10 30\n\"B\\\" 0 8 5 5 5 5\n",
        )
        .unwrap();
        assert_eq!(table.label(7, 15, 20).unwrap().name, "\\xB7\\xCE");
        assert!(!table.label(7, 15, 20).unwrap().whole_map);
        assert_eq!(table.label(7, 9, 20).unwrap().name, "ALL;X");
        assert_eq!(table.label(8, 5, 5).unwrap().name, "B\\x5C");
        assert_eq!(table.label(8, 5, 6), None);
    }

    #[test]
    fn malformed_tables_fail_explicitly() {
        let cases: [(&[u8], &str); 13] = [
            (b"", "header rows missing"),
            (b"x\n1\n1 2 3\n", "row count is not a decimal"),
            (b"1\n2\n1 2 3\n", "color count invalid"),
            (b"3\n1\n1 2 3\n", "declares 3 rows but has 2"),
            (b"2\n1\n1 2 256\n", "blue out of range"),
            (b"2\n1\n1 2\n", "needs r g b"),
            (b"3\n1\n1 2 3\nA 0 7 0 0 0 0\n", "quoted name"),
            (b"3\n1\n1 2 3\n\"A 0 7 0 0 0 0\n", "not closed"),
            (b"3\n1\n1 2 3\n\"A\" 1 7 0 0 0 0\n", "color index 1 is not defined"),
            (b"3\n1\n1 2 3\n\"A\" 0 7 0 0 0\n", "four bounds"),
            (b"3\n1\n1 2 3\n\"A\" 0 -7 0 0 0 0\n", "map is not a decimal"),
            (b"4\n1\n1 2 3\n\"A\" 0 7 0 0 0 0\n\"B\" 0 7 0 0 0 0\n", "two whole-map zones"),
            (b"1\n1\n1 2 3\0\n", "contains NUL"),
        ];
        for (bad, needle) in cases {
            let error = format!("{:#}", Table::parse(bad).unwrap_err());
            assert!(error.contains(needle), "{error} / {needle}");
        }
        assert!(Table::parse(&vec![b' '; MAX_TABLE_BYTES + 1]).is_err());
    }
}
