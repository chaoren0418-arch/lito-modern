//! One NPC action authorizes one page-scoped S_F3/S_F4 continuation.
use super::{native_merchant::Owner, native_services::{Availability, Directory, VsEndpoint}};
use anyhow::{ensure, Result};
use std::net::SocketAddr;

struct SameEndpointServices {
    owner: Owner,
    character: String,
    endpoint: SocketAddr,
    directory: Directory,
}

pub struct Pending {
    pub owner: Owner,
    pub character: String,
    pub index: u8,
    availability: Option<Availability>,
    pub destination: Option<SocketAddr>,
    same_endpoint_services: Option<SameEndpointServices>,
}
impl Pending {
    pub fn new(owner: Owner, character: String, html: &str, action: &str) -> Option<Self> {
        // Only these two choices are reachable on the current formal page.
        let index = match (html, action) {
            ("TeleportMe", "LT_TELEPORT_B") => 1,
            ("TeleportMe", "LT_TELEPORT_C") => 2,
            _ => return None,
        };
        Some(Self { owner, character, index, availability: None, destination: None, same_endpoint_services: None })
    }
    pub fn status(&mut self, owner: &Owner, status: Availability) -> Result<()> {
        ensure!(&self.owner == owner && self.availability.is_none() && self.destination.is_none(),
            "unsolicited or duplicate teleport status");
        ensure!(status.available(self.index), "selected teleport destination unavailable");
        self.availability = Some(status);
        Ok(())
    }
    pub fn endpoint(&mut self, owner: &Owner, reply: VsEndpoint, _current: SocketAddr) -> Result<SocketAddr> {
        ensure!(&self.owner == owner && self.destination.is_none(), "stale or duplicate teleport endpoint");
        ensure!(reply.index == self.index && self.availability.as_ref().is_some_and(|s|s.available(reply.index)),
            "teleport endpoint does not match the selected available slot");
        // Equality of peer addresses does not preserve the old session. The
        // controller writes C_D9, retires that epoch and reconnects with a fresh
        // handshake for every authorized endpoint, including the same address.
        self.destination = Some(reply.endpoint);
        Ok(reply.endpoint)
    }

    /// S_FB is advertised at LOGIN, not repeated by a same-endpoint GAME.
    /// Keep only routes to that exact peer. Availability, selected set, room
    /// membership and old world/input ownership never enter this catalog.
    pub fn remember_same_endpoint_services(&mut self, current: SocketAddr, directory: Option<&Directory>) {
        self.same_endpoint_services = None;
        if self.destination != Some(current) { return; }
        let Some(directory) = directory else { return; };
        let rooms: Vec<_> = directory.rooms.iter().filter(|room|room.endpoint==current).cloned().collect();
        if rooms.is_empty() { return; }
        self.same_endpoint_services = Some(SameEndpointServices {
            owner: self.owner.clone(), character: self.character.clone(), endpoint: current,
            directory: Directory { rooms, servers: Vec::new() },
        });
    }

    /// Called only after fresh GAME accepts the expected character. A rejected
    /// binding consumes this one-shot catalog too, so it cannot revive later.
    pub fn take_same_endpoint_services(&mut self, peer: SocketAddr, connection: u64, character: &str) -> Option<Directory> {
        let saved = self.same_endpoint_services.take()?;
        (saved.endpoint==peer && self.destination==Some(peer)
            && saved.owner==self.owner && saved.character==self.character
            && saved.character==character && saved.owner.connection.checked_add(1)==Some(connection))
            .then_some(saved.directory)
    }
}
