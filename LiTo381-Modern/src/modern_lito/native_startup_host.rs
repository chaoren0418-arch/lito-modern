//! Separate startup-host input for explicitly synthetic Native-path tests.
//! 64B7E4 -> private050104CD -> 64AD74 is not runtime-qualified. This
//! adapter declares its own provenance; neither a peer nor a selected alias
//! can supply it. Native hostnames/DNS and private initialization stay OPEN.
use anyhow::{ensure,Result};
use serde::Deserialize;
use std::net::{Ipv4Addr,SocketAddrV4};
#[path="native_host_buffers.rs"]pub mod bytes;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureInput {kind:String,host:String}
#[derive(Clone)]
pub struct FixtureStartupHost {host:bytes::HostBuffer}
impl FixtureStartupHost {
 pub fn load(path:&std::path::Path)->Result<Self>{
  use std::io::Read;ensure!(path.is_absolute(),"absolute startup input required");let mut raw=Vec::new();std::fs::File::open(path)?.take(513).read_to_end(&mut raw)?;Self::parse(&raw)
 }
 pub fn parse(bytes:&[u8])->Result<Self>{
  ensure!(bytes.len()<=512,"startup input exceeds bound");let value:FixtureInput=serde_json::from_slice(bytes)?;
  ensure!(value.kind=="modern_synthetic_startup_host_v1","explicit synthetic startup provenance required");
  let ip:Ipv4Addr=value.host.parse()?;
  ensure!(ip.is_loopback()&&ip.to_string()==value.host,"canonical numeric loopback startup host required");
  let mut host=bytes::HostBuffer::default();host.copy_payload(value.host.as_bytes())?;
  Ok(Self{host})
 }
 /// Synthetic input data only; not proof that private050104CD returned.
 pub fn raw_host(&self)->&bytes::HostBuffer{&self.host}
 pub(crate) fn target(&self)->SocketAddrV4{let host=std::str::from_utf8(self.host.payload().expect("validated fixture host")).expect("validated numeric ASCII host");SocketAddrV4::new(host.parse().expect("validated loopback host"),2000)}
}
impl super::Runtime {
 pub fn configure_source_startup_host_fixture(&mut self,host:FixtureStartupHost)->Result<()> {
  ensure!(self.synthetic_player_fixture_configured()&&self.source_connection_epoch().is_none()&&matches!(self.page,super::UiPage::Booting|super::UiPage::Menu)&&self.startup_host.is_none(),"startup authority must precede owned connection and is immutable");
  self.startup_host=Some(host);Ok(())
 }
 pub fn source_startup_host_fixture_target(&self)->Option<SocketAddrV4>{self.startup_host.as_ref().map(FixtureStartupHost::target)}
}
