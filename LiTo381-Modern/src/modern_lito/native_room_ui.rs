//! Room form and member intents retain the identity shown by the native window.
use super::{native_login::{LoginPhase, Snapshot}, native_room::{Member, Room, Settings}};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Configuration {
    id: u32,
    owner: String,
    name: String,
    attributes: u8,
    capacity: u16,
    options: [u8; 4],
    selections: Vec<u8>,
}
impl From<&Room> for Configuration {
    fn from(r: &Room) -> Self {
        Self { id:r.id, owner:r.owner.clone(), name:r.name.clone(), attributes:r.attributes,
            capacity:r.capacity, options:r.options, selections:r.selections.clone() }
    }
}
#[derive(Clone, Debug)]
pub struct Draft {
    connection: u64,
    character: String,
    channel: u8,
    original: Option<Configuration>,
}
impl Draft {
    pub fn capture(connection:u64, s:&Snapshot, edit:bool)->Result<Self> {
        let character=s.selected_character.clone().context("尚未選擇角色")?;
        let channel=s.channel.as_ref().context("尚未進入頻道")?.id;
        let original=if edit {
            ensure!(s.phase==LoginPhase::RoomReady,"目前不能修改房間");
            let r=s.joined_room.as_ref().context("目前沒有房間")?;
            ensure!(r.owner==character,"只有房主可以修改設定");
            Some(Configuration::from(r))
        } else {
            ensure!(s.phase==LoginPhase::LobbyReady && s.joined_room.is_none(),"請先返回大廳建立房間");
            None
        };
        Ok(Self {connection,character,channel,original})
    }
    pub fn editing(&self)->bool {self.original.is_some()}
    pub fn name(&self)->&str {self.original.as_ref().map_or("新房間",|r|r.name.as_str())}
    pub fn capacity(&self)->u16 {self.original.as_ref().map_or(8,|r|r.capacity)}
    pub fn protected(&self)->bool {self.original.as_ref().is_some_and(|r|r.attributes&4!=0)}
    pub fn validate(&self,connection:u64,s:&Snapshot)->Result<()> {
        ensure!(connection==self.connection && s.selected_character.as_deref()==Some(self.character.as_str())
            && s.channel.as_ref().is_some_and(|c|c.id==self.channel),"房間表單已失效，請重新開啟");
        match &self.original {
            Some(original)=>{
                ensure!(s.phase==LoginPhase::RoomReady,"目前不能送出設定");
                let r=s.joined_room.as_ref().context("房間已離開")?;
                ensure!(r.owner==self.character && Configuration::from(r)==*original,"房主或房間設定已更新，請重新開啟表單");
            }
            None=>ensure!(s.phase==LoginPhase::LobbyReady && s.joined_room.is_none(),"已離開建立房間的大廳"),
        }
        Ok(())
    }
    pub fn settings<'a>(&'a self,connection:u64,s:&Snapshot,name:&'a str,password:&'a str,
        protected:bool,capacity:u16)->Result<Settings<'a>> {
        self.validate(connection,s)?;
        ensure!((2..=100).contains(&capacity),"房間人數須介於2至100人");
        ensure!(protected==!password.is_empty(),"啟用密碼時請輸入新密碼；取消密碼時請清空輸入");
        let attributes=self.original.as_ref().map_or(0,|r|r.attributes)&!4 | if protected {4}else{0};
        let value=Settings {name,password,attributes,capacity,
            options:self.original.as_ref().map(|r|(r.options,r.selections.as_slice()))};
        value.request(if self.editing(){0xdd}else{0xcc})?;
        Ok(value)
    }
}

/// The lobby row may move or change user count while the same room remains.
/// A row index alone cannot retain the displayed join intent across updates.
#[derive(Clone, Debug)]
pub struct RoomChoice {
    connection:u64,
    character:String,
    channel:u8,
    room:u32,
}
impl RoomChoice {
    pub fn capture(connection:u64,s:&Snapshot,index:usize)->Result<Self> {
        ensure!(matches!(s.phase,LoginPhase::LobbyReady|LoginPhase::LoadingRooms) && s.joined_room.is_none(),"請等待大廳房間清單");
        Ok(Self {connection,character:s.selected_character.clone().context("尚未選擇角色")?,
            channel:s.channel.as_ref().context("尚未進入頻道")?.id,
            room:s.rooms.get(index).context("請選擇房間")?.id})
    }
    pub fn index(&self,connection:u64,s:&Snapshot)->Result<usize> {
        ensure!(connection==self.connection && s.selected_character.as_deref()==Some(self.character.as_str())
            && s.channel.as_ref().is_some_and(|c|c.id==self.channel)
            && matches!(s.phase,LoginPhase::LobbyReady|LoginPhase::LoadingRooms) && s.joined_room.is_none(),
            "房間選取已失效，請重新選擇");
        s.rooms.iter().position(|r|r.id==self.room).context("房間已從清單移除")
    }
    pub fn join_index(&self,connection:u64,s:&Snapshot)->Result<usize> {
        ensure!(s.phase==LoginPhase::LobbyReady,"請等待大廳房間清單更新");self.index(connection,s)
    }
}

#[derive(Clone, Debug)]
pub struct MemberChoice {
    connection:u64,
    room:u32,
    owner:String,
    member:Member,
}
impl MemberChoice {
    pub fn capture(connection:u64,s:&Snapshot,index:usize)->Result<Self> {
        ensure!(s.phase==LoginPhase::RoomReady,"請等待房間更新");
        let r=s.joined_room.as_ref().context("目前没有房間")?;
        ensure!(s.selected_character.as_deref()==Some(r.owner.as_str()),"只有房主可以移出成員");
        let member=r.members.get(index).context("請選擇成員")?;
        ensure!(member.name!=r.owner,"不能移出房主");
        Ok(Self {connection,room:r.id,owner:r.owner.clone(),member:member.clone()})
    }
    pub fn resolve<'a>(&'a self,connection:u64,s:&Snapshot)->Result<&'a str> {
        ensure!(connection==self.connection && s.phase==LoginPhase::RoomReady,"成員選擇已失效");
        let r=s.joined_room.as_ref().context("房間已離開")?;
        ensure!(r.id==self.room && r.owner==self.owner && s.selected_character.as_deref()==Some(self.owner.as_str())
            && r.members.iter().any(|m|m==&self.member),"成員或房主已變更，請重新選擇");
        Ok(&self.member.name)
    }
}
pub fn room_page(s:&Snapshot)->bool {
    matches!(s.phase,LoginPhase::LobbyReady|LoginPhase::LoadingRooms|LoginPhase::RoomCreating
        |LoginPhase::RoomJoining|LoginPhase::RoomAwaitSelf|LoginPhase::RoomReady|LoginPhase::ReadyPending
        |LoginPhase::ForcePending|LoginPhase::ModePending|LoginPhase::MapPending|LoginPhase::RoomEditing|LoginPhase::KickPending
        |LoginPhase::RoomLeaving) || (s.phase==LoginPhase::ObserverPending && s.channel.is_some())
}

pub fn status(s: &Snapshot) -> String {
    let Some(room) = &s.joined_room else { return String::new(); };
    let mode = ["寵物競賽", "死鬥", "任務戰", "攻城戰", "限時挑戰", "榮譽戰", "租裝村"]
        .get(usize::from(room.mode)).copied().unwrap_or("未知");
    let countdown = room.countdown.filter(|value| *value > 0)
        .map(|value| format!("　倒數 {value}"))
        .unwrap_or_default();
    let mut text = format!("{}　房主：{}\r\n模式 {}　地圖 {}　已收到 {} 位成員{}",
        room.name, room.owner, mode, room.map_id, room.members.len(), countdown);
    if let Some(message) = &s.room_system_message {
        text.push_str("\r\n");
        text.push_str(message);
    }
    text
}
