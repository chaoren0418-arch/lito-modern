fn filtered_view()->View {
    let mut v=view();let scope=v.chat.scope();v.chat.align_profile(true,true,scope);
    let echo=v.chat.filtered_read().unwrap();
    let owner=super::super::direct_chat_protocol::Peer{key:echo.context.key,character_name:"3333".into(),source_name:"太陽神".into()};
    v.chat.receive_filtered(super::super::filtered_chat_protocol::Reply{echo,status:super::super::filtered_chat_protocol::Status::Ok,
        membership:7,sequence:0,cursor:0,available:true,truncated:false,owner,messages:vec![],receipt:None}).unwrap();v
}
#[test]
fn category_input_is_typed_and_legacy_cannot_silently_downgrade(){
    let mut ids=CONTROL_ORDER;ids.sort_unstable();
    assert_eq!(ids,std::array::from_fn(|i|i),"every native control must be created exactly once");
    let old=view();let mut draft=Draft::default();draft.align(old.chat.token());
    assert!(draft.intent_category(&old,"文字".into(),PublicKind::World).is_err());assert!(!draft.queued);
    let v=filtered_view();draft.align(v.chat.token());
    let intent=draft.intent_category(&v,"世界文字".into(),PublicKind::World).unwrap();assert_eq!(intent.kind,Kind::Public(PublicKind::World));
    assert_eq!(intent.token,v.chat.token().unwrap());assert!(draft.intent(&v,"重複".into()).is_err());
    assert_eq!(next_category(next_category(next_category(PublicKind::Normal))),PublicKind::Normal);
}

#[test]
#[ignore = "explicit native category UI inspection; synthetic state, no network or authentication"]
fn filtered_native_category_inspection_without_network() {
    unsafe {
        let view = filtered_view();
        let window = ChatWindow::open(HWND::default(), view.clone()).unwrap();
        assert!(window.state.controls.get().unwrap().iter().all(|h| IsWindow(Some(*h)).as_bool()));
        set_text(window.handle().unwrap(), "LiTo — 分類聊天檢查（測試資料／無連線）").unwrap();
        println!("FILTERED_UI_READY");
        let started = std::time::Instant::now();
        let mut refreshed = started;
        let mut seen = 0u8;
        let mut last_focus = HWND::default();
        // Normal application message loop. Only the external native UI tool
        // supplies input; no fabricated send or server receipt is produced.
        while window.handle().is_some() && started.elapsed() < std::time::Duration::from_secs(180) {
            let mut m = MSG::default();
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                if m.message == WM_KEYDOWN && [9,13,27,32].contains(&m.wParam.0) {
                    println!("FILTERED_UI_KEY key={} target={:?} category_style={:x}", m.wParam.0,
                        window.state.controls.get().unwrap().iter().position(|h|*h==m.hwnd),
                        GetWindowLongW(window.state.controls.get().unwrap()[CATEGORY],GWL_STYLE));
                }
                if !window.pre_translate(&m) {
                    let _ = TranslateMessage(&m);
                    DispatchMessageW(&m);
                }
            }
            seen |= match window.state.category.get() {
                PublicKind::Normal => 1,
                PublicKind::World => 2,
                PublicKind::Trade => 4,
            };
            let focus = GetFocus();
            if focus != last_focus {
                println!("FILTERED_UI_FOCUS control={:?}", window.state.controls.get().unwrap().iter().position(|h| *h==focus));
                last_focus = focus;
            }
            if window.handle().is_some() && refreshed.elapsed() >= std::time::Duration::from_millis(100) {
                window.update(view.clone()).unwrap();
                refreshed = std::time::Instant::now();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(window.handle().is_none(), "UI inspection watchdog expired");
        assert_eq!(seen, 7, "externally exercise all three categories before closing");
        println!("FILTERED_UI_CLOSED categories={seen}");
    }
}
#[test]
fn silenced_category_keeps_draft_and_read_cannot_report_delivery(){
    use super::super::filtered_chat_protocol as f;
    let mut v=filtered_view();let token=v.chat.token().unwrap();let mut d=Draft::default();d.align(Some(token));
    d.intent_category(&v,"禁言文字".into(),PublicKind::Trade).unwrap();d.dispatched(Kind::Public(PublicKind::Trade),true);
    v.chat_pending=true;assert!(d.confirmed(&v).is_none());
    let echo=v.chat.filtered_send(token,PublicKind::Trade,"禁言文字".into()).unwrap();
    v.chat.receive_filtered(f::Reply{echo,status:f::Status::Silenced,membership:7,sequence:1,cursor:0,available:true,truncated:false,
        owner:super::super::direct_chat_protocol::Peer{key:token.scope.context.key,character_name:"3333".into(),source_name:"太陽神".into()},messages:vec![],receipt:None}).unwrap();
    v.chat_pending=false;let (clear,text)=d.confirmed(&v).unwrap();assert!(!clear&&text.contains("禁言"));assert!(transcript(&v.chat).is_empty());
}
