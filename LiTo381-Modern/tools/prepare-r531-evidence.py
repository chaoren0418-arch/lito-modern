import pathlib,json
P=pathlib.Path(__file__).resolve().parent.parent;T=P/'target/r531-player-entry-candidate';T.mkdir()
w=P/'src/modern_lito/native_city_window.rs';s=w.read_text('utf-8')
def rep(a,b):
 global s
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
rep('unsafe fn action(hwnd:HWND,s:&State,a:Action){','''unsafe fn show_error(s:&State,error:&impl std::fmt::Display){
 if let Some(b)=s.backend.borrow_mut().as_mut(){b.record_failure("player_command_refused");}
 label(s.controls.get()[STATUS],&error.to_string());
}
unsafe fn action(hwnd:HWND,s:&State,a:Action){
 if a==Action::Close{if let(Some(b),Some(r))=(s.backend.borrow_mut().as_mut(),s.runtime.borrow().as_ref()){b.record_close_intent(r);}}
''')
s=s.replace('label(s.controls.get()[STATUS],&e.to_string())','show_error(s,&e)')
rep('let yes=MessageBoxW(Some(hwnd),w!("確定要離開嗎？"),w!("離開"),MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2)==IDYES;', 'let yes=MessageBoxW(Some(hwnd),w!("確定要離開嗎？"),w!("離開"),MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2)==IDYES;\n  if let Some(b)=s.backend.borrow_mut().as_mut(){b.record_entry_step("close_confirmation",Some(yes));}')
rep('if b.poll_health(r){*s.refresh_gate.borrow_mut()=Gate::default();}', 'b.observe_runtime(r);if b.poll_health(r){*s.refresh_gate.borrow_mut()=Gate::default();}')
rep('if let Some(r)=r.as_mut(){(r.poll(),r.page()==UiPage::Closed)}', 'if let Some(r)=r.as_mut(){let changed=r.poll();if let Some(b)=s.backend.borrow_mut().as_mut(){b.observe_runtime(r);}(changed,r.page()==UiPage::Closed)}')
rep('s.fault.set(true);if let Ok(mut r)=s.runtime.try_borrow_mut()', 's.fault.set(true);if let Ok(mut b)=s.backend.try_borrow_mut(){if let Some(b)=b.as_mut(){b.record_failure("window_callback_panicked");}}if let Ok(mut r)=s.runtime.try_borrow_mut()')
rep('if let(Some(b),Some(r))=(state.backend.borrow().as_ref(),state.runtime.borrow().as_ref()){b.finish(r)?;}', 'if let(Some(b),Some(r))=(state.backend.borrow_mut().as_mut(),state.runtime.borrow().as_ref()){b.finish(r)?;}')
rep('*state.runtime.borrow_mut()=Some(runtime);','if let Some(b)=state.backend.borrow_mut().as_mut(){b.observe_runtime(&runtime);}\n *state.runtime.borrow_mut()=Some(runtime);')
rep('"LiTo Modern · 玩家入口候選 R530"','"LiTo Modern · 玩家入口候選 R531"')
w.write_text(s,'utf-8',newline='\n')
main=(P/'target/r530-player-entry-final/LiToModern-R530.rs').read_text('utf-8').replace('R530_','R531_').replace('"r530"','"r531"')
with (T/'LiToModern-R531.rs').open('x',encoding='utf-8',newline='\n')as f:f.write(main)
build=(P/'tools/build-r530-final-player.ps1').read_text('utf-8').replace('r530-final','r531').replace('R530','R531').replace('r530','r531').replace("'target/r531-player-entry-final'","'target/r531-player-entry-candidate'").replace("$filter='r531_new_health_stage'","$filter='r531_evidence_checks'").replace('production_source_changes=6','production_source_changes=3').replace('production_source_count=643','production_source_count=644').replace('r531-entry-source-contract.json','r529-entry-source-contract.json')
build=build.replace("'R531_EXPLICIT_BACKEND485F_FINAL_PLAYER_COMPILED_GUI_UNEXECUTED'","'R531_BACKEND485F_BOUNDED_RECEIPT_EVIDENCE_COMPILED_GUI_UNEXECUTED'")
with (P/'tools/build-r531-player.ps1').open('x',encoding='utf-8',newline='\n')as f:f.write(build)
print('R531 timer/Close/error evidence hooks and separate main/build prepared; no provider/GUI execution')
