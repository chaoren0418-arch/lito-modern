from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
s=s.replace('#[path="native_entry_transfer.rs"]pub mod entry_transfer;','#[path="native_entry_transfer.rs"]pub mod entry_transfer;\n#[path="native_startup_host.rs"]pub mod startup_host;')
s=s.replace('#[derive(Clone)]pub struct SourceChannelControl', '#[derive(Clone)]pub struct SourceChannelBack {epoch:SourceEpoch,revision:u64}\n#[derive(Clone)]pub struct SourceChannelControl')
s=s.replace('pub struct Runtime {entry_transfer:', 'pub struct Runtime {startup_host:Option<startup_host::FixtureStartupHost>,entry_transfer:')
s=s.replace('Ok(Self{entry_transfer:', 'Ok(Self{startup_host:None,entry_transfer:')
assert 'transfer:self.entry_transfer.key.clone(),' in s
s=s.replace('transfer:self.entry_transfer.key.clone(),','transfer:self.entry_transfer.key.clone(),startup_host_fixture:self.source_startup_host_fixture_target(),')
s=s.replace('Action::Close=>{if matches!(self.page,UiPage::Closing|UiPage::Closed){return Ok(());}', 'Action::Close=>{if matches!(self.page,UiPage::Closing|UiPage::Closed){return Ok(());}self.startup_host=None;')
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8').replace('SourceLobbyBack,SourceChannelControl','SourceLobbyBack,SourceChannelBack,SourceChannelControl')
s=s.replace('struct State {channel_controls:', 'struct State {channel_back:RefCell<Option<SourceChannelBack>>,channel_controls:')
s=s.replace('State{channel_controls:', 'State{channel_back:RefCell::new(None),channel_controls:')
s=s.replace(' *s.lobby_back.borrow_mut()=', ' *s.channel_back.borrow_mut()=s.runtime.borrow().as_ref().and_then(Runtime::source_channel_back);\n *s.lobby_back.borrow_mut()=')
s=s.replace('BACK=>s.lobby_back.borrow().is_some()||', 'BACK=>s.channel_back.borrow().is_some()||s.lobby_back.borrow().is_some()||')
old='let result=s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::BackChannel);'
assert old in s
s=s.replace(old,'let token=s.channel_back.borrow().clone();let result=token.as_ref().ok_or_else(||anyhow::anyhow!("返回地址尚未可用")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::BackChannel(c)));')
p.write_text(s,encoding='utf-8')
