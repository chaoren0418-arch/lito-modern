"""Wire #69's owned destination, consent and server summon continuation into native runtime source."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-mass69-integration-20261025-004000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_mass69_fire70_development'];require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and sha(ref['qualification'])==ref['qualification_sha256'],'phase differs');q=read(ref['qualification']);require(sha(q['contract'])==q['contract_sha256'],'prior changed');base=read(q['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()});require(not C.exists(),'candidate exists');prepared={};changes={};counts={}
 def change(short,a,b,n=1):
  s=prepared.setdefault(short,Path(paths[PREFIX+short]).read_text('utf-8-sig'));require(s.count(a)==n,'source anchor differs '+short+': '+a[:90]+' count='+str(s.count(a)));prepared[short]=s.replace(a,b);changes.setdefault(short,[]).append([a,b]);
  if n!=1:counts.setdefault(short,{})[a]=n
 def fragment(name):return (T/'templates'/name).read_text('utf-8-sig')
 change('native_mass69.rs','    pub(crate) fn request(&self,s:&Snapshot,epoch:u64,book:&Bookmarks)->Result<Vec<u8>> {','''    pub(crate) fn current(&self,s:&Snapshot,epoch:u64,book:&Bookmarks)->bool {Scene::current(s,epoch).as_ref()==Some(&self.owner)&&self.bookmark.as_ref().is_none_or(|(revision,row)|*revision==book.revision&&book.rows(s,epoch).contains(row))}
    pub(crate) fn request(&self,s:&Snapshot,epoch:u64,book:&Bookmarks)->Result<Vec<u8>> {''')
 change('native_mass69.rs','room:Option<u32>,actor:','room:Option<(u32,u8)>,actor:')
 change('native_mass69.rs','        Some(Self{character:Character::current(s,epoch)?,','''        if s.phase==LoginPhase::BattlePending&&s.joined_room.is_none()||s.phase==LoginPhase::WorldReady&&s.joined_room.is_some(){return None;}
        Some(Self{character:Character::current(s,epoch)?,''')
 change('native_mass69.rs','room:s.joined_room.as_ref().map(|r|r.id),actor:','room:s.joined_room.as_ref().map(|r|(r.id,r.mode)),actor:')
 change('native_mass69.rs','    pub(crate) fn current(&self,s:&Snapshot,epoch:u64)->bool {self.owner.is_some()','    pub(crate) fn pending(&self)->bool{self.owner.is_some()}\n    pub(crate) fn current(&self,s:&Snapshot,epoch:u64)->bool {self.owner.is_some()')
 change('native_mass69.rs','ensure!(d.map==owner.map,"battle summon portal changed map")','ensure!(d.map==owner.map||owner.room.is_some_and(|(_,mode)|mode==6),"battle summon portal changed map")')
 change('native_skills.rs','pub fn item_targeted(skill:u16)->bool {','''pub fn mass_targeted(skill:u16)->bool {matches!(skill,69)}
pub fn mass_request(s:&super::native_login::Snapshot,epoch:u64,book:&super::native_mass69::Bookmarks,choice:&super::native_mass69::Choice)->Result<Vec<u8>> {choice.request(s,epoch,book)}
pub fn item_targeted(skill:u16)->bool {''')
 change('native_skill_selection.rs','                        70 =>','                        69 => "#69 集體傳送術（選擇隨機／書籤目的地；消耗 1 顆魔法寶石）".into(),\n                        70 =>')
 short='native_login.rs'
 change(short,'    shape:super::native_shape67::Session,','''    mass_books:super::native_mass69::Bookmarks,
    mass_invitation:super::native_mass69::Invitation,
    mass_portal:super::native_mass69::PortalGrant,
    mass_deadline:Option<Instant>,
    shape:super::native_shape67::Session,''')
 change(short,'            shape:Default::default(),','            mass_books:Default::default(),mass_invitation:Default::default(),mass_portal:Default::default(),mass_deadline:None,\n            shape:Default::default(),')
 change(short,'    pub(crate) fn shape_prompt(',fragment('native_mass69_controller_methods.rs')+'\n    pub(crate) fn shape_prompt(')
 change(short,'self.resurrection.retire();self.shape.retire();','self.resurrection.retire();self.shape.retire();self.mass_invitation.retire();self.mass_portal.retire();self.mass_deadline=None;',2)
 for a,b in [('self.cast_request(skill, target, None, None, None, frame)','self.cast_request(skill, target, None, None, None, None, frame)'),('self.cast_request(skill, None, Some(position), None, None, frame)','self.cast_request(skill, None, Some(position), None, None, None, frame)'),('self.cast_request(skill,None,None,Some(target),None,frame)','self.cast_request(skill,None,None,Some(target),None,None,frame)'),('self.cast_request(51,None,None,None,Some(target),frame)','self.cast_request(51,None,None,None,Some(target),None,frame)')]:change(short,a,b)
 change(short,'    fn cast_request(','''    pub(crate) fn cast_mass(&mut self,choice:&super::native_mass69::Choice,frame:&super::native_world_scene::Frame)->Result<()> {self.cast_request(69,None,None,None,None,Some(choice),frame)}
    fn cast_request(''')
 change(short,'        summon: Option<&super::native_skills::SummonTarget>,','        summon: Option<&super::native_skills::SummonTarget>,\n        mass:Option<&super::native_mass69::Choice>,')
 change(short,'        let body = if let Some(summon)=summon {','        ensure!(mass.is_none()||skill==69&&summon.is_none()&&item.is_none()&&ground.is_none()&&target.is_none(),"混合集體傳送目的地");\n        ensure!(skill!=69||mass.is_some(),"請明確選擇集體傳送目的地");\n        let body = if let Some(mass)=mass {super::native_skills::mass_request(&self.state,self.transport.snapshot().epoch,&self.mass_books,mass)?}else if let Some(summon)=summon {')
 for name in ['rental_npc_talk','rental_npc_action','rental_merchant_order','rental_npc_quantity','pickup','item_action','item_target_action','attack','move_step','enter_map_portal','toggle_door','request_battle_withdrawal','return_from_rental','cast_request']:
  s=prepared[short];matches=list(re.finditer(r'\b(?:pub(?:\(crate\))? )?fn '+name+r'\([^{}]*?\)\s*->\s*[^{}]+?\{',s));require(len(matches)==1,'world input header differs '+name);a=matches[0].group();change(short,a,a+'\n        self.ensure_native_input()?;')
 change(short,'        if self.resurrection.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}','''        if self.mass_books.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}
        if self.mass_invitation.receive(&self.state,self.transport.snapshot().epoch,Instant::now(),body)?{self.resurrection.retire();self.shape.retire();return Ok(());}
        if body.first()==Some(&0x55){self.accept_native_summon(body)?;return Ok(());}
        if self.resurrection.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}''')
 a='''            let tree=self.world_tree_request.take().filter(|(stamp,since)|
                *stamp==super::native_navigation::InputStamp::capture(self.transport.snapshot().epoch,&self.state.world)
                && since.elapsed()<Duration::from_secs(2));
            if tree.is_some(){
                super::native_portal::admit_world_tree(&self.state,destination)?;
                self.state.phase=LoginPhase::WorldEntering;
            }else{super::native_portal::admit(&self.state,destination)?;}'''
 change(short,a,'            self.admit_received_portal(destination)?;')
 change(short,'        self.resurrection.synchronize(&self.state,self.transport.snapshot().epoch);','''        self.mass_invitation.synchronize(&self.state,self.transport.snapshot().epoch,Instant::now());
        if let Err(e)=self.mass_books.synchronize(&self.state,self.transport.snapshot().epoch){self.fail(format!("bookmark owner: {e:#}"));}
        if self.mass_portal.pending()&&(!self.mass_portal.current(&self.state,self.transport.snapshot().epoch)||self.mass_deadline.is_some_and(|d|Instant::now()>=d)){self.fail("native summon owner retired or eight-second portal timeout".into());}
        self.resurrection.synchronize(&self.state,self.transport.snapshot().epoch);''')
 short='native_login_window.rs'
 change(short,'const SHAPE_LABEL:u16=320;','const MASS_LABEL:u16=321;\nconst MASS_OPTIONS:u16=219;\nconst MASS_YES:u16=142;\nconst MASS_NO:u16=143;\nconst SHAPE_LABEL:u16=320;')
 change(short,'    shape_token:Option<super::native_shape67::Token>,','    mass_token:Option<super::native_mass69::Token>,\n    mass_was_pending:bool,\n    mass_menu:super::native_mass69::Menu,\n    mass_choice:Option<super::native_mass69::Choice>,\n    shape_token:Option<super::native_shape67::Token>,')
 change(short,'            (SHAPE_LABEL,w!("STATIC"),String::new(),0),','            (MASS_LABEL,w!("STATIC"),String::new(),0),\n            (MASS_YES,w!("BUTTON"),"同意傳送".into(),0),\n            (MASS_NO,w!("BUTTON"),"不同意".into(),0),\n            (MASS_OPTIONS,w!("COMBOBOX"),String::new(),CBS_DROPDOWNLIST as u32|WS_VSCROLL.0),\n            (SHAPE_LABEL,w!("STATIC"),String::new(),0),')
 change(short,'            let visible = match *id {','''            let visible = match *id {
                MASS_LABEL=>self.world_view&&!self.result_view&&!self.closing&&(self.controller.mass_prompt().is_some()||self.controller.mass_travel_pending()),
                MASS_YES|MASS_NO=>self.world_view&&!self.result_view&&!self.closing&&self.controller.mass_prompt().is_some()&&!self.controller.mass_travel_pending(),
                SKILL_LABEL|SKILLS|CAST_SKILL|SUMMON_OPTIONS|MASS_OPTIONS|SHAPE_LABEL|SHAPE_OPTIONS|SHAPE_APPLY|SHAPE_CLOSE|RESURRECT_LABEL|RESURRECT_YES|RESURRECT_NO|INVENTORY_TOGGLE|GROUND_TOGGLE|RETURN if self.controller.mass_prompt().is_some()||self.controller.mass_travel_pending()=>false,
                MASS_OPTIONS=>self.world_view&&!self.result_view&&!self.inventory_view&&!self.rental.open&&self.skills.selected()==Some(69)&&self.controller.resurrection_prompt().is_none()&&self.controller.shape_prompt().is_none(),''')
 change(short,'if self.skills.selected()==Some(51)&&!self.inventory_view&&!self.rental.open {','if matches!(self.skills.selected(),Some(51|69))&&!self.inventory_view&&!self.rental.open {')
 change(short,'            let _=MoveWindow(self.control(RESURRECT_LABEL)','''            let _=MoveWindow(self.control(MASS_LABEL),px(20),area.bottom-px(132),(area.right-px(40)).max(px(240)),px(32),true);
            let _=MoveWindow(self.control(MASS_YES),px(64),area.bottom-px(90),px(116),px(32),true);
            let _=MoveWindow(self.control(MASS_NO),px(196),area.bottom-px(90),px(116),px(32),true);
            let _=MoveWindow(self.control(RESURRECT_LABEL)''')
 change(short,'            send(self.control(SUMMON_OPTIONS),CB_SETDROPPEDWIDTH,WPARAM(px(440) as usize),LPARAM(0));','            send(self.control(SUMMON_OPTIONS),CB_SETDROPPEDWIDTH,WPARAM(px(440) as usize),LPARAM(0));\n            let _=MoveWindow(self.control(MASS_OPTIONS),px(64)+skill_width/2+px(4),area.bottom-px(90),(skill_width-skill_width/2-px(4)).max(px(40)),px(300),true);\n            send(self.control(MASS_OPTIONS),CB_SETDROPPEDWIDTH,WPARAM(px(520) as usize),LPARAM(0));')
 change(short,'        if super::native_skills::summon_targeted(skill) {','''        if super::native_skills::mass_targeted(skill) {
            let epoch=self.controller.transport().snapshot().epoch;ensure!(self.skills.selected_skill(epoch,&self.controller.snapshot().world)?==69&&!self.inventory_view&&!self.rental.open&&!self.closing,"集體傳送畫面已失效");
            let choice=self.mass_choice.clone().context("請明確選擇隨機傳送或書籤目的地")?;let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;return self.controller.cast_mass(&choice,frame);
        }
        if super::native_skills::summon_targeted(skill) {''')
 change(short,'if self.controller.resurrection_prompt().is_some()||self.controller.shape_prompt().is_some(){','if self.controller.resurrection_prompt().is_some()||self.controller.shape_prompt().is_some()||self.controller.mass_prompt().is_some()||self.controller.mass_travel_pending(){')
 change(short,'        if !self.inventory_view||self.ground_view||self.closing {self.skill_item_target=None;}','        self.refresh_mass_destinations();let snapshot=self.controller.snapshot();\n        if !self.inventory_view||self.ground_view||self.closing {self.skill_item_target=None;}')
 change(short,'        if self.skills.selected()!=Some(51)||self.inventory_view||self.rental.open||self.closing||','        if self.skills.selected()!=Some(51)||self.inventory_view||self.rental.open||self.closing||self.controller.mass_prompt().is_some()||self.controller.mass_travel_pending()||')
 change(short,'        let choice_valid=if self.skills.selected()==Some(51) {','        let choice_valid=if self.skills.selected()==Some(69) {\n            !self.inventory_view&&!self.rental.open&&self.skills.selected_skill(epoch,&snapshot.world).is_ok()&&self.mass_choice.as_ref().is_some_and(|c|self.controller.mass_choice_ready(c))\n        }else if self.skills.selected()==Some(51) {')
 change(short,'            && choice_valid','            && choice_valid&&self.controller.mass_prompt().is_none()&&!self.controller.mass_travel_pending()')
 change(short,'let show_summon=self.controller.resurrection_prompt().is_none()&&','let show_summon=self.controller.mass_prompt().is_none()&&!self.controller.mass_travel_pending()&&self.controller.resurrection_prompt().is_none()&&')
 change(short,'        if IsWindowVisible(self.control(SUMMON_OPTIONS)).as_bool()!=show_summon {self.layout(self.parent,false);}','        let show_mass=self.world_view&&!self.result_view&&!self.inventory_view&&!self.rental.open&&self.skills.selected()==Some(69)&&self.controller.mass_prompt().is_none()&&!self.controller.mass_travel_pending()&&self.controller.resurrection_prompt().is_none()&&self.controller.shape_prompt().is_none();\n        if IsWindowVisible(self.control(SUMMON_OPTIONS)).as_bool()!=show_summon||IsWindowVisible(self.control(MASS_OPTIONS)).as_bool()!=show_mass {self.layout(self.parent,false);}')
 change(short,'    unsafe fn refresh_shape(&mut self) {',fragment('native_mass69_window_refresh.rs')+'\n    unsafe fn refresh_shape(&mut self) {')
 change(short,'        self.refresh_resurrection();','        self.refresh_mass();\n        self.refresh_resurrection();')
 change(short,'    unsafe fn action(&mut self, id: u16) -> Result<()> {','    unsafe fn action(&mut self, id: u16) -> Result<()> {\n        ensure!(!self.controller.mass_travel_pending()||id==CANCEL,"等待伺服器傳送結果");\n        ensure!(self.controller.mass_prompt().is_none()||matches!(id,RETURN|CANCEL),"請先回覆集體傳送邀請");')
 change(short,'                WM_LBUTTONDOWN if ui.controller.shape_prompt().is_some()=>','                WM_LBUTTONDOWN if ui.controller.mass_prompt().is_some()||ui.controller.mass_travel_pending()=>{return LRESULT(0);}\n                WM_LBUTTONDOWN if ui.controller.shape_prompt().is_some()=>')
 change(short,'                WM_COMMAND if lparam.0!=0 && [ui.control(SHAPE_OPTIONS)',fragment('native_mass69_window_commands.rs')+'\n                WM_COMMAND if lparam.0!=0 && [ui.control(SHAPE_OPTIONS)')
 change(short,'                shape_token:None,shape_choice:None,','                mass_token:None,mass_was_pending:false,mass_menu:Default::default(),mass_choice:None,\n                shape_token:None,shape_choice:None,')
 change(short,'                // The received menu owns world shortcuts until it closes.',fragment('native_mass69_window_keys.rs')+'\n                // The received menu owns world shortcuts until it closes.')
 supplements={'native_mass69.rs':[T/'templates/native_mass69_menu.rs'],'native_login.rs':[T/'templates/native_mass69_controller_tests.rs'],'native_skills.rs':[T/'templates/native_mass69_request_tests.rs']}
 for short,rows in supplements.items():
  for p in rows:prepared[short]+='\n'+p.read_text('utf-8-sig')
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;prior=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(prior),predecessor_sha256=sha(prior),changes=changes[short],supplements=[dict(path=str(t),sha256=sha(t))for t in supplements.get(short,[])],repeated_replacement_counts=counts.get(short,{}));files[name]=sha(p);paths[name]=str(p)
 old=q['current_source_coverage'];cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(set(old['recognized_ids'])|{69}),recognized_count=76,missing_request_ids=sorted(set(old['missing_request_ids'])-{69}),missing_request_count=104);require('pub fn mass_targeted(skill:u16)->bool {matches!(skill,69)}'in prepared['native_skills.rs'] and len(cov['recognized_ids'])==76,'dedicated producer absent');cq=read(w['current_E149_skill_catalog_development']['qualification']);facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core']for r in facts['definitions_by_id']if r['skill_id']in next_ids];cohort=Path(q['compile_run'])/'cohort.json';cases=sum(p.read_text('utf-8-sig').count('#[test]')for rows in supplements.values()for p in rows);require((len(files),len(overrides),cases)==(42,5,9),'scope differs');tools=[Path(__file__).resolve(),T/'templates/native_mass69_controller_methods.rs',T/'templates/native_mass69_window_refresh.rs',T/'templates/native_mass69_window_commands.rs',T/'templates/native_mass69_window_keys.rs',*[p for rows in supplements.values()for p in rows]]
 k=dict(schema='E149-mass69-controller-window-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=base['inspection'],inspection_sha256=base['inspection_sha256'],original_pins=base['original_pins'],preparation_pins={str(p):sha(p)for p in tools},modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=[d for d in base['definitions']if d['skillId']==69],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=9,new_resource_opt_in_cases=0,previous_written_deferred_cases=362,mass69_components_source_complete=True,mass69_controller_or_GUI_integrated=True,mass69_request_registered=True,bookmarked_and_random_both_available=True,recipient_does_not_require_spell_or_MP=True,old_command_and_bookmark_ABA_fenced=True,own_S55_one_C4E_then_owned_S5A_C08=True,invalidated_summon_no_ordinary_fallback=True,original_eight_second_portal_wait_preserved=True,actual_GAME_overrides_observed=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(k));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=42,new_cases=9,recognized=76,missing=104,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
