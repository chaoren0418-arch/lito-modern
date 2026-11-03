from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
s=s.replace('navigation:self.entry_navigation.key.clone(),epoch:', 'navigation:self.entry_navigation.key.clone(),transfer:self.entry_transfer.key.clone(),epoch:')
s=s.replace('let packet=match epoch.decoded_packet(&bytes)', 'let packet=match if bytes.first()==Some(&203)&&self.entry_transfer.roster_alternate(){epoch.decoded_transfer_roster_trigger(&bytes)}else{epoch.decoded_packet(&bytes)}')
s=s.replace('matches!(bytes.first(),Some(243|244))','matches!(bytes.first(),Some(243|244|251))')
s=s.replace('else if matches!(bytes.first(),Some(3|203)){self.consume_source_identity_event(&packet).map(StreamIngress::Roster)}','else if matches!(bytes.first(),Some(3|203)){if self.entry_transfer.roster_alternate(){self.consume_source_transfer_roster(&packet).map(StreamIngress::Transfer)}else{self.consume_source_identity_event(&packet).map(StreamIngress::Roster)}}')
start='Ok(StreamIngress::Transfer(change@entry_transfer::Change::RestartAfterUnqualifiedAddress{..}))=>{'
a=s.index(start);b=s.index('Ok(StreamIngress::Actor(',a);s=s[:a]+s[b:]
s=s.replace('}}Ok(progress)\n }','}}if matches!(progress,source_send::Progress::Complete){self.finish_source_navigation_write()?;}Ok(progress)\n }')
assert 'self.finish_source_navigation_write()?;' in s
s=s.replace('pub fn poll(&mut self)->bool {let mut changed=self.poll_player_fixture_events();','pub fn poll(&mut self)->bool {let mut changed=self.poll_source_transfer_timer();changed|=self.poll_player_fixture_events();')
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_presentation.rs';s=p.read_text(encoding='utf-8').replace('pub navigation:super::entry_navigation::Key,','pub navigation:super::entry_navigation::Key,\n pub transfer:super::entry_transfer::Key,');p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_player_entry_commands.rs';s=p.read_text(encoding='utf-8').replace('    BackChannel,','    ChooseChannel(&\'a super::SourceChannelControl),\n    ChooseTransferServer(&\'a super::SourceTransferServerControl),\n    BackChannel,').replace('            Command::BackChannel =>','            Command::ChooseChannel(c) => self.handle_source_channel_control(c),\n            Command::ChooseTransferServer(c) => self.handle_source_transfer_server(c),\n            Command::BackChannel =>');p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_login_events.rs';s=p.read_text(encoding='utf-8');a=s.index('        let (tx, rx) = mpsc::sync_channel(1);',s.index('pub(crate) fn begin_player_fixture_if_needed'));s=s[:a]+'''        self.spawn_owned_player_fixture(profile,epoch)
    }
    fn spawn_owned_player_fixture(&mut self,profile:FixtureProfile,epoch:SourceEpoch)->Result<()> {
'''+s[a:]
a=s.index('    pub(crate) fn request_player_fixture_submit');s=s[:a]+'''    pub(crate) fn reconnect_source_transfer_fixture(&mut self,target:std::net::SocketAddrV4)->Result<()> {
        ensure!(target.ip().is_loopback()&&target.port()!=0&&self.login_events.active.is_some(),"owned loopback redirect required");
        let late=self.login_events.reap();self.stale_results_rejected+=late as u64;
        ensure!(self.login_events.retiring.len()<4,"fixture redirect retirement budget reached");
        let mut profile=self.login_events.profile.as_ref().context("configured fixture absent")?.clone();
        //Wire F4 supplies the target. Profile only supplies synthetic IO/environment.
        profile.endpoint=SocketAddr::V4(target);
        let mut transfer=self.entry_transfer.clone();transfer.reconnect_after_write=None;
        transfer.key.server_rows.clear();transfer.key.rows_revision=transfer.key.rows_revision.checked_add(1).context("row revision exhausted")?;
        let flow=self.source_flow.transfer_connected();let old_return=(self.entry_navigation.key.return_state,self.entry_navigation.key.return_mode);
        let epoch=self.begin_source_connection_candidate()?;
        self.entry_transfer=transfer;self.source_flow=flow;
        self.entry_navigation.key.scene=super::entry_navigation::Scene::AwaitTransferRoster;
        self.entry_navigation.key.return_state=old_return.0;self.entry_navigation.key.return_mode=old_return.1;
        self.spawn_owned_player_fixture(profile,epoch)?;
        self.status="頻道切換連線中，等待伺服器回覆…".into();Ok(())
    }
'''+s[a:]
s=s.replace('        let h = self.login_events.active.as_mut().unwrap();\n        if !h.inflight', '        self.drive_source_navigation_sequence()?;\n        let h = self.login_events.active.as_mut().unwrap();\n        if !h.inflight')
p.write_text(s,encoding='utf-8');print('R506 Runtime epoch/stream/IO/shared command paths connected')
