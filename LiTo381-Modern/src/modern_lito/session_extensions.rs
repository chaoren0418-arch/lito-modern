//! Explicit second capability word. Kinds 80/81 extend exact base v47/schema1;
//! no legacy bit reuse, authentication grant or implicit gameplay admission.
use super::*;
pub const BASE_VERSION: u16 = 47;
pub const HELLO_KIND: u16 = 80;
pub const ACK_KIND: u16 = 81;
pub const PLAYER_HEALING_DATA: u32 = 1;
pub const PLAYER_HEALING_COMMANDS: u32 = 2;
pub const ARMOR_PROTECTION_DATA: u32 = 4;
pub const ARMOR_PRESENTATION_DATA: u32 = 8;
pub const FULL_VIP_DATA: u32 = 16;
pub const FULL_VIP_EQUIPMENT: u32 = 32;
pub const FULL_VIP_WORLD: u32 = 64;
pub const FULL_VIP_APPEARANCE: u32 = 128;
pub const KNOWN: u32 = 255;
pub const HEALING_MASK: u32 = PLAYER_HEALING_DATA | PLAYER_HEALING_COMMANDS;
pub const DEPENDENCIES: u32 = CAP_EXPLICIT_I64_STATS
    | CAP_SOURCE_SCOPED_IDENTITIES
    | CAP_TOURNAMENT_NAVIGATION
    | CAP_CHARACTER_DATA
    | CAP_EQUIPMENT
    | CAP_ROOMS
    | CAP_WORLD
    | CAP_COMBAT
    | CAP_SHOP
    | CAP_WEIGHT
    | CAP_TIMED
    | CAP_REGEN
    | CAP_MATCH_DESCRIPTOR
    | CAP_BATTLE_DATA
    | CAP_ACTION_LIFE;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub supported: u32,
    pub required: u32,
}
pub const HEALING: Offer = Offer {
    supported: HEALING_MASK,
    required: HEALING_MASK,
};
pub const FULL_VIP: Offer = Offer {
    supported: FULL_VIP_DATA | FULL_VIP_EQUIPMENT | FULL_VIP_WORLD,
    required: FULL_VIP_DATA | FULL_VIP_EQUIPMENT | FULL_VIP_WORLD,
};
pub const FULL_VIP_V2: Offer = Offer {
    supported: FULL_VIP.supported | FULL_VIP_APPEARANCE,
    required: FULL_VIP.required | FULL_VIP_APPEARANCE,
};
pub const ARMOR: Offer = Offer {
    supported: ARMOR_PROTECTION_DATA,
    required: ARMOR_PROTECTION_DATA,
};
pub const HEALING_ARMOR: Offer = Offer {
    supported: HEALING_MASK | ARMOR_PROTECTION_DATA,
    required: HEALING_MASK | ARMOR_PROTECTION_DATA,
};
pub const ARMOR_PRESENTATION: Offer = Offer {
    supported: ARMOR_PROTECTION_DATA | ARMOR_PRESENTATION_DATA,
    required: ARMOR_PROTECTION_DATA | ARMOR_PRESENTATION_DATA,
};
pub const HEALING_ARMOR_PRESENTATION: Offer = Offer {
    supported: HEALING_MASK | ARMOR_PROTECTION_DATA | ARMOR_PRESENTATION_DATA,
    required: HEALING_MASK | ARMOR_PROTECTION_DATA | ARMOR_PRESENTATION_DATA,
};
fn closed(bits: u32) -> bool {
    (bits & PLAYER_HEALING_COMMANDS == 0 || bits & PLAYER_HEALING_DATA != 0)
        && (bits & ARMOR_PRESENTATION_DATA == 0 || bits & ARMOR_PROTECTION_DATA != 0)
        && (bits & (FULL_VIP_EQUIPMENT | FULL_VIP_WORLD | FULL_VIP_APPEARANCE) == 0
            || bits & FULL_VIP_DATA != 0)
}
fn fail() -> ProtocolError {
    error(
        ErrorKind::UnsupportedCapability,
        "invalid session extensions",
    )
}
fn bits(base: u32, ext: u32) -> Result<(), ProtocolError> {
    if ext & !KNOWN != 0 || !closed(ext) || (ext != 0 && base & DEPENDENCIES != DEPENDENCIES) {
        return Err(fail());
    }
    Ok(())
}
pub(super) fn validate_offer(base: Hello, offer: Offer) -> Result<(), ProtocolError> {
    if base.min_version != BASE_VERSION
        || base.max_version != BASE_VERSION
        || base.numeric_schema != NUMERIC_SCHEMA_I64_V1
        || offer.required & !offer.supported != 0
    {
        return Err(fail());
    }
    bits(base.supported_capabilities, offer.supported)?;
    bits(base.required_capabilities, offer.required)
}
pub(super) fn validate_selected(base: Ack, extensions: u32) -> Result<(), ProtocolError> {
    if base.version != BASE_VERSION || base.numeric_schema != NUMERIC_SCHEMA_I64_V1 {
        return Err(fail());
    }
    bits(base.capabilities, extensions)
}
pub fn allows_data(base: Ack, extensions: u32) -> bool {
    extensions & PLAYER_HEALING_DATA != 0 && validate_selected(base, extensions).is_ok()
}
pub fn allows_commands(base: Ack, extensions: u32) -> bool {
    extensions & HEALING_MASK == HEALING_MASK && validate_selected(base, extensions).is_ok()
}
pub fn allows_armor(base: Ack, extensions: u32) -> bool {
    extensions & ARMOR_PROTECTION_DATA != 0 && validate_selected(base, extensions).is_ok()
}
pub fn allows_armor_presentation(base: Ack, extensions: u32) -> bool {
    extensions & ARMOR_PRESENTATION.required == ARMOR_PRESENTATION.required
        && validate_selected(base, extensions).is_ok()
}
pub fn allows_vip_data(base: Ack, extensions: u32) -> bool {
    extensions & FULL_VIP_DATA != 0 && validate_selected(base, extensions).is_ok()
}
pub fn allows_vip_equipment(base: Ack, extensions: u32) -> bool {
    extensions & (FULL_VIP_DATA | FULL_VIP_EQUIPMENT) == (FULL_VIP_DATA | FULL_VIP_EQUIPMENT)
        && validate_selected(base, extensions).is_ok()
}
pub fn allows_vip_world(base: Ack, extensions: u32) -> bool {
    extensions & (FULL_VIP_DATA | FULL_VIP_WORLD) == (FULL_VIP_DATA | FULL_VIP_WORLD)
        && validate_selected(base, extensions).is_ok()
}

/// Complete compatibility selection. Not an authenticated owner or permission.
/// Keeping the extension word here prevents accidental admission from v47 alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selected {
    base: Ack,
    extensions: u32,
}
/// Compatibility for internal version-only callers; it can never select an
/// extension. Live streams must use their verified complete HELLO/ACK selection.
impl From<Ack> for Selected {
    fn from(base: Ack) -> Self {
        Self::legacy(base)
    }
}
impl Selected {
    /// Internal version-only callers retain their existing local gates. This
    /// cannot enable any extension and is not a replacement for wire ACK checks.
    pub(crate) fn legacy(base: Ack) -> Self {
        Self {
            base,
            extensions: 0,
        }
    }
    pub fn new(base: Ack, extensions: u32) -> Result<Self, ProtocolError> {
        super::validate(Message::Ack(base))?;
        if extensions != 0 {
            validate_selected(base, extensions)?;
        }
        Ok(Self { base, extensions })
    }
    pub fn base(self) -> Ack {
        self.base
    }
    pub fn extensions(self) -> u32 {
        self.extensions
    }
    pub fn allows_data(self) -> bool {
        allows_data(self.base, self.extensions)
    }
    pub fn allows_commands(self) -> bool {
        allows_commands(self.base, self.extensions)
    }
    pub fn allows_armor(self) -> bool {
        allows_armor(self.base, self.extensions)
    }
    pub fn allows_armor_presentation(self) -> bool {
        allows_armor_presentation(self.base, self.extensions)
    }
    pub fn allows_vip_data(self) -> bool {
        allows_vip_data(self.base, self.extensions)
    }
    pub fn allows_vip_appearance(self) -> bool {
        self.allows_vip_data() && self.extensions & FULL_VIP_APPEARANCE != 0
    }
    pub fn allows_vip_equipment(self) -> bool {
        allows_vip_equipment(self.base, self.extensions)
    }
    pub fn allows_vip_world(self) -> bool {
        allows_vip_world(self.base, self.extensions)
    }
}
