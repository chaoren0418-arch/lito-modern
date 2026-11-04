//! GAME bootstrap packets and the server-owned eight-row character directory.
use super::{
    native_city_runtime::source_identity::{
        self as native_source_identity, fields::Fields, Record,
    },
    native_login::decode_text,
    native_opcodes::{
        C_LITO_SELECT_CHAR, C_LOGIN_RESULT, C_READ_NOTICE, S_LITO_CHARACTER_INFO, S_LITO_NEWS,
        S_LITO_SELECT_CHAR_RESULT, S_NEWS, S_NOTICE, S_NUM_CHARACTER,
    },
};
use anyhow::{ensure, Context, Result};
use zeroize::Zeroizing;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    ClassicEntry(ClassicEntryCheck),
    Notice(u16),
    Ready,
    News(String),
    Count(u8),
    Row(Record),
    Selected(u8),
    Other(u8),
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct ClassicEntryCheck {pub accepted:bool,pub value:u32}
pub fn classic_entry_request(name:&str)->Result<Zeroizing<Vec<u8>>>{
    let mut body=Zeroizing::new(vec![5]);body.extend_from_slice(&super::native_login::encode_text(name,27)?);body.push(0);Ok(body)
}
pub fn parse(body: &[u8]) -> Result<Packet> {
    let opcode = *body.first().context("empty GAME packet")?;
    Ok(match opcode {
        7=>{
            ensure!(body.len()==6&&matches!(body[1],0|3),"classic entry check envelope/domain");
            Packet::ClassicEntry(ClassicEntryCheck{accepted:body[1]==3,value:u32::from_le_bytes(body[2..6].try_into()?)})
        }
        S_NOTICE => {
            ensure!(body.len() == 3, "native notice envelope");
            let id = u16::from_le_bytes([body[1], body[2]]);
            ensure!(id <= 32767, "native notice id");
            Packet::Notice(id)
        }
        S_LITO_NEWS => {
            ensure!(body.len() == 1, "native readiness envelope");
            Packet::Ready
        }
        S_NEWS => {
            ensure!(
                (3..=257).contains(&body.len()) && body.last() == Some(&0),
                "native common news envelope"
            );
            Packet::News(super::native_login::decode_message(
                &body[1..body.len() - 1],
                true,
            )?)
        }
        S_NUM_CHARACTER | S_LITO_CHARACTER_INFO => match native_source_identity::decode(body)? {
            native_source_identity::Packet::Start(n) => Packet::Count(n),
            native_source_identity::Packet::Record(row) => {
                row.fields().validate_java_projection()?;
                decode_text(row.name())?;
                if !row.clan_name().is_empty() {
                    decode_text(row.clan_name())?;
                }
                Packet::Row(row)
            }
        },
        S_LITO_SELECT_CHAR_RESULT => {
            ensure!(
                body.len() == 2 && [0, 1, 7, 8].contains(&body[1]),
                "native character selection result"
            );
            Packet::Selected(body[1])
        }
        _ => Packet::Other(opcode),
    })
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Character {
    pub name: String,
    pub clan: String,
    pub fields: Fields,
}
#[derive(Default)]
pub struct Directory {
    expected: Option<u8>,
    rows: Vec<Record>,
}
impl Directory {
    pub fn start(&mut self, count: u8) -> Result<()> {
        ensure!(
            self.expected.is_none() && count <= 8,
            "duplicate/out-of-bound character count"
        );
        self.expected = Some(count);
        Ok(())
    }
    pub fn push(&mut self, row: Record) -> Result<()> {
        let count = self.expected.context("character row before count")?;
        ensure!(self.rows.len() < usize::from(count), "extra character row");
        ensure!(
            !self.rows.iter().any(|old| old.name() == row.name()),
            "duplicate character name"
        );
        self.rows.push(row);
        Ok(())
    }
    pub fn complete(&self) -> bool {
        self.expected
            .is_some_and(|n| usize::from(n) == self.rows.len())
    }
    pub fn view(&self) -> Result<Vec<Character>> {
        ensure!(self.complete(), "incomplete character directory");
        self.rows
            .iter()
            .map(|r| {
                Ok(Character {
                    name: decode_text(r.name())?,
                    clan: if r.clan_name().is_empty() {
                        String::new()
                    } else {
                        decode_text(r.clan_name())?
                    },
                    fields: r.fields().clone(),
                })
            })
            .collect()
    }
    pub fn select(&self, index: usize) -> Result<Zeroizing<Vec<u8>>> {
        ensure!(self.complete(), "character directory not ready");
        let row = self.rows.get(index).context("character selection index")?;
        let mut body = Zeroizing::new(vec![C_LITO_SELECT_CHAR]);
        body.extend_from_slice(row.name());
        body.push(0);
        Ok(body)
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Notice {
    pub message_id: Option<u16>,
    pub text: Option<String>,
}
impl Notice {
    pub fn acknowledgement(&self) -> Option<u8> {
        match self.message_id {
            Some(438) => Some(C_LOGIN_RESULT),
            Some(_) => Some(C_READ_NOTICE),
            // S_NEWS is an announcement. C_33 requests another copy; it is not an ACK.
            None => None,
        }
    }
}
