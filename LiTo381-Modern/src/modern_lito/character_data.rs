//! Source-persisted character and inventory inputs, never effective/live combat
//! values. These resource IDs are metadata, not native pointers or equip authority.

use super::application_protocol::{
    self as app, ApplicationError, CharacterSummary, SelectionContext,
};
use std::sync::Arc;

pub const REQUEST_KIND: u16 = 14;
pub const RESULT_KIND: u16 = 15;
pub const MAX_FRAME_BYTES: usize = 256 * 1024;
pub const MAX_ITEMS: usize = 512;
pub const MAX_NAME_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterDataFailure {
    Denied,
    BackendUnavailable,
    Busy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CharacterDataState {
    NotRequested,
    Unsupported,
    Unavailable(CharacterDataFailure),
    Ready(Arc<CharacterData>),
}
impl Default for CharacterDataState {
    fn default() -> Self {
        Self::NotRequested
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterDataReply {
    pub context: SelectionContext,
    pub result: Result<CharacterData, CharacterDataFailure>,
}

/// Stored AC may already include equipment, while max HP/MP are base maxima.
/// In particular saved HP/MP above those maxima are valid and must not be clamped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedStats {
    pub max_hp: i64,
    pub max_mp: i64,
    pub saved_hp: i64,
    pub saved_mp: i64,
    pub armor_class: i64,
    pub strength: i64,
    pub dexterity: i64,
    pub constitution: i64,
    pub intelligence: i64,
    pub wisdom: i64,
    pub charisma: i64,
    pub experience: i64,
    pub lawful: i64,
    pub original_strength: i64,
    pub original_dexterity: i64,
    pub original_constitution: i64,
    pub original_intelligence: i64,
    pub original_wisdom: i64,
    pub food225: i64,
}
impl SavedStats {
    pub fn food_percent(&self) -> Option<u8> {
        (0..=225)
            .contains(&self.food225)
            .then(|| (self.food225 * 100 / 225) as u8)
    }
    fn valid(&self) -> bool {
        self.max_hp >= 1
            && self.max_mp >= 0
            && self.saved_hp >= 0
            && self.saved_mp >= 0
            && [
                self.strength,
                self.dexterity,
                self.constitution,
                self.intelligence,
                self.wisdom,
                self.charisma,
                self.experience,
                self.original_strength,
                self.original_dexterity,
                self.original_constitution,
                self.original_intelligence,
                self.original_wisdom,
            ]
            .iter()
            .all(|value| *value >= 0)
            && (0..=225).contains(&self.food225)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemKind {
    Weapon,
    Armor,
    Etc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryItem {
    pub object_id: u64,
    pub catalog_id: u64,
    pub quantity: i64,
    pub enchantment: i64,
    pub durability: i64,
    pub blessing: i64,
    pub charge_count: i64,
    pub inventory_graphic: i64,
    pub ground_graphic: i64,
    pub unit_weight: i64,
    pub kind: ItemKind,
    pub equipped: bool,
    pub identified: bool,
    pub sealed: bool,
    /// Raw catalog token, possibly `$descriptionID`. No localization was resolved.
    pub name_token: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterData {
    pub context: SelectionContext,
    pub character: CharacterSummary,
    pub stats: SavedStats,
    /// Strictly increasing instance object IDs, not catalog IDs or UI indexes.
    pub items: Vec<InventoryItem>,
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid source-persisted character data",
    )
}
fn valid_id(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_NAME_BYTES
        && !value.trim().is_empty()
        && !value.chars().any(char::is_control)
}
impl CharacterData {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.character.key != self.context.key
            || !valid_text(&self.character.name)
            || self.character.level < 1
            || self.character.sex > 1
            || !self.stats.valid()
            || self.items.len() > MAX_ITEMS
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for item in &self.items {
            if !valid_id(item.object_id)
                || item.object_id <= previous
                || !valid_id(item.catalog_id)
                || item.quantity <= 0
                || item.durability < 0
                || !(0..=255).contains(&item.blessing)
                || item.charge_count < 0
                || item.inventory_graphic < 0
                || item.ground_graphic < 0
                || item.unit_weight < 0
                || !valid_text(&item.name_token)
            {
                return Err(invalid());
            }
            previous = item.object_id;
        }
        Ok(())
    }
}

pub(crate) fn encode_request(
    request_id: u32,
    context: SelectionContext,
) -> Result<Vec<u8>, ApplicationError> {
    if !app::valid_context(context) {
        return Err(app::error(
            app::ErrorKind::InvalidIdentity,
            "invalid character data context",
        ));
    }
    let mut frame = app::header(request_id, REQUEST_KIND, 36)?;
    frame.extend_from_slice(&context.key.source_id.to_be_bytes());
    frame.extend_from_slice(&context.key.object_id.to_be_bytes());
    frame.extend_from_slice(&context.epoch.to_be_bytes());
    Ok(frame)
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.offset.checked_add(count).ok_or_else(invalid)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(invalid)?;
        self.offset = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ApplicationError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, ApplicationError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, ApplicationError> {
        Ok(self.u64()? as i64)
    }
    fn text(&mut self) -> Result<String, ApplicationError> {
        let length = self.u16()? as usize;
        if length == 0 || length > MAX_NAME_BYTES {
            return Err(invalid());
        }
        let text = std::str::from_utf8(self.take(length)?).map_err(|_| invalid())?;
        if !valid_text(text) {
            return Err(invalid());
        }
        Ok(text.to_owned())
    }
}

pub(crate) fn decode_body(bytes: &[u8]) -> Result<CharacterDataReply, ApplicationError> {
    if bytes.len() > MAX_FRAME_BYTES - 16 {
        return Err(invalid());
    }
    let mut input = Reader { bytes, offset: 0 };
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: input.u32()?,
            object_id: input.u64()?,
        },
        epoch: input.u64()?,
    };
    if !app::valid_context(context) {
        return Err(invalid());
    }
    let status = input.u8()?;
    if input.take(3)? != [0, 0, 0] {
        return Err(invalid());
    }
    let failure = match status {
        0 => None,
        1 => Some(CharacterDataFailure::Denied),
        2 => Some(CharacterDataFailure::BackendUnavailable),
        3 => Some(CharacterDataFailure::Busy),
        _ => return Err(invalid()),
    };
    if let Some(failure) = failure {
        if input.offset != bytes.len() {
            return Err(invalid());
        }
        return Ok(CharacterDataReply {
            context,
            result: Err(failure),
        });
    }
    let object_id = input.u64()?;
    let name = input.text()?;
    let class = input.u16()?;
    let sex = input.u8()?;
    if input.u8()? != 0 || object_id != context.key.object_id {
        return Err(invalid());
    }
    let character = CharacterSummary {
        key: context.key,
        name,
        class,
        sex,
        level: input.i64()?,
    };
    let stats = SavedStats {
        max_hp: input.i64()?,
        max_mp: input.i64()?,
        saved_hp: input.i64()?,
        saved_mp: input.i64()?,
        armor_class: input.i64()?,
        strength: input.i64()?,
        dexterity: input.i64()?,
        constitution: input.i64()?,
        intelligence: input.i64()?,
        wisdom: input.i64()?,
        charisma: input.i64()?,
        experience: input.i64()?,
        lawful: input.i64()?,
        original_strength: input.i64()?,
        original_dexterity: input.i64()?,
        original_constitution: input.i64()?,
        original_intelligence: input.i64()?,
        original_wisdom: input.i64()?,
        food225: input.i64()?,
    };
    let count = input.u16()? as usize;
    if input.u16()? != 0
        || count > MAX_ITEMS
        || count.checked_mul(87).ok_or_else(invalid)? > bytes.len() - input.offset
    {
        return Err(invalid());
    }
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = input.u64()?;
        let catalog_id = input.u64()?;
        let quantity = input.i64()?;
        let enchantment = input.i64()?;
        let durability = input.i64()?;
        let blessing = input.i64()?;
        let charge_count = input.i64()?;
        let inventory_graphic = input.i64()?;
        let ground_graphic = input.i64()?;
        let unit_weight = input.i64()?;
        let kind = match input.u8()? {
            0 => ItemKind::Weapon,
            1 => ItemKind::Armor,
            2 => ItemKind::Etc,
            _ => return Err(invalid()),
        };
        let flags = input.u8()?;
        if flags & !7 != 0 || input.u16()? != 0 {
            return Err(invalid());
        }
        let name_token = input.text()?;
        items.push(InventoryItem {
            object_id,
            catalog_id,
            quantity,
            enchantment,
            durability,
            blessing,
            charge_count,
            inventory_graphic,
            ground_graphic,
            unit_weight,
            kind,
            equipped: flags & 1 != 0,
            identified: flags & 2 != 0,
            sealed: flags & 4 != 0,
            name_token,
        });
    }
    if input.offset != bytes.len() {
        return Err(invalid());
    }
    let data = CharacterData {
        context,
        character,
        stats,
        items,
    };
    data.validate()?;
    Ok(CharacterDataReply {
        context,
        result: Ok(data),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn context() -> SelectionContext {
        SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    fn text(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    pub(crate) fn fixture_body(count: usize) -> Vec<u8> {
        fixture_body_named(count, "$123")
    }
    pub(crate) fn fixture_body_named(count: usize, name_token: &str) -> Vec<u8> {
        let context = context();
        let mut body = Vec::new();
        body.extend_from_slice(&context.key.source_id.to_be_bytes());
        body.extend_from_slice(&context.key.object_id.to_be_bytes());
        body.extend_from_slice(&context.epoch.to_be_bytes());
        body.extend_from_slice(&[0; 4]);
        body.extend_from_slice(&context.key.object_id.to_be_bytes());
        text(&mut body, "勇者");
        body.extend_from_slice(&[0, 5, 1, 0]);
        body.extend_from_slice(&4_000_000_000i64.to_be_bytes());
        for value in [
            500,
            300,
            i64::MAX,
            400,
            i64::MIN,
            1,
            2,
            3,
            4,
            5,
            6,
            i64::MAX,
            i64::MIN,
            7,
            8,
            9,
            10,
            11,
            225,
        ] {
            body.extend_from_slice(&value.to_be_bytes());
        }
        body.extend_from_slice(&(count as u16).to_be_bytes());
        body.extend_from_slice(&[0; 2]);
        for index in 0..count {
            for value in [
                index as i64 + 1,
                70000,
                i64::MAX,
                i64::MIN,
                1,
                255,
                2,
                i64::MAX,
                3,
                4,
            ] {
                body.extend_from_slice(&value.to_be_bytes());
            }
            body.extend_from_slice(&[index as u8 % 3, 7, 0, 0]);
            text(&mut body, name_token);
        }
        body
    }
    pub(crate) fn sample_data() -> CharacterData {
        decode_body(&fixture_body(2)).unwrap().result.unwrap()
    }

    #[test]
    fn persisted_fields_are_lossless_ordered_and_never_effective_or_clamped() {
        let data = sample_data();
        assert_eq!(data.context, context());
        assert_eq!(data.character.level, 4_000_000_000);
        assert_eq!(data.stats.saved_hp, i64::MAX);
        assert!(data.stats.saved_hp > data.stats.max_hp);
        assert_eq!(data.stats.saved_mp, 400);
        assert_eq!(data.stats.armor_class, i64::MIN);
        assert_eq!(data.stats.lawful, i64::MIN);
        assert_eq!(data.stats.experience, i64::MAX);
        assert_eq!(
            [
                data.stats.strength,
                data.stats.dexterity,
                data.stats.constitution,
                data.stats.intelligence,
                data.stats.wisdom,
                data.stats.charisma
            ],
            [1, 2, 3, 4, 5, 6]
        );
        assert_eq!(
            [
                data.stats.original_strength,
                data.stats.original_dexterity,
                data.stats.original_constitution,
                data.stats.original_intelligence,
                data.stats.original_wisdom
            ],
            [7, 8, 9, 10, 11]
        );
        assert_eq!(data.stats.food225, 225);
        assert_eq!(data.stats.food_percent(), Some(100));
        assert_eq!(data.items[0].quantity, i64::MAX);
        assert_eq!(data.items[0].enchantment, i64::MIN);
        assert_eq!(data.items[0].inventory_graphic, i64::MAX);
        assert_eq!(data.items[0].name_token, "$123");
        assert!(data.items[0].equipped && data.items[0].identified && data.items[0].sealed);
        assert_eq!(data.items[1].kind, ItemKind::Armor);
    }
    #[test]
    fn all_failure_statuses_have_no_payload_and_the_request_matches_java_golden() {
        let request = encode_request(0xfedcba98, context()).unwrap();
        assert_eq!(
            request
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            "4c544d3100000024fedcba98000e0000000000070000000000000d05ffffffffffffffff"
        );
        assert!(encode_request(
            1,
            SelectionContext {
                epoch: 0,
                ..context()
            }
        )
        .is_err());
        for (status, expected) in [
            (1, CharacterDataFailure::Denied),
            (2, CharacterDataFailure::BackendUnavailable),
            (3, CharacterDataFailure::Busy),
        ] {
            let mut body = fixture_body(0);
            body.truncate(24);
            body[20] = status;
            let result = decode_body(&body).unwrap();
            assert_eq!(result.context, context());
            assert_eq!(result.result, Err(expected));
            body.push(0);
            assert!(decode_body(&body).is_err());
        }
    }
    #[test]
    fn counts_ordering_text_reserved_flags_and_every_short_read_fail_closed() {
        let original = fixture_body(2);
        for length in 0..original.len() {
            assert!(decode_body(&original[..length]).is_err(), "length {length}");
        }
        let item_start = fixture_body(0).len();
        let mut invalid = original.clone();
        invalid.push(0);
        assert!(decode_body(&invalid).is_err());
        for (offset, value) in [
            (20, 4),
            (21, 1),
            (31, 0),
            (32, 2),
            (43, 1),
            (item_start + 80, 3),
            (item_start + 81, 8),
            (item_start + 82, 1),
            (item_start + 84, 2),
            (item_start + 86, 0),
        ] {
            let mut bytes = original.clone();
            bytes[offset] = value;
            assert!(decode_body(&bytes).is_err(), "offset {offset}");
        }
        let mut bytes = original.clone();
        bytes[item_start + 90..item_start + 98].copy_from_slice(&1u64.to_be_bytes());
        assert!(decode_body(&bytes).is_err(), "duplicate instance ID");
        assert!(decode_body(&fixture_body(MAX_ITEMS)).is_ok());
        assert!(decode_body(&fixture_body(MAX_ITEMS + 1)).is_err());
    }
    #[test]
    fn validation_rejects_invalid_ranges_but_accepts_zero_food_and_signed_fields() {
        let original = sample_data();
        for index in 0..18 {
            let mut data = original.clone();
            match index {
                0 => data.stats.max_hp = 0,
                1 => data.stats.saved_hp = -1,
                2 => data.stats.max_mp = -1,
                3 => data.stats.food225 = 226,
                4 => data.stats.food225 = -1,
                5 => data.stats.strength = -1,
                6 => data.items[0].quantity = 0,
                7 => data.items[0].blessing = 256,
                8 => data.items[0].catalog_id = u64::MAX,
                9 => data.items[0].name_token = "\n".into(),
                10 => data.items[0].durability = -1,
                11 => data.items[0].charge_count = -1,
                12 => data.items[0].inventory_graphic = -1,
                13 => data.items[0].ground_graphic = -1,
                14 => data.items[0].unit_weight = -1,
                15 => data.character.sex = 2,
                16 => data.character.level = 0,
                _ => data.items[0].name_token = "x".repeat(MAX_NAME_BYTES + 1),
            }
            assert!(data.validate().is_err(), "case {index}");
        }
        let mut data = original;
        data.stats.food225 = 0;
        data.validate().unwrap();
        assert_eq!(data.stats.food_percent(), Some(0));
        data.stats.food225 = 224;
        assert_eq!(data.stats.food_percent(), Some(99));
    }
}
