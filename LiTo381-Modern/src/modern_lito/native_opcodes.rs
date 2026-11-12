//! Client (`C_*`) and server (`S_*`) opcode constants from
//! `docs/PROTOCOL-CONTRACT.md` §2–§5. Names match `Protocol.java`.
//!
//! The same numeric value can mean different packets in each direction
//! (for example `C_LITO_LIN_CHARACTER_INFO` and `S_LITO_SELECT_CHAR_RESULT`
//! are both `0xC8`). Direction-specific constants must not be shared.

// --- §2 first plaintext frame ---
pub const S_KEY: u8 = 0x01;

// Legacy integrity contract: JAR E1498871 Protocol.addressChallenge/addressProof.
pub const S_EVENT: u8 = 0x7B;
pub const C_ADDR: u8 = 0x2A;

// --- §4 handshake / LOGIN / GAME bootstrap ---
pub const C_VERSION: u8 = 0x00;
pub const C_LOGIN: u8 = 0x01;
pub const C_READ_NEWS: u8 = 0x33;
pub const C_SERVER_SELECT: u8 = 0x35;
pub const C_READ_NOTICE: u8 = 0x46;
pub const C_LOGIN_RESULT: u8 = 0x61;
pub const C_LITO_LIN_CHARACTER_INFO: u8 = 0xC8;
pub const C_LITO_SELECT_CHAR: u8 = 0xC9;
pub const C_LITO_READ_NEWS: u8 = 0xE2;

pub const S_LOGIN_CHECK: u8 = 0x00;
pub const S_NEW_ACCOUNT_CHECK: u8 = 0x02;
pub const S_NUM_CHARACTER: u8 = 0x03;
pub const S_NEWS: u8 = 0x38;
pub const S_VERSION_CHECK: u8 = 0x3A;
pub const S_SERVER_LIST: u8 = 0x3B;
pub const S_NOTICE: u8 = 0x4B;
pub const S_LITO_SELECT_CHAR_RESULT: u8 = 0xC8;
pub const S_LITO_CHARACTER_INFO: u8 = 0xCB;
pub const S_LITO_NEWS: u8 = 0xE6;
pub const S_LITO_LOGIN_OK: u8 = 0xE7;
pub const S_LITO_TIMELIMIT: u8 = 0xE9;
pub const S_LITO_VS_SERVER_STATUS: u8 = 0xF3;
pub const S_LITO_VS_SERVER_IP: u8 = 0xF4;
pub const S_LITO_SERVER_LIST: u8 = 0xFB;

// --- §5 lobby / channel / room ---
pub const C_LITO_ENTER_CHANNEL: u8 = 0xCA;
pub const C_LITO_LEAVE_CHANNEL: u8 = 0xCB;
pub const C_LITO_CREATE_ROOM: u8 = 0xCC;
pub const C_LITO_ROOM_USER: u8 = 0xCD;
pub const C_LITO_ROOM_LIST: u8 = 0xCE;
pub const C_LITO_JOIN: u8 = 0xCF;
pub const C_LITO_EXIT_ROOM: u8 = 0xD0;

pub const S_LITO_ENTER_CHANNEL_RESULT: u8 = 0xC9;
pub const S_LITO_CHANNEL_INFO: u8 = 0xCA;
pub const S_LITO_ROOM_INFO: u8 = 0xCC;
pub const S_LITO_ROOM_USER: u8 = 0xCD;
pub const S_LITO_JOIN_RESULT: u8 = 0xCE;
pub const S_LITO_DELETE_ROOM: u8 = 0xCF;
pub const S_LITO_EXIT_ROOM: u8 = 0xD0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_constants_match_protocol_contract_sections_2_to_5() {
        assert_eq!(S_KEY, 0x01);

        assert_eq!(C_VERSION, 0x00);
        assert_eq!(C_LOGIN, 0x01);
        assert_eq!(C_READ_NEWS, 0x33);
        assert_eq!(C_SERVER_SELECT, 0x35);
        assert_eq!(C_READ_NOTICE, 0x46);
        assert_eq!(C_LOGIN_RESULT, 0x61);
        assert_eq!(C_LITO_LIN_CHARACTER_INFO, 0xC8);
        assert_eq!(C_LITO_SELECT_CHAR, 0xC9);
        assert_eq!(C_LITO_READ_NEWS, 0xE2);

        assert_eq!(S_LOGIN_CHECK, 0x00);
        assert_eq!(S_NEW_ACCOUNT_CHECK, 0x02);
        assert_eq!(S_NUM_CHARACTER, 0x03);
        assert_eq!(S_NEWS, 0x38);
        assert_eq!(S_VERSION_CHECK, 0x3A);
        assert_eq!(S_SERVER_LIST, 0x3B);
        assert_eq!(S_NOTICE, 0x4B);
        assert_eq!(S_LITO_SELECT_CHAR_RESULT, 0xC8);
        assert_eq!(S_LITO_CHARACTER_INFO, 0xCB);
        assert_eq!(S_LITO_NEWS, 0xE6);
        assert_eq!(S_LITO_LOGIN_OK, 0xE7);
        assert_eq!(S_LITO_TIMELIMIT, 0xE9);
        assert_eq!(S_LITO_VS_SERVER_STATUS, 0xF3);
        assert_eq!(S_LITO_VS_SERVER_IP, 0xF4);
        assert_eq!(S_LITO_SERVER_LIST, 0xFB);

        assert_eq!(C_LITO_ENTER_CHANNEL, 0xCA);
        assert_eq!(C_LITO_LEAVE_CHANNEL, 0xCB);
        assert_eq!(C_LITO_CREATE_ROOM, 0xCC);
        assert_eq!(C_LITO_ROOM_USER, 0xCD);
        assert_eq!(C_LITO_ROOM_LIST, 0xCE);
        assert_eq!(C_LITO_JOIN, 0xCF);
        assert_eq!(C_LITO_EXIT_ROOM, 0xD0);

        assert_eq!(S_LITO_ENTER_CHANNEL_RESULT, 0xC9);
        assert_eq!(S_LITO_CHANNEL_INFO, 0xCA);
        assert_eq!(S_LITO_ROOM_INFO, 0xCC);
        assert_eq!(S_LITO_ROOM_USER, 0xCD);
        assert_eq!(S_LITO_JOIN_RESULT, 0xCE);
        assert_eq!(S_LITO_DELETE_ROOM, 0xCF);
        assert_eq!(S_LITO_EXIT_ROOM, 0xD0);
    }
}
