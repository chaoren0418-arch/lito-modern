//! HPBAR：頭頂 HP 比例條（T046-3，Issue #183）。
//!
//! 權威來源全部是伺服器（唯讀參考 lito-launcher-server）：
//! - `S_PUT_OBJECT` 的 hp 欄位：任務怪 `Protocol.questMonster`、限時怪
//!   `Protocol.timeAttackActor`、守護塔帶 `healthRatio`（0..100）；自己、非主人
//!   寵物（`Protocol.classicPet`）、普通世界怪（`Protocol.classicWorldMonster`）帶
//!   `0xFF`（隱藏）。
//! - `S_HIT_RATIO`（0x68＝104）`d(objectId) c(ratio)`：E149 `Protocol.hitRatio`
//!   只送 0..100。候選 `NativeSourceMonsterHp`（integration-lifecycle）只對任務／
//!   限時怪（kind 1／2）的施法者送比例，怪物死亡時送 `c(255)` 隱藏（hide255）。
//!
//! 因此「只給任務／限時怪物」是伺服器決定誰拿到 0..100；客戶端不依怪物種類、
//! 死亡或距離自行推算，只看收到的值：0..=100 顯示，0xFF 隱藏。
//! 條的位置與大小是現代版版面（`NATIVE_GEOMETRY_COMPLETE = false`），原生
//! LiTo.bin 的 HPBAR 繪製函式尚未逆向。
use super::{compositor::Canvas, native_world::World};
use anyhow::{ensure, Result};

/// 伺服器的「隱藏」值（S_PUT_OBJECT 尾段與候選 hide255）。
pub const HIDDEN: u8 = 0xFF;
pub const MAX_RATIO: u8 = 100;
/// 現代版條寬高（含 1px 外框）。
pub const BAR_WIDTH: i32 = 40;
pub const BAR_HEIGHT: i32 = 5;
pub const NATIVE_GEOMETRY_COMPLETE: bool = false;
const FRAME: [u8; 4] = [16, 16, 16, 255];
const EMPTY: [u8; 4] = [72, 16, 16, 255];
const FILL: [u8; 4] = [224, 32, 32, 255];

/// `S_HIT_RATIO` 的值域：0..=100 是比例，0xFF 是隱藏；101..=254 沒有任何伺服器
/// 版本會送，維持原本的嚴格拒絕（整包不套用）。
pub fn receive(ratio: u8) -> Result<u8> {
    ensure!(ratio <= MAX_RATIO || ratio == HIDDEN, "invalid HP ratio");
    Ok(ratio)
}

/// 要顯示的比例；0xFF（以及 S_PUT_OBJECT 帶來的其他越界值）都不顯示。
pub fn visible_ratio(ratio: u8) -> Option<u8> {
    (ratio <= MAX_RATIO).then_some(ratio)
}

/// 內部填色寬度（不含外框），無條件捨去；只要比例 > 0 就至少 1px。
pub fn fill_width(ratio: u8) -> i32 {
    let inner = BAR_WIDTH - 2;
    let w = inner * i32::from(ratio.min(MAX_RATIO)) / i32::from(MAX_RATIO);
    if ratio > 0 { w.max(1) } else { 0 }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Bar {
    pub id: u32,
    pub ratio: u8,
    /// 左上角（畫布座標）。
    pub x: i32,
    pub y: i32,
    pub fill: i32,
}

/// `hits` 是本幀實際畫出的 actor 與身體範圍 `[left, top, right, bottom]`。
/// 條放在身體上緣上方；上緣超出畫布時改放下緣下方（與名稱標籤同規則）。
pub fn layout(world: &World, hits: &[(u32, [i32; 4])]) -> Vec<Bar> {
    let mut bars = Vec::new();
    for (id, b) in hits {
        if !world.actor_visible(*id) {
            continue;
        }
        let Some(actor) = world.actors.get(id) else { continue };
        let Some(ratio) = visible_ratio(actor.hp_ratio) else { continue };
        let top = b[1] - BAR_HEIGHT - 1;
        let y = if top < 0 { b[3] + 1 } else { top };
        let x = ((b[0] + b[2] - BAR_WIDTH) / 2).clamp(0, 800 - BAR_WIDTH);
        bars.push(Bar { id: *id, ratio, x, y, fill: fill_width(ratio) });
    }
    bars
}

fn rgba(bar: &Bar) -> Vec<u8> {
    let mut out = Vec::with_capacity((BAR_WIDTH * BAR_HEIGHT * 4) as usize);
    for y in 0..BAR_HEIGHT {
        for x in 0..BAR_WIDTH {
            let edge = x == 0 || y == 0 || x == BAR_WIDTH - 1 || y == BAR_HEIGHT - 1;
            out.extend(if edge {
                FRAME
            } else if x - 1 < bar.fill {
                FILL
            } else {
                EMPTY
            });
        }
    }
    out
}

pub fn draw(canvas: &mut Canvas, bars: &[Bar]) -> Result<serde_json::Value> {
    for bar in bars {
        canvas.blit_rgba(&rgba(bar), BAR_WIDTH as u32, BAR_HEIGHT as u32, bar.x, bar.y)?;
    }
    Ok(serde_json::json!({
        "bars": bars,
        "size": [BAR_WIDTH, BAR_HEIGHT],
        "hidden_value": HIDDEN,
        "authority": "server ratio only (0..100 shown, 0xFF hidden)",
        "ownership": "click_through",
        "native_geometry_complete": NATIVE_GEOMETRY_COMPLETE,
    }))
}

#[cfg(test)]
#[path = "native_hp_meter_tests.rs"]
mod tests;
