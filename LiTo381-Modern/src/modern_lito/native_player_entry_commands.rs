//! Shared Window callback and headless candidate command path; no protocol controller.
use super::{
    login_form, Action, Runtime, SourceCharacterBack, SourceCharacterControl, SourceCharacterSubmit,
};
use anyhow::Result;
pub enum Command<'a> {
    ChooseSource(&'a super::login_bootstrap::SourceControl),
    ContinueNews(&'a super::login_bootstrap::NewsControl),
    OpenLogin,
    EditLogin {
        control: &'a login_form::Control,
        account: &'a [u8],
        password: &'a [u8],
    },
    SubmitLogin(&'a login_form::Control),
    CancelLogin(&'a login_form::Control),
    ChooseCharacter {
        control: &'a SourceCharacterControl,
        double_click: bool,
    },
    SubmitCharacter(&'a SourceCharacterSubmit),
    BackCharacter(&'a SourceCharacterBack),
    BackLobby(&'a super::SourceLobbyBack),
    ChooseChannel(&'a super::SourceChannelControl),
    ChooseTransferServer(&'a super::SourceTransferServerControl),
    BackChannel(&'a super::SourceChannelBack),
    BackRoom,
    ConfirmJavaReturn { confirmation: &'a super::java_return::Confirmation, accepted: bool },
    Close,
}
impl Runtime {
    pub fn handle_player_entry_command(&mut self, command: Command<'_>) -> Result<()> {
        match command {
            Command::ChooseSource(c) => self.handle_source_choice(c),
            Command::ContinueNews(c) => self.handle_source_news_continue(c),
            Command::ConfirmJavaReturn{confirmation,accepted}=>self.resolve_java_return_confirmation(confirmation,accepted),
            Command::OpenLogin => {
                if self.java_return.key.phase==super::java_return::Phase::Retained{return self.resume_java_return_fixture();}
                anyhow::ensure!(self.login_bootstrap.key.stage==super::login_bootstrap::Stage::Idle,"source bootstrap already active; disconnect or Close required");
                self.begin_player_fixture_if_needed()?;
                self.open_source_login_form()
            }
            Command::EditLogin {
                control,
                account,
                password,
            } => self.edit_source_login_form(control, account, password),
            Command::SubmitLogin(c) => {
                self.stage_source_login_form(c)?;
                self.request_player_fixture_submit();
                Ok(())
            }
            Command::CancelLogin(c) => {
                self.cancel_source_login_form(c)?;
                self.login_events.submit = None;
                Ok(())
            }
            Command::ChooseCharacter {
                control,
                double_click,
            } => self.choose_source_character_control(control, double_click),
            Command::SubmitCharacter(c) => self.handle_source_character_submit(c),
            Command::BackLobby(c) => self.handle_source_lobby_back(c),
            Command::ChooseChannel(c) => self.handle_source_channel_control(c),
            Command::ChooseTransferServer(c) => self.handle_source_transfer_server(c),
            Command::BackChannel(c) => self.handle_source_channel_back(c),
            Command::BackRoom => anyhow::bail!("房間返回大廳尚未可用：房間場景来源未接線"),
            Command::BackCharacter(c) => {
                self.handle_source_character_back(c)?;
                self.request_player_fixture_reopen();
                Ok(())
            }
            Command::Close => self.handle(Action::Close),
        }
    }
}
