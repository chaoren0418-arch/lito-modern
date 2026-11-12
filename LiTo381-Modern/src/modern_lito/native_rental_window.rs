//! Child of the native login window: rental controls share the same controller.
use super::*;
use crate::modern_lito::{native_merchant::{Owner,Listing},native_services::DirectRoom,native_npc_document::{Service,Action}};
pub const SET:u16=127;pub const ENTER:u16=128;pub const TOGGLE:u16=129;
pub const TALK:u16=130;pub const BUY:u16=131;pub const SELL:u16=132;pub const ORDER:u16=133;
pub const FOLLOW:u16=134;pub const BACK:u16=135;pub const LINKS:u16=224;
pub const SETS:u16=217;pub const SERVICES:u16=218;pub const NEARBY:u16=219;pub const BODY:u16=220;pub const LIST:u16=221;pub const QUANTITY:u16=222;pub const MONEY:u16=223;pub const HELP:u16=317;
pub const IDS:[u16;18]=[SET,ENTER,TOGGLE,TALK,BUY,SELL,ORDER,SETS,SERVICES,NEARBY,BODY,LIST,QUANTITY,MONEY,HELP,FOLLOW,BACK,LINKS];
#[derive(Clone,Debug,PartialEq,Eq)]
struct Target {owner:Owner,id:u32,incarnation:u64,revision:u64,name:String}
impl Target{
    fn capture(c:&LoginController,id:u32)->Option<Self>{let owner=c.merchant_owner()?;let w=&c.snapshot().world;let own=w.own_actor()?;let a=w.actors.get(&id)?;
        if id==own.id||a.dead||!a.draw_state.visible(false)||own.x.abs_diff(a.x).max(own.y.abs_diff(a.y))>5{return None;}
        Some(Self{owner,id,incarnation:a.incarnation,revision:a.target_revision,name:a.name.clone()})}
    fn valid(&self,c:&LoginController)->bool{Self::capture(c,self.id).as_ref()==Some(self)}
}
struct Binding{target:Target,text:String,actions:Vec<Action>,images:Vec<u16>,trailing:Vec<(usize,u16)>,html:String,received_revision:u64,history:Vec<String>,arguments:Vec<String>}
pub struct View{
    pub open:bool,pub documents:Service,pub set_attempt:Option<u8>,entry:Option<(u64,Option<String>,Vec<DirectRoom>)>,targets:Vec<Target>,
    layout_key:Option<(bool,bool,bool,bool,bool)>,binding:Option<Binding>,list_key:Option<(Owner,u64)>,labels:Vec<String>,feedback:String,action_rows:Vec<Action>,message_floor:u64,code_floor:u64,observed_generation:u64,
}
impl View{pub fn new(root:std::path::PathBuf)->Result<Self>{Ok(Self{open:false,documents:Service::start(root)?,set_attempt:None,entry:None,targets:vec![],layout_key:None,binding:None,list_key:None,labels:vec![],feedback:String::new(),action_rows:vec![],message_floor:0,code_floor:0,observed_generation:0})}}
fn entry(s:&crate::modern_lito::native_login::Snapshot)->bool{
    matches!(s.phase,LoginPhase::CharacterSelected|LoginPhase::RentalSetPending|LoginPhase::RentalReady|LoginPhase::RentalArmPending|LoginPhase::RentalAwaitPermission)
        &&s.services.as_ref().is_some_and(|v|!v.rooms.is_empty())
}
fn rental(s:&crate::modern_lito::native_login::Snapshot)->bool{
    matches!(s.phase,LoginPhase::BattlePending|LoginPhase::RentalRestartPending)&&s.joined_room.as_ref().is_some_and(|r|r.mode==6)
}
fn hotel_room(s:&crate::modern_lito::native_login::Snapshot)->bool{
    rental(s)&&matches!(s.world.map_id,Some(501..=503))
}
impl Ui{
    pub(super) unsafe fn draw_rental_choice(&self,draw:&DRAWITEMSTRUCT){
        if draw.hwndItem!=self.control(LINKS){return;}
        let selected=draw.itemState.0&ODS_SELECTED.0!=0;
        let bg=GetSysColor(if selected{COLOR_HIGHLIGHT}else{COLOR_WINDOW});
        FillRect(draw.hDC,&draw.rcItem,GetSysColorBrush(if selected{COLOR_HIGHLIGHT}else{COLOR_WINDOW}));
        if self.closing||!self.rental.open||self.inventory_view||!rental(self.controller.snapshot()){return;}
        let Some(binding)=self.rental.binding.as_ref().filter(|b|b.target.valid(&self.controller)) else{return;};
        let Some(action)=self.rental.action_rows.get(draw.itemID as usize) else{return;};
        let dpi=GetDpiForWindow(self.parent).max(96) as i32;let px=|n:i32|n*dpi/96;
        // Original TeleportMe layout: `<a>label</a>   <img>` puts the icon after
        // the label on the same line, bound by the document, not by command alone.
        let image=binding.actions.iter().position(|a|a==action).and_then(|index|
            self.rental.documents.catalog().and_then(|c|c.images.as_ref())
                .and_then(|bank|bank.trailing_glyph(&binding.html,&binding.trailing,index,&action.command)));
        let label=if matches!(action.command.as_str(),"LT_TELEPORT_B"|"LT_TELEPORT_C"){action.label.split("(這是server名.").next().unwrap_or(&action.label).trim_end()}else{&action.label};
        let old_font=SelectObject(draw.hDC,self.font.into());let old_color=SetTextColor(draw.hDC,COLORREF(GetSysColor(if selected{COLOR_HIGHLIGHTTEXT}else{COLOR_WINDOWTEXT})));let old_mode=SetBkMode(draw.hDC,TRANSPARENT);
        let mut text:Vec<u16>=label.encode_utf16().collect();
        let mut text_rect=draw.rcItem;text_rect.left+=px(4);text_rect.right-=px(4);
        if let Some(image)=image{
            let extent=|s:&[u16]|{let mut size=SIZE::default();if GetTextExtentPoint32W(draw.hDC,s,&mut size).as_bool(){size.cx}else{0}};
            let space:Vec<u16>=" ".encode_utf16().collect();
            let row=crate::modern_lito::native_npc_images::trailing_row([draw.rcItem.left,draw.rcItem.top,draw.rcItem.right,draw.rcItem.bottom],
                px(4),extent(&text),extent(&space),(px(image.width as i32),px(image.height as i32)));
            text_rect=RECT{left:row.text[0],top:row.text[1],right:row.text[2],bottom:row.text[3]};
            if let Some([x,y])=row.image{
                let background=[bg as u8,(bg>>8) as u8,(bg>>16) as u8];
                let pixels:Vec<u8>=image.rgba.chunks_exact(4).flat_map(|p|{
                    let a=u32::from(p[3]);let c=|n:usize|((u32::from(p[n])*a+u32::from(background[n])*(255-a)+127)/255) as u8;
                    [c(2),c(1),c(0),255]
                }).collect();
                let info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>() as u32,biWidth:image.width as i32,biHeight:-(image.height as i32),biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,..Default::default()},..Default::default()};
                let previous=SetStretchBltMode(draw.hDC,COLORONCOLOR);
                StretchDIBits(draw.hDC,x,y,px(image.width as i32),px(image.height as i32),0,0,image.width as i32,image.height as i32,Some(pixels.as_ptr().cast()),&info,DIB_RGB_COLORS,SRCCOPY);
                SetStretchBltMode(draw.hDC,STRETCH_BLT_MODE(previous));
            }
        }
        DrawTextW(draw.hDC,&mut text,&mut text_rect,DT_SINGLELINE|DT_VCENTER|DT_NOPREFIX|DT_END_ELLIPSIS);
        SetBkMode(draw.hDC,BACKGROUND_MODE(old_mode as u32));SetTextColor(draw.hDC,old_color);SelectObject(draw.hDC,old_font);
        if draw.itemState.0&ODS_FOCUS.0!=0{let _=DrawFocusRect(draw.hDC,&draw.rcItem);}
    }
    pub(super) unsafe fn create_rental_controls(&mut self)->Result<()>{
        let instance=HINSTANCE(GetModuleHandleW(None)?.0);
        for(id,class,label,style) in [
            (SET,w!("BUTTON"),"確認配裝",0),(ENTER,w!("BUTTON"),"進入租裝村",0),(TOGGLE,w!("BUTTON"),"NPC／租裝",0),
            (TALK,w!("BUTTON"),"對話",0),(BUY,w!("BUTTON"),"租用道具",0),(SELL,w!("BUTTON"),"退還道具",0),(ORDER,w!("BUTTON"),"送出交易",0),
            (FOLLOW,w!("BUTTON"),"選擇此項",0),(BACK,w!("BUTTON"),"上一頁",0),
            (LINKS,w!("COMBOBOX"),"",CBS_DROPDOWNLIST as u32|CBS_OWNERDRAWFIXED as u32|CBS_HASSTRINGS as u32|WS_VSCROLL.0),
            (SETS,w!("COMBOBOX"),"",CBS_DROPDOWNLIST as u32|WS_VSCROLL.0),
            (SERVICES,w!("COMBOBOX"),"",CBS_DROPDOWNLIST as u32|WS_VSCROLL.0),
            (NEARBY,w!("COMBOBOX"),"",CBS_DROPDOWNLIST as u32|WS_VSCROLL.0),
            (BODY,w!("EDIT"),"",ES_MULTILINE as u32|ES_READONLY as u32|ES_AUTOVSCROLL as u32|WS_VSCROLL.0|WS_BORDER.0),
            (LIST,w!("LISTBOX"),"",LBS_NOTIFY as u32|LBS_NOINTEGRALHEIGHT as u32|WS_VSCROLL.0|WS_HSCROLL.0),
            (QUANTITY,w!("EDIT"),"1",ES_NUMBER as u32|WS_BORDER.0),(MONEY,w!("STATIC"),"",0),(HELP,w!("STATIC"),"租裝服務：選擇配裝，再確認服務入口",0),
        ]{let text=wide(label);let h=CreateWindowExW(WINDOW_EX_STYLE(0),class,PCWSTR(text.as_ptr()),WS_CHILD|WS_TABSTOP|WINDOW_STYLE(style),0,0,0,0,Some(self.parent),Some(HMENU(id as usize as *mut _)),Some(instance),None)?;self.controls.push((id,h));}
        for n in 1..=4{let label=wide(&format!("配裝 {n}"));send(self.control(SETS),CB_ADDSTRING,WPARAM(0),LPARAM(label.as_ptr() as isize));}
        send(self.control(SETS),CB_SETCURSEL,WPARAM(0),LPARAM(0));send(self.control(QUANTITY),EM_SETLIMITTEXT,WPARAM(10),LPARAM(0));Ok(())
    }
    pub(super) unsafe fn layout_rental(&self){
        let dpi=GetDpiForWindow(self.parent).max(96) as i32;let px=|n:i32|n*dpi/96;let mut area=RECT::default();let _=GetClientRect(self.parent,&mut area);
        send(self.control(LINKS),CB_SETITEMHEIGHT,WPARAM(0),LPARAM(px(32) as isize));send(self.control(LINKS),CB_SETITEMHEIGHT,WPARAM(usize::MAX),LPARAM(px(32) as isize));
        let s=self.controller.snapshot();let entry=entry(s)&&!self.inventory_view;let town=rental(s)&&self.world_view&&!self.inventory_view;let panel=town&&self.rental.open;let hotel=hotel_room(s);
        for id in IDS{let show=match id{SET|ENTER|SETS|SERVICES|HELP=>entry,TOGGLE=>town&&!hotel,_=>panel};let _=ShowWindow(self.control(id),if show{SW_SHOW}else{SW_HIDE});}
        if entry{for id in [SOURCES,304,SELECT,ROOMS,305,306,ROOM_PASSWORD,JOIN,READY,UNREADY,EXIT_ROOM,CHANGE_FORCE]{let _=ShowWindow(self.control(id),SW_HIDE);}}
        for(id,x,y,w,h) in [(HELP,20,476,area.right-px(40),25),(SETS,20,510,px(140),180),(SERVICES,170,510,area.right-px(190),180),(SET,20,558,px(130),32),(ENTER,170,558,px(150),32)]{
            let _=MoveWindow(self.control(id),px(x),px(y),w,px(h),true);
        }
        if rental(s)&&self.world_view{
            let _=ShowWindow(self.control(RETURN),if town&&hotel{SW_SHOW}else{SW_HIDE});
            if hotel{self.text(RETURN,"返回選角");}
            let _=MoveWindow(self.control(TOGGLE),px(160),area.bottom-px(45),px(130),px(32),true);
        }
        if panel{
            for id in [SKILLS,CAST_SKILL,GROUND_TOGGLE]{let _=ShowWindow(self.control(id),SW_HIDE);}
            for(id,x,y,w,h) in [(NEARBY,20,20,area.right-px(180),180),(TALK,area.right*96/dpi-145,20,px(125),32),
                (MONEY,20,65,area.right-px(40),48),(BODY,20,118,area.right-px(40),175),
                (LINKS,20,305,area.right-px(170),180),(FOLLOW,area.right*96/dpi-140,305,px(120),32),
                (BUY,20,350,px(130),32),(SELL,170,350,px(130),32),(BACK,320,350,px(130),32),
                (LIST,20,395,area.right-px(40),127),(QUANTITY,20,535,px(100),28),(ORDER,140,535,px(145),32)]{
                let _=MoveWindow(self.control(id),px(x),px(y),w,px(h),true);
            }
            let _=MoveWindow(self.control(STATUS),px(20),px(580),area.right-px(40),px(90),true);
        }
    }
    pub(super) unsafe fn refresh_rental(&mut self){
        self.rental.documents.poll();
        let generation=self.controller.snapshot().world.generation;
        if generation!=self.rental.observed_generation{self.rental.observed_generation=generation;self.rental.open=false;self.rental.binding=None;self.rental.feedback.clear();self.rental.message_floor=self.controller.snapshot().world.system_message_count;self.rental.code_floor=self.controller.snapshot().world.message_count;}
        if !rental(self.controller.snapshot())||self.closing||self.inventory_view{self.rental.open=false;}
        if !entry(self.controller.snapshot()){self.rental.set_attempt=None;}
        let s=self.controller.snapshot();let current=if entry(s){Some((self.controller.transport().snapshot().epoch,s.selected_character.clone(),s.services.as_ref().unwrap().rooms.clone()))}else{None};
        if self.rental.entry!=current{self.rental.entry=current;send(self.control(SERVICES),CB_RESETCONTENT,WPARAM(0),LPARAM(0));if let Some((_,_,rooms))=&self.rental.entry{for r in rooms{let text=wide(&format!("{}（{} 人）",r.name,r.users));send(self.control(SERVICES),CB_ADDSTRING,WPARAM(0),LPARAM(text.as_ptr() as isize));}send(self.control(SERVICES),CB_SETCURSEL,WPARAM(0),LPARAM(0));}}
        let targets:Vec<_>=self.controller.snapshot().world.actors.keys().filter_map(|id|Target::capture(&self.controller,*id)).take(128).collect();
        if targets!=self.rental.targets{self.rental.targets=targets;send(self.control(NEARBY),CB_RESETCONTENT,WPARAM(0),LPARAM(0));for t in &self.rental.targets{let text=wide(&t.name);send(self.control(NEARBY),CB_ADDSTRING,WPARAM(0),LPARAM(text.as_ptr() as isize));}}
        let s=self.controller.snapshot();let session=&s.merchant;
        if self.rental.binding.as_ref().is_some_and(|b|!b.target.valid(&self.controller)||session.npc!=Some(b.target.id)||session.owner.as_ref()!=Some(&b.target.owner)) {self.rental.binding=None;self.rental.feedback.clear();}
        if let Some(dialog)=&session.dialog{
            if self.rental.binding.as_ref().is_none_or(|b|b.received_revision!=session.revision){
                self.rental.binding=None;
                if let Some(target)=Target::capture(&self.controller,dialog.npc).filter(|t|session.owner.as_ref()==Some(&t.owner)){
                    let arguments=crate::modern_lito::native_npc_document::resolved_arguments(&dialog.html,&dialog.arguments,&target.name);
                    if let Some(catalog)=self.rental.documents.catalog(){match catalog.get(&dialog.html).and_then(|doc|Ok((doc,doc.text_with_args(&arguments)?))){
                        Ok((doc,text)) if dialog.parameter.is_empty()=>self.rental.binding=Some(Binding{target,text:text.replace('\n',"\r\n"),actions:doc.actions.clone(),images:doc.images.clone(),trailing:doc.trailing_images.clone(),html:dialog.html.clone(),received_revision:session.revision,history:vec![],arguments}),
                        Ok(_)=>self.rental.feedback="此對話包含尚未支援的動態參數".into(),Err(e)=>self.rental.feedback=format!("{e:#}"),
                    }}else{self.rental.feedback=self.rental.documents.error.clone().unwrap_or_else(||"正在載入原始 NPC 對話…".into());}
                }
            }
        }
        let text=self.rental.binding.as_ref().map(|b|b.text.as_str()).unwrap_or(if self.rental.feedback.is_empty(){"選取附近對象並開啟對話。"}else{&self.rental.feedback});
        if GetWindowTextLengthW(self.control(BODY)) as usize!=text.encode_utf16().count()||self.read(BODY,MAX_DIALOG_DISPLAY).is_ok_and(|v|v.as_str()!=text){self.text(BODY,text);}
        let actions=self.rental.binding.as_ref().map(|b|b.actions.iter().filter(|a|a.navigate||!matches!(a.command.as_str(),"buy"|"sell")).cloned().collect()).unwrap_or_default();
        if self.rental.action_rows!=actions{self.rental.action_rows=actions;send(self.control(LINKS),CB_RESETCONTENT,WPARAM(0),LPARAM(0));for a in &self.rental.action_rows{
            // The original B label carries an authoring note after its name.
            // Keep the source document intact and show the destination name.
            let label=if matches!(a.command.as_str(),"LT_TELEPORT_B"|"LT_TELEPORT_C"){a.label.split("(這是server名.").next().unwrap_or(&a.label).trim_end()}else{&a.label};
            let text=wide(label);send(self.control(LINKS),CB_ADDSTRING,WPARAM(0),LPARAM(text.as_ptr() as isize));
        }}
        let key=if session.listing.is_some()||session.input.is_some(){session.owner.clone().map(|o|(o,session.revision))}else{None};
        if key!=self.rental.list_key{if let Some(input)=&session.input{self.text(QUANTITY,&input.current.to_string());}}
        let labels=match &session.listing{
            Some(Listing::Buy{rows,..})=>rows.iter().map(|r|format!("{}　租用單價 {}",r.name,r.price)).collect(),
            Some(Listing::Sell{rows,..})=>rows.iter().map(|r|{let name=s.inventory.items.get(&r.id).map(|i|crate::modern_lito::native_inventory_names::label(i,self.inventory_names.catalog()).text).unwrap_or_else(||"物品已不在背包".into());format!("{name}　歸還單價 {}",r.price)}).collect(),_=>Vec::new(),
        };
        if key!=self.rental.list_key||labels!=self.rental.labels{self.rental.list_key=key;self.rental.labels=labels;send(self.control(LIST),LB_RESETCONTENT,WPARAM(0),LPARAM(0));for text in &self.rental.labels{let text=wide(text);send(self.control(LIST),LB_ADDSTRING,WPARAM(0),LPARAM(text.as_ptr() as isize));}self.list_extent(LIST,self.rental.labels.iter().map(String::as_str));}
        let gems=if let Some(c)=self.inventory_names.catalog(){let n:u64=s.inventory.items.values().filter(|i|c.resolve(&i.name).is_ok_and(|name|name=="光之寶石")).map(|i|u64::from(i.count)).sum();n.to_string()}else{"載入名稱中".into()};
        let points=s.shop_points.map(|n|n.to_string()).unwrap_or_else(||"未收到".into());self.text(MONEY,&format!("背包光之寶石：{gems}　商店積分：{points}\r\n交易數量請填於下方，餘額以伺服器回覆為準。"));
        self.text(ORDER,if session.input.is_some(){"確認數量"}else{"送出交易"});
        self.text(TOGGLE,if self.rental.open{"返回場景"}else{"NPC／租裝"});
        let ready=!self.closing&&!session.waiting&&self.controller.merchant_owner().is_some();
        let can=|cmd:&str|ready&&self.rental.binding.as_ref().is_some_and(|b|b.actions.iter().any(|a|!a.navigate&&a.command==cmd));
        for(id,enabled) in [(SETS,matches!(s.phase,LoginPhase::CharacterSelected|LoginPhase::RentalReady)),(SET,matches!(s.phase,LoginPhase::CharacterSelected|LoginPhase::RentalReady)),
            (SERVICES,s.phase==LoginPhase::RentalReady),(ENTER,s.phase==LoginPhase::RentalReady),(TOGGLE,rental(s)),(TALK,ready&&!self.rental.targets.is_empty()),(NEARBY,ready),
            (LINKS,ready&&!self.rental.action_rows.is_empty()),(FOLLOW,ready&&!self.rental.action_rows.is_empty()),(BACK,ready&&self.rental.binding.as_ref().is_some_and(|b|!b.history.is_empty())),
            (BUY,can("buy")),(SELL,can("sell")),(ORDER,ready&&self.rental.binding.is_some()&&self.rental.list_key.is_some()&&(session.input.as_ref().is_some_and(|v|v.mode==0)||send(self.control(LIST),LB_GETCURSEL,WPARAM(0),LPARAM(0)).0>=0)),(QUANTITY,ready&&self.rental.list_key.is_some())]{let _=EnableWindow(self.control(id),enabled&&!self.closing);}
        if rental(s){let _=EnableWindow(self.control(RETURN),ready&&hotel_room(s)&&self.world_view&&!self.inventory_view);}
        let key=(entry(s),rental(s),self.rental.open,self.inventory_view,hotel_room(s));
        if self.rental.layout_key!=Some(key){self.rental.layout_key=Some(key);self.layout(self.parent,false);let _=InvalidateRect(Some(self.parent),None,true);}
    }
    pub(super) unsafe fn rental_action(&mut self,id:u16)->Result<bool>{
        match id{
            RETURN if hotel_room(self.controller.snapshot())=>{
                ensure!(!self.closing&&self.world_view&&!self.inventory_view,"目前不可從旅館返回選角");
                self.controller.return_from_rental()?;
                self.held.clear();self.held_attack.clear();self.rental.open=false;
                self.rental.binding=None;self.painted=None;
            },
            SET|ENTER=>{
                let(epoch,character,rooms)=self.rental.entry.as_ref().context("租裝服務清單已失效")?;let s=self.controller.snapshot();
                ensure!(*epoch==self.controller.transport().snapshot().epoch&&*character==s.selected_character&&s.services.as_ref().is_some_and(|d|d.rooms==*rooms),"租裝服務或角色已變更");
                let n=send(self.control(if id==SET{SETS}else{SERVICES}),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;ensure!(n>=0,"請先選取配裝或租裝服務");
                if id==SET{let index=u8::try_from(n)?;self.controller.select_rental_set(index)?;self.rental.set_attempt=Some(index);}else{self.controller.join_rental_room(n as usize)?;}
            },
            TOGGLE=>{ensure!(rental(self.controller.snapshot()),"目前不在租裝村");self.rental.open=!self.rental.open;self.held.clear();self.held_attack.clear();self.painted=None;self.layout(self.parent,false);let _=InvalidateRect(Some(self.parent),None,true);},
            TALK=>{let n=send(self.control(NEARBY),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;let t=usize::try_from(n).ok().and_then(|i|self.rental.targets.get(i)).context("請選取附近對象")?;ensure!(t.valid(&self.controller),"對象已離開或改變");let id=t.id;self.rental.binding=None;self.rental.feedback.clear();self.controller.rental_npc_talk(id)?;},
            FOLLOW|BACK=>{
                let b=self.rental.binding.as_ref().context("請重新開啟對話")?;ensure!(b.target.valid(&self.controller)&&!self.controller.snapshot().merchant.waiting,"NPC 對話已失效或等待回覆");
                let action=if id==BACK{Action{command:b.history.last().context("沒有上一頁")?.clone(),label:String::new(),navigate:true}}else{
                    let n=send(self.control(LINKS),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;usize::try_from(n).ok().and_then(|i|self.rental.action_rows.get(i)).context("請先選取對話項目")?.clone()
                };
                if action.navigate{
                    let doc=self.rental.documents.catalog().context("NPC 資源尚未就緒")?.get(&action.command)?;
                    let text=doc.text_with_args(&b.arguments)?.replace('\n',"\r\n");let actions=doc.actions.clone();let images=doc.images.clone();let trailing=doc.trailing_images.clone();let b=self.rental.binding.as_mut().unwrap();
                    if id==BACK{b.history.pop();}else{ensure!(b.history.len()<16,"對話頁數超出限制，請重新對話");b.history.push(b.html.clone());}
                    b.html=action.command;b.text=text;b.actions=actions;b.images=images;b.trailing=trailing;
                }else{self.controller.rental_npc_action(b.target.id,&action.command)?;}
            },
            BUY|SELL=>{let b=self.rental.binding.as_ref().context("請重新開啟對話")?;let action=if id==BUY{"buy"}else{"sell"};ensure!(b.target.valid(&self.controller)&&b.actions.iter().any(|a|a.command==action),"NPC 對話選項已失效");self.controller.rental_npc_action(b.target.id,action)?;},
            ORDER=>{let b=self.rental.binding.as_ref().context("請重新開啟 NPC 對話")?;ensure!(b.target.valid(&self.controller),"交易對象已變更");let(owner,revision)=self.rental.list_key.as_ref().context("請重新取得交易清單")?;ensure!(self.controller.merchant_owner().as_ref()==Some(owner),"交易角色已變更");let quantity=self.read(QUANTITY,10)?.parse::<u32>().context("請輸入有效數量")?;if let Some(input)=self.controller.snapshot().merchant.input.as_ref(){ensure!(quantity>=input.minimum&&quantity<=input.maximum,"請輸入 {}～{} 的數量",input.minimum,input.maximum);self.controller.rental_npc_quantity(*revision,quantity)?;}else{let index=send(self.control(LIST),LB_GETCURSEL,WPARAM(0),LPARAM(0)).0;ensure!(index>=0,"請先選取交易項目");self.controller.rental_merchant_order(*revision,index as usize,quantity)?;}},
            _=>return Ok(false),
        }
        if matches!(id,TALK|BUY|SELL|ORDER|FOLLOW){self.rental.message_floor=self.controller.snapshot().world.system_message_count;self.rental.code_floor=self.controller.snapshot().world.message_count;}
        Ok(true)
    }
    pub(super) fn rental_status(&self)->String{
        let s=self.controller.snapshot();let m=&s.merchant;
        let refusal=if m.submitted&&s.world.message_count>self.rental.code_floor{crate::modern_lito::native_merchant::inventory_refusal(s.world.last_message.as_ref())}else{None};
        let stage=if let Some(reason)=refusal{reason}else if m.waiting{"正在等待 NPC 回覆…"}else if m.timed_out{"NPC 回覆逾時，請重新開啟對話。"}else if m.submitted{"已送出交易，請核對背包回覆與餘額。"}else if m.listing.is_some(){"選取項目並填入數量，再送出交易。"}else{"請選取附近對象，或使用收到的對話選項。"};
        let message=if s.world.system_message_count>self.rental.message_floor{s.world.last_system_message.as_ref().map(|(_,s)|s.as_str()).unwrap_or("")}else{""};
        let request=m.input.as_ref().map(|v|format!("{}\r\n請輸入 {}～{} 的數量，再按確認數量。",v.prompt,v.minimum,v.maximum)).unwrap_or_default();
        let image_error=self.rental.binding.as_ref().filter(|b|b.html=="TeleportMe").and_then(|_|self.rental.documents.catalog()).and_then(|c|c.image_error.as_deref()).unwrap_or("");
        format!("{stage}\r\n{request}\r\n最近系統訊息：{message}\r\n{image_error}")
    }
}
const MAX_DIALOG_DISPLAY:usize=crate::modern_lito::native_npc_document::MAX_BYTES*3;
