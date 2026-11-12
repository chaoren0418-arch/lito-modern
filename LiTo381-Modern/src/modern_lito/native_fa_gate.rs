//! Native chat-command gate on the own actor's +0x13a flag (dispatcher
//! 403F71..4042B0). The flag comes from FA `active` (4AC234) or from the
//! S_PUT_OBJECT status-4 tail dword. The modern client has no slash-command
//! input, so nothing calls this yet; no command packet is built here.
use super::native_world::World;

/// The five command words whose dispatcher branch reads own actor+0x13a.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Command {
    Title,
    Emblem,
    Ban,
    Surrender,
    Cease,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Gate {
    /// The flag is set: the original continues into the command body.
    Run,
    /// The flag is clear: the original prints this string-table entry instead.
    Notice(u16),
    /// The flag is clear: the original returns without any output.
    Ignore,
}
/// String-table index printed by the refused `title`/`emblem`/`ban` branches
/// (403F94, 40404F, 4042B1: table 834B94, offset 0x3F4).
pub const REFUSED_NOTICE: u16 = 253;

impl Command {
    pub const ALL: [Self; 5] = [
        Self::Title,
        Self::Emblem,
        Self::Ban,
        Self::Surrender,
        Self::Cease,
    ];
    /// Built-in ASCII word compared by the dispatcher.
    pub fn word(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Emblem => "emblem",
            Self::Ban => "ban",
            Self::Surrender => "surrender",
            Self::Cease => "cease",
        }
    }
    /// Index of the localized alias in string table 834BC4 that the dispatcher
    /// accepts next to the ASCII word.
    pub fn alias_index(self) -> u16 {
        match self {
            Self::Title => 364,
            Self::Emblem => 379,
            Self::Surrender => 381,
            Self::Cease => 382,
            Self::Ban => 384,
        }
    }
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.word() == word)
    }
}
/// Native own actor+0x13a; false without an own actor.
pub fn own_flag(world: &World) -> bool {
    world.own_actor().is_some_and(|a| a.clan_marker_active)
}
pub fn gate(own_flag: bool, command: Command) -> Gate {
    match (own_flag, command) {
        (true, _) => Gate::Run,
        (false, Command::Title | Command::Emblem | Command::Ban) => Gate::Notice(REFUSED_NOTICE),
        (false, Command::Surrender | Command::Cease) => Gate::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet(key: &str) -> Vec<u8> {
        let f: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-world-current.json"
        ))
        .unwrap();
        let h = f["packets"][key].as_str().unwrap();
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn refused_commands_split_into_notice_and_silent() {
        for c in Command::ALL {
            assert_eq!(gate(true, c), Gate::Run);
            assert_eq!(Command::from_word(c.word()), Some(c));
        }
        assert_eq!(gate(false, Command::Title), Gate::Notice(253));
        assert_eq!(gate(false, Command::Emblem), Gate::Notice(253));
        assert_eq!(gate(false, Command::Ban), Gate::Notice(253));
        assert_eq!(gate(false, Command::Surrender), Gate::Ignore);
        assert_eq!(gate(false, Command::Cease), Gate::Ignore);
        // `war` is dispatched without reading the flag.
        assert_eq!(Command::from_word("war"), None);
    }
    #[test]
    fn put_object_status_tail_dword_and_fa_drive_the_own_flag() {
        let mut own = packet("actor");
        let end = own.len();
        assert_eq!(own[end - 5..], [0xff, 0, 0, 0, 0]);
        let a = super::super::native_world::actor(&own).unwrap();
        assert!(a.status & 4 != 0 && !a.clan_marker_active);
        // 4A3A72: any nonzero dword sets +0x13a; the category byte does not.
        own[end - 5] = 3;
        assert!(!super::super::native_world::actor(&own).unwrap().clan_marker_active);
        own[end - 2] = 1;
        let a = super::super::native_world::actor(&own).unwrap();
        assert!(a.clan_marker_active);
        assert_eq!(a.status_tail, Some([3, 0, 0, 1, 0]));
        // A body without status bit 4 carries no tail and starts clear.
        let bot = super::super::native_world::actor(&packet("bot")).unwrap();
        assert!(bot.status & 4 == 0 && !bot.clan_marker_active);

        let mut w = World::default();
        for key in ["world", "start", "status", "actor", "bot"] {
            w.accept(&packet(key)).unwrap();
        }
        let id = w.own_actor().unwrap().id;
        assert!(!own_flag(&w));
        // FA: object D, emblem D, text S, active D (4AC1C0).
        let fa = |active: u32| {
            let mut b = vec![0xfa];
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes());
            b.extend_from_slice(b"Raiders\0");
            b.extend_from_slice(&active.to_le_bytes());
            b
        };
        w.accept(&fa(1)).unwrap();
        assert!(own_flag(&w));
        assert_eq!(gate(own_flag(&w), Command::Title), Gate::Run);
        w.accept(&fa(0)).unwrap();
        assert!(!own_flag(&w));
        // A new S_PUT_OBJECT takes the flag from its own tail dword.
        w.accept(&fa(1)).unwrap();
        w.accept(&packet("actor")).unwrap();
        assert!(!own_flag(&w));
        w.accept(&own).unwrap();
        assert!(own_flag(&w));
    }
}
