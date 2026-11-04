//! Native Tournament channel replies and room rows. No Modern v47 envelopes.
use super::{
    native_login::decode_text,
    native_opcodes::{
        C_LITO_ENTER_CHANNEL, S_LITO_CHANNEL_INFO, S_LITO_ENTER_CHANNEL_RESULT, S_LITO_ROOM_INFO,
    },
};
use anyhow::{ensure, Context, Result};

pub const CHANNEL_NAMES: [&str; 4] = ["DeathKnight", "Baphomet", "Succubus", "Ogre"];
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Channel {
    pub id: u8,
    pub users: u16,
    pub capacity: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub attributes: u8,
    pub users: u16,
    pub capacity: u16,
    pub reserved: u16,
    pub option: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Enter {
        result: u8,
        channel: u8,
        words: Option<(u32, u16, u32)>,
    },
    Channel(Channel),
    Room(Room),
    SourceRefresh,
    Other,
}
fn active(id: u8) -> bool {
    id <= 3 || id == 99
}
pub fn parse(b: &[u8]) -> Result<Packet> {
    let op = *b.first().context("empty lobby packet")?;
    Ok(match op {
        S_LITO_ENTER_CHANNEL_RESULT => {
            ensure!(b.len() >= 3 && active(b[2]), "channel result envelope");
            let words = match b[1] {
                0 => {
                    ensure!(b.len() == 13, "channel success tail");
                    Some((
                        u32::from_le_bytes(b[3..7].try_into()?),
                        u16::from_le_bytes(b[7..9].try_into()?),
                        u32::from_le_bytes(b[9..13].try_into()?),
                    ))
                }
                1 | 10 => {
                    ensure!(
                        b.len() == 3 && (b[1] != 10 || b[2] <= 3),
                        "channel failure envelope"
                    );
                    None
                }
                _ => anyhow::bail!("unsupported channel result"),
            };
            Packet::Enter {
                result: b[1],
                channel: b[2],
                words,
            }
        }
        S_LITO_CHANNEL_INFO => {
            ensure!(b.len() == 6 && active(b[1]), "channel population envelope");
            Packet::Channel(Channel {
                id: b[1],
                users: u16::from_le_bytes([b[2], b[3]]),
                capacity: u16::from_le_bytes([b[4], b[5]]),
            })
        }
        S_LITO_ROOM_INFO => {
            ensure!(b.len() >= 2, "room subtype missing");
            if b[1] != 0 {
                return Ok(Packet::Other);
            }
            ensure!((16..=104).contains(&b.len()), "room row bound");
            let end = 6 + b[6..]
                .iter()
                .position(|v| *v == 0)
                .context("room name missing NUL")?;
            ensure!(end <= 95 && b.len() == end + 9, "room row tail");
            let t = &b[end + 1..];
            Packet::Room(Room {
                id: u32::from_le_bytes(b[2..6].try_into()?),
                name: decode_text(&b[6..end])?,
                attributes: t[0],
                users: u16::from_le_bytes([t[1], t[2]]),
                capacity: u16::from_le_bytes([t[3], t[4]]),
                reserved: u16::from_le_bytes([t[5], t[6]]),
                option: t[7],
            })
        }
        0xf5 => {
            ensure!(b.len() == 1, "source refresh envelope");
            Packet::SourceRefresh
        }
        _ => Packet::Other,
    })
}
pub fn enter(id: u8) -> Result<Vec<u8>> {
    ensure!(id <= 3, "ordinary channel index");
    Ok(vec![C_LITO_ENTER_CHANNEL, id])
}
pub fn upsert(rows: &mut Vec<Room>, room: Room) -> Result<()> {
    if let Some(old) = rows.iter_mut().find(|r| r.id == room.id) {
        *old = room;
    } else {
        ensure!(rows.len() < 512, "room directory exceeds 512 rows");
        rows.push(room);
    }
    Ok(())
}
