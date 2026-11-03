
#[derive(Clone,Debug)]
pub(crate) struct MenuRow {pub(crate) label:String,choice:Option<Choice>}
/// HWND and this non-reused command ID jointly scope a native combo notification.
#[derive(Default)]
pub(crate) struct Menu {serial:u16,owner:Option<(Scene,u64)>,rows:Vec<MenuRow>}
impl Menu {
    pub(crate) fn rows(&self)->&[MenuRow]{&self.rows}
    pub(crate) fn command_id(&self)->Option<u16>{self.owner.as_ref().map(|_|1000+self.serial)}
    pub(crate) fn retire(&mut self){self.owner=None;self.rows.clear();}
    pub(crate) fn synchronize(&mut self,s:&Snapshot,epoch:u64,book:&Bookmarks,active:bool)->Result<bool> {
        let owner=if active {Scene::current(s,epoch).map(|scene|(scene,book.revision))}else{None};
        if self.owner==owner{return Ok(false);}
        let Some((scene,revision))=owner else{self.retire();return Ok(true);};
        ensure!(self.serial<30_000,"集體傳送目的地命令已用盡，請重新開啟視窗");
        let mut rows=vec![MenuRow{label:"隨機傳送（目前地圖）".into(),choice:Some(Choice::random(s,epoch)?)}];
        for (i,row)in book.rows(s,epoch).iter().enumerate(){let d=row.destination;let label:String=row.label.chars().map(|c|if c.is_control(){' '}else{c}).collect();rows.push(MenuRow{label:format!("{label}（地圖 {}，{}, {}）",d.map,d.x,d.y),choice:Choice::stored(s,epoch,book,i).ok()});}
        self.serial+=1;self.owner=Some((scene,revision));self.rows=rows;Ok(true)
    }
    pub(crate) fn choose(&self,s:&Snapshot,epoch:u64,book:&Bookmarks,command:u16,index:usize)->Result<Choice> {
        ensure!(self.command_id()==Some(command)&&self.owner==Scene::current(s,epoch).map(|scene|(scene,book.revision)),"集體傳送目的地已更新，請重新選擇");
        self.rows.get(index).and_then(|r|r.choice.clone()).context("請選取有效的集體傳送目的地")
    }
}

#[cfg(test)]mod menu_integration_tests {
    use super::*;
    fn snapshot()->Snapshot {let mut s=super::super::native_login::LoginController::default().snapshot().clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();s.world.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}s.phase=LoginPhase::WorldReady;s.game_connected=true;s.selected_source=Some("isolated".into());s.selected_character=Some("fixture".into());s.inventory.owner=Some(native_inventory::Owner{connection:7,selection:1,character:"fixture".into()});s.world.hp=Some([1,100]);s.world.mp=Some([1,100]);s.world.learned_skills.insert(69);s}
    #[test]fn menu_requires_explicit_row_and_retired_combo_commands_cannot_bind_new_records(){let s=snapshot();let mut b=Bookmarks::default();let mut m=Menu::default();assert!(m.synchronize(&s,7,&b,true).unwrap());let old=m.command_id().unwrap();assert!(m.choose(&s,7,&b,old,1).is_err());let c=m.choose(&s,7,&b,old,0).unwrap();assert_eq!(&c.request(&s,7,&b).unwrap()[5..],&[0,0,0,0]);let mut wire=vec![48];wire.extend(b"town\0");wire.extend(123_u16.to_le_bytes());wire.extend(234_u16.to_le_bytes());wire.extend(4_u16.to_le_bytes());b.receive(&s,7,&wire).unwrap();m.synchronize(&s,7,&b,true).unwrap();assert!(m.choose(&s,7,&b,old,0).is_err());assert_eq!(m.rows().len(),2);let id=m.command_id().unwrap();let c=m.choose(&s,7,&b,id,1).unwrap();assert_eq!(c.request(&s,7,&b).unwrap(),vec![20,8,4,4,0,123,0,234,0]);}
    #[test]fn leaving_and_reopening_menu_never_reuses_native_id_or_character_lease(){let s=snapshot();let b=Bookmarks::default();let mut m=Menu::default();m.synchronize(&s,7,&b,true).unwrap();let id=m.command_id().unwrap();m.synchronize(&s,7,&b,false).unwrap();assert!(m.command_id().is_none());m.synchronize(&s,7,&b,true).unwrap();assert_ne!(m.command_id(),Some(id));let new=m.command_id().unwrap();let mut changed=s.clone();changed.inventory.owner.as_mut().unwrap().selection+=1;assert!(m.choose(&changed,7,&b,new,0).is_err());}
}
