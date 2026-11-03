//! GAME-epoch environment authority, independent of replaceable World scenes.
//! Calendar policy is the pinned C02 Asia/Taipei contract, including historical
//! DST. This does not resolve the original client's imported calendar API.
//! No local clock extrapolation or framebuffer/night shader is implemented here.
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Calendar {
    pub raw: u32,
    pub offset_seconds: i32,
    pub year: u16,
    pub day_of_year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
impl Calendar {
    pub fn from_raw(raw: u32) -> Result<Self> {
        ensure!(raw <= i32::MAX as u32, "environment time exceeds retained native time_t");
        // C02 declared zone, JDK tzdb2025b. Six transitions in raw0..INT_MAX.
        const TRANSITIONS: [(u32, i32); 6] = [
            (133977600, 32400), (149785200, 28800),
            (165513600, 32400), (181321200, 28800),
            (299606400, 32400), (307551600, 28800),
        ];
        let offset_seconds = TRANSITIONS.iter().rev().find(|(at, _)| raw >= *at)
            .map_or(28800, |(_, offset)| *offset);
        let seconds = u64::from(raw) + offset_seconds as u64;
        let mut days = (seconds / 86400) as u32;
        let mut year: u16 = 1970;
        let leap = |y: u16| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        while days >= if leap(year) { 366 } else { 365 } {
            days -= if leap(year) { 366 } else { 365 };
            year += 1;
        }
        let day_of_year = days as u16;
        let months = [31, if leap(year) {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 0;
        while days >= months[month] { days -= months[month]; month += 1; }
        Ok(Self { raw, offset_seconds, year, day_of_year, month: month as u8 + 1,
            day: days as u8 + 1, hour: ((seconds / 3600) % 24) as u8,
            minute: ((seconds / 60) % 60) as u8, second: (seconds % 60) as u8 })
    }
    pub fn minute_of_day(self) -> u16 { u16::from(self.hour) * 60 + u16::from(self.minute) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Daylight { pub level: u8, pub sunrise: u16, pub sunset: u16, pub minute: u16 }
pub fn daylight(day: u16, minute: u16, weather: [u8; 3]) -> Result<Daylight> {
    ensure!(day < 366 && minute < 1440, "daylight calendar bounds");
    let (day, m) = (i32::from(day), i32::from(minute));
    // Original4D02F0 integer divisions truncate toward zero.
    let rise = if day <= 172 {452 - day * 192 / 172} else {260 + day - 173};
    let set = if day <= 172 {986 + day * 197 / 172} else {1183 + (173 - day) * 197 / 192};
    let value = if rise - 40 < m && m < rise {1 + (m - rise + 40) * 15 / 40}
        else if rise <= m && m <= set {15}
        else if set < m && m < set + 40 {1 + (set - m + 40) * 15 / 40}
        else {2};
    let mut level = (5 + 2 * value / 3).min(15);
    if level >= 10 && weather.iter().any(|v| *v != 0) { level -= 1; }
    Ok(Daylight { level: level.max(5) as u8, sunrise: rise as u16, sunset: set as u16, minute })
}

// Filled from the retained original237-entry map table, not inferred map names.
const SPECIAL_MAPS: &[u16] = &[
    1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 36,
    37, 38, 39, 40, 41, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
    54, 55, 56, 59, 60, 61, 62, 63, 64, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 85, 86, 87, 101, 102, 103, 104, 105, 106, 107, 108, 109,
    110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
    126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141,
    142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157,
    158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189,
    190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205,
    206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221,
    222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237,
    301, 304, 305, 306, 307, 308, 309, 310, 320, 330, 666, 1000, 1001, 1002, 1003, 1004,
    1005, 16384, 17408, 18432, 18944, 19456, 19968, 20480, 20992, 511, 513, 527, 553,
];
pub fn map_policy_flag(map: u16) -> bool { map >= 0x4000 || SPECIAL_MAPS.contains(&map) }
pub fn map_mask_argument(map: u16, flag: u8) -> u8 {
    if flag == 0 { return if (547..=550).contains(&map) {8} else {15}; }
    match map {
        72..=74 => 4, 75..=77 => 11,
        307..=310 | 320 | 330 | 511 | 527 => 2,
        _ => 1,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct State {
    pub epoch: Option<u64>,
    retired_through: u64,
    pub revision: u64,
    pub calendar: Option<Calendar>,
    pub daylight: Option<Daylight>,
    pub ambient_level: Option<u8>,
    /// Original648EFC suppresses time-based ambient writes only when positive.
    /// Effect override callbacks are a separate, not yet wired render scope.
    pub ambient_override_counter: i32,
    pub blind: u8,
    pub weather: [u8; 3],
    pub last_weather_raw: Option<u8>,
    pub map: Option<u16>,
    /// Set by S28, retained across S5A, cleared on explicit world retirement.
    pub underwater: bool,
    pub map_mask_argument: Option<u8>,
    pub time_packets: u64,
    pub status_time_packets: u64,
    pub water_clock:super::native_water_clock::Owned,
    #[serde(skip)]
    water_origin:Option<std::time::Instant>,
}
impl State {
    pub fn begin(&mut self, epoch: u64) -> Result<bool> {
        ensure!(epoch != 0, "zero GAME environment epoch");
        if self.epoch == Some(epoch) { return Ok(false); }
        let last = self.epoch.unwrap_or(0).max(self.retired_through);
        ensure!(epoch > last, "retired GAME environment epoch");
        let mut water_clock=self.water_clock.clone();water_clock.begin(epoch)?;
        *self = Self { epoch: Some(epoch), retired_through: last,water_clock,
            water_origin:Some(std::time::Instant::now()), ..Default::default() };
        Ok(true)
    }
    pub fn retire(&mut self) {
        let last = self.epoch.unwrap_or(0).max(self.retired_through);
        let mut water_clock=std::mem::take(&mut self.water_clock);water_clock.retire();
        *self = Self { retired_through: last,water_clock, ..Default::default() };
    }
    pub fn retire_scene(&mut self) {
        let changed=self.underwater;
        self.water_clock.retire_scene();self.underwater=false;
        if changed {self.revision=self.revision.saturating_add(1);}
    }
    /// Accepted World generation boundary, including a destination on the same
    /// numeric map. Old visual leases cannot suppress the destination's current
    /// server calendar/weather or restore an earlier scene's saved brightness.
    pub fn ambient_scene_changed(&mut self,epoch:u64,map:u16)->Result<bool> {
        self.owner(epoch)?;ensure!(self.map==Some(map),"ambient destination map not accepted");
        let mut next=self.clone();next.ambient_override_counter=0;
        next.daylight=next.calendar.map(|calendar|daylight(calendar.day_of_year,calendar.minute_of_day(),next.weather)).transpose()?;
        next.ambient_level=if map_policy_flag(map){Some(1)}else{next.daylight.map(|value|value.level)};
        if self.ambient_override_counter==0 && self.daylight==next.daylight && self.ambient_level==next.ambient_level {return Ok(false)}
        next.bump()?;*self=next;Ok(true)
    }
    fn owner(&self, epoch: u64) -> Result<()> {
        ensure!(self.epoch == Some(epoch) && epoch != 0, "stale GAME environment update");
        Ok(())
    }
    fn bump(&mut self) -> Result<()> {
        self.revision = self.revision.checked_add(1).context("environment revision exhausted")?;
        Ok(())
    }
    fn time(&mut self, raw: u32, status: bool) -> Result<()> {
        let calendar = Calendar::from_raw(raw)?;
        let daylight = daylight(calendar.day_of_year, calendar.minute_of_day(), self.weather)?;
        self.calendar = Some(calendar);
        self.daylight = Some(daylight);
        if self.map.is_none_or(|map| !map_policy_flag(map)) && self.ambient_override_counter <= 0 {
            self.ambient_level = Some(daylight.level);
        }
        if status { self.status_time_packets = self.status_time_packets.saturating_add(1); }
        else { self.time_packets = self.time_packets.saturating_add(1); }
        Ok(())
    }
    /// Called only after the World controller has accepted the destination.
    pub fn map_changed(&mut self, epoch: u64, map: u16, underwater: Option<u8>) -> Result<bool> {
        self.owner(epoch)?;
        let water = underwater.map_or(self.underwater, |raw| raw != 0);
        let mut next = self.clone();
        if underwater.is_some() {
            let origin=next.water_origin.context("water clock origin absent")?;
            next.water_clock.packet(epoch,water,||origin.elapsed().as_millis() as u32)?;
        }
        if self.map == Some(map) && self.underwater == water {
            self.water_clock=next.water_clock;return Ok(false);
        }
        let changed_map = next.map != Some(map);
        next.map = Some(map);
        next.underwater = water;
        next.map_mask_argument = Some(map_mask_argument(map, u8::from(map_policy_flag(map))));
        if changed_map {
            next.daylight = next.calendar.map(|c| daylight(c.day_of_year, c.minute_of_day(), next.weather)).transpose()?;
            next.ambient_level = if map_policy_flag(map) {Some(1)} else {next.daylight.map(|v| v.level)};
        }
        next.bump()?;
        *self = next;
        Ok(true)
    }
    /// The accepted world generation becomes ready after its authority bundle;
    /// this is the modern map-load/green-start boundary, independent of async
    /// asset reads. Both visual callbacks share the original deadline ordering.
    pub fn water_poll(&mut self,epoch:u64,generation:Option<u64>)->Result<bool> {
        self.owner(epoch)?;let origin=self.water_origin.context("water clock origin absent")?;
        let mut next=self.water_clock.clone();
        if let Some(generation)=generation {next.loaded(epoch,generation,||origin.elapsed().as_millis() as u32)?;}
        let fired=next.tick(epoch,||origin.elapsed().as_millis() as u32)?;
        self.water_clock=next;if fired {self.bump()?;}Ok(fired)
    }
    /// Known but ignored S0A values are consumed without state mutation.
    /// S0C's raw zero is retained; C02 deliberately writes that value.
    pub fn accept(&mut self, epoch: u64, body: &[u8]) -> Result<bool> {
        let Some(&opcode) = body.first() else { return Ok(false); };
        if !matches!(opcode, 0x0a | 0x0c | 0x21 | 0x33) { return Ok(false); }
        self.owner(epoch)?;
        let mut next = self.clone();
        match opcode {
            0x0c => next.time(super::native_city_runtime::status_receipt::decode(body)?.game_time, true)?,
            0x21 => {
                ensure!(body.len() == 5, "S_TIME exact five-byte body");
                next.time(u32::from_le_bytes(body[1..].try_into()?), false)?;
            }
            0x0a => {
                ensure!(body.len() == 2, "S_BLIND exact two-byte body");
                let value = body[1] as i8;
                if !(0..3).contains(&value) || value as u8 == next.blind { return Ok(true); }
                next.blind = value as u8;
            }
            0x33 => {
                ensure!(body.len() == 2, "S_WEATHER exact two-byte body");
                let raw = body[1];
                let value = if raw & 15 <= 3 {raw & 15} else {0};
                if value != 0 {
                    if let Some(level) = &mut next.ambient_level { if *level >= 10 { *level -= 1; } }
                }
                match raw & 0xf0 { 0 => next.weather[0] = value,
                    0x10 => next.weather[1] = value, 0x20 => next.weather[2] = value, _ => {} }
                next.last_weather_raw = Some(raw);
            }
            _ => unreachable!(),
        }
        next.bump()?;
        *self = next;
        Ok(true)
    }
}
