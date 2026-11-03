//! Room/world identities are not database character credentials. A bot may use
//! the same numeric pair as a human without becoming that selected character.
use super::application_protocol::{self as app, CharacterKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum MemberKind {
    Human = 0,
    ServerBot = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticipantId {
    pub kind: MemberKind,
    pub source_id: u32,
    pub id: u64,
}
impl ParticipantId {
    pub fn new(kind: MemberKind, key: CharacterKey) -> Self {
        Self {
            kind,
            source_id: key.source_id,
            id: key.object_id,
        }
    }
    pub fn valid(self) -> bool {
        app::valid_source(self.source_id) && self.id > 0 && self.id <= i64::MAX as u64
    }
    pub fn character(self) -> Option<CharacterKey> {
        (self.kind == MemberKind::Human && self.valid()).then_some(CharacterKey {
            source_id: self.source_id,
            object_id: self.id,
        })
    }
    pub fn matches_character(self, key: CharacterKey) -> bool {
        self.character() == Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kind_is_part_of_identity_not_a_display_label_or_database_key() {
        let key = CharacterKey {
            source_id: 7,
            object_id: 3333,
        };
        let human = ParticipantId::new(MemberKind::Human, key);
        let bot = ParticipantId::new(MemberKind::ServerBot, key);
        assert_ne!(human, bot);
        assert!(human < bot);
        assert!(human.matches_character(key));
        assert!(!bot.matches_character(key));
        assert_eq!(bot.character(), None);
        assert_eq!(std::collections::HashSet::from([human, bot]).len(), 2);
    }
}
