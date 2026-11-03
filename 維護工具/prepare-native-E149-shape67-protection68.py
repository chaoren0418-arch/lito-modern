"""Prepare isolated source only: formal monlist, native controls and actor67/68 requests."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-shape67-protection68-20261024-154000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development phase required')
 ref=w['current_E149_appearance27_binding_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor changed');q=read(ref['qualification']);require(sha(q['contract'])==q['contract_sha256'],'predecessor contract changed');base=read(q['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()})
 inspection=V/'native-E149-shape67-protection68-inspection/20261024-151230-983350/inspection.json';require(sha(inspection)=='DB3CB3A449641D1ABA269EA9048A8558BCC09398580121D5CEFA07D1AC3367F1','inspection identity changed');proof=read(inspection)
 forms=read(inspection.parent/'standard-menu-forms.json');require(len(forms)==96 and len({r['selection'] for r in forms})==96 and all(r['standard_menu']=='1' and r['selection'].isascii() and 1<=int(r['min_level'])<=80 for r in forms),'formal standard menu differs')
 require(not C.exists(),'candidate already exists')
 render=T/'templates/native_protection68_render_tests.rs';require(not render.exists(),'render template already exists');render.write_text((T/'templates/native_sleep66_render_tests.rs').read_text('utf-8-sig').replace('sleep66_render','protection68_render').replace('original_sleep_visual','original_protection_visual').replace('original760/136','original228/132').replace('760','228').replace('136','132').replace('5250','830'),encoding='utf-8',newline='\n')
 changes={
 'native_skills.rs':[
  ('48 | 55 | 57 | 79 =>','48 | 55 | 57 | 67 | 68 | 79 =>'),
  ('43 | 48 | 55 | 79)','43 | 48 | 55 | 67 | 68 | 79)'),
  ('53|59|62|66=>1500','53|59|62|66|68=>1500')],
 'native_skill_selection.rs':[
  ('                        65 =>','                        67 => "#67 變形術（選取角色／自己，畫面內；消耗 1 顆魔法寶石）".into(),\n                        68 => "#68 聖結界（選取角色／自己，畫面內；消耗 2 顆魔法寶石，同血盟資格由伺服器判定）".into(),\n                        65 =>')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177|760)','matches!(cue.graphics,3943|2230|3944|230|2177|760|228)')],
 'native_login.rs':[
  ('    resurrection:super::native_resurrection61::Session,','    shape:super::native_shape67::Session,\n    resurrection:super::native_resurrection61::Session,'),
  ('            resurrection:Default::default(),','            shape:Default::default(),\n            resurrection:Default::default(),'),
  ('    pub(crate) fn resurrection_prompt(','''    pub(crate) fn shape_prompt(&self)->Option<&super::native_shape67::Prompt>{self.shape.current(&self.state,self.transport.snapshot().epoch)}
    pub(crate) fn answer_shape(&mut self,token:&super::native_shape67::Token,index:usize)->Result<()> {
        let body=self.shape.answer(&self.state,self.transport.snapshot().epoch,token,index)?;
        if let Err(error)=self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(body)){self.fail(format!("shape answer send: {error:#}"));return Err(error);} Ok(())
    }
    pub(crate) fn dismiss_shape(&mut self,token:&super::native_shape67::Token)->Result<()> {self.shape.dismiss(&self.state,self.transport.snapshot().epoch,token)}
    pub(crate) fn resurrection_prompt('''),
  ('        ensure!(self.world_tree_request.is_none(),"等待世界樹傳送回覆");\n        ensure!(!self.state.observer,"observer cannot cast");','        ensure!(self.shape_prompt().is_none(),"請先完成或關閉變形選單");\n        ensure!(self.world_tree_request.is_none(),"等待世界樹傳送回覆");\n        ensure!(!self.state.observer,"observer cannot cast");'),
  ('self.resurrection.retire();','self.resurrection.retire();self.shape.retire();'),
  ('        if self.resurrection.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}','        if self.resurrection.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}\n        if self.shape.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}'),
  ('        self.resurrection.synchronize(&self.state,self.transport.snapshot().epoch);','        self.resurrection.synchronize(&self.state,self.transport.snapshot().epoch);\n        self.shape.synchronize(&self.state,self.transport.snapshot().epoch);')],
 'native_login_window.rs':[
  ('const RESURRECT_YES:u16=138;','const SHAPE_LABEL:u16=320;\nconst SHAPE_OPTIONS:u16=218;\nconst SHAPE_APPLY:u16=140;\nconst SHAPE_CLOSE:u16=141;\nconst RESURRECT_YES:u16=138;'),
  ('    resurrection_token:Option<super::native_resurrection61::Token>,','    shape_token:Option<super::native_shape67::Token>,\n    shape_choice:Option<usize>,\n    resurrection_token:Option<super::native_resurrection61::Token>,'),
  ('            (RESURRECT_LABEL,w!("STATIC"),String::new(),0),','            (SHAPE_LABEL,w!("STATIC"),String::new(),0),\n            (SHAPE_OPTIONS,w!("COMBOBOX"),String::new(),CBS_DROPDOWNLIST as u32|WS_VSCROLL.0),\n            (SHAPE_APPLY,w!("BUTTON"),"套用變形".into(),0),\n            (SHAPE_CLOSE,w!("BUTTON"),"關閉選單".into(),0),\n            (RESURRECT_LABEL,w!("STATIC"),String::new(),0),'),
  ('            let visible = match *id {','''            let visible = match *id {
                SHAPE_LABEL|SHAPE_OPTIONS|SHAPE_APPLY|SHAPE_CLOSE=>self.world_view&&!self.result_view&&!self.closing&&self.controller.shape_prompt().is_some(),
                SKILL_LABEL|SKILLS|CAST_SKILL|SUMMON_OPTIONS|RESURRECT_LABEL|RESURRECT_YES|RESURRECT_NO|INVENTORY_TOGGLE|GROUND_TOGGLE if self.controller.shape_prompt().is_some()=>false,'''),
  ('            let _=MoveWindow(self.control(RESURRECT_LABEL)', '''            let shape_width=(area.right-px(304)).max(px(100));
            let _=MoveWindow(self.control(SHAPE_LABEL),px(20),area.bottom-px(132),(area.right-px(40)).max(px(240)),px(32),true);
            let _=MoveWindow(self.control(SHAPE_OPTIONS),px(20),area.bottom-px(90),shape_width,px(300),true);
            send(self.control(SHAPE_OPTIONS),CB_SETDROPPEDWIDTH,WPARAM(px(440) as usize),LPARAM(0));
            let _=MoveWindow(self.control(SHAPE_APPLY),(area.right-px(268)).max(px(120)),area.bottom-px(90),px(116),px(32),true);
            let _=MoveWindow(self.control(SHAPE_CLOSE),(area.right-px(140)).max(px(248)),area.bottom-px(90),px(116),px(32),true);
            let _=MoveWindow(self.control(RESURRECT_LABEL)'''),
  ('if self.controller.resurrection_prompt().is_some(){self.held.clear();self.held_attack.clear();return;}','if self.controller.resurrection_prompt().is_some()||self.controller.shape_prompt().is_some(){self.held.clear();self.held_attack.clear();return;}'),
  ('let show_summon=self.controller.resurrection_prompt().is_none()&&','let show_summon=self.controller.resurrection_prompt().is_none()&&self.controller.shape_prompt().is_none()&&'),
  ('    unsafe fn refresh_resurrection(&mut self) {',(T/'templates/native_shape67_window_refresh.rs').read_text('utf-8-sig')+'\n    unsafe fn refresh_resurrection(&mut self) {'),
  ('        self.refresh_resurrection();','        self.refresh_resurrection();\n        self.refresh_shape();'),
  ('    unsafe fn action(&mut self, id: u16) -> Result<()> {','    unsafe fn action(&mut self, id: u16) -> Result<()> {\n        ensure!(self.controller.shape_prompt().is_none()||matches!(id,RETURN|CANCEL),"請先完成或關閉變形選單");'),
  ('                WM_LBUTTONDOWN if ui.world_view && !ui.inventory_view && !ui.rental.open => {','                WM_LBUTTONDOWN if ui.controller.shape_prompt().is_some()=>{return LRESULT(0);}\n                WM_LBUTTONDOWN if ui.world_view && !ui.inventory_view && !ui.rental.open => {'),
  ('                WM_COMMAND if (wparam.0>>16)==BN_CLICKED as usize && lparam.0!=0',(T/'templates/native_shape67_window_commands.rs').read_text('utf-8-sig')+'\n                WM_COMMAND if (wparam.0>>16)==BN_CLICKED as usize && lparam.0!=0'),
  ('                resurrection_token:None,','                shape_token:None,shape_choice:None,\n                resurrection_token:None,'),
  ('                // Submit a received numeric NPC request from its quantity field.',(T/'templates/native_shape67_window_keys.rs').read_text('utf-8-sig')+'\n                // Submit a received numeric NPC request from its quantity field.')]
 }
 supplements={'native_skills.rs':[T/'templates/native_shape67_protection68_request_tests.rs'],'native_login.rs':[T/'templates/native_shape67_controller_tests.rs'],'native_skill_effects.rs':[render]};prepared={}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:
   count=2 if short=='native_login.rs' and a=='self.resurrection.retire();' else 1
   require(s.count(a)==count,'source anchor differs '+short+': '+a[:90]+' count='+str(s.count(a)));s=s.replace(a,b)
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig')
  prepared[short]=s
 module_template=T/'templates/native_shape67.rs';module_tests=T/'templates/native_shape67_tests.rs'
 module=module_template.read_text('utf-8-sig');require(module.count('// @STANDARD_MENU_FORMS@')==1,'menu marker differs');generated='pub(crate) const FORMS:&[Form]=&[\n'+''.join('    Form{action:'+json.dumps(r['selection'])+',minimum_level:'+r['min_level']+'},\n' for r in forms)+'];'
 module=module.replace('// @STANDARD_MENU_FORMS@',generated)+'\n'+module_tests.read_text('utf-8-sig')
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;previous=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n')
  overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(previous),predecessor_sha256=sha(previous),changes=changes[short],supplements=[dict(path=str(t),sha256=sha(t))for t in supplements.get(short,[])],repeated_replacement_counts={'self.resurrection.retire();':2} if short=='native_login.rs' else {})
  files[name]=sha(p);paths[name]=str(p)
 name=PREFIX+'native_shape67.rs';p=C/name;p.write_text(module,encoding='utf-8',newline='\n');files[name]=sha(p);paths[name]=str(p);modules={'native_shape67':dict(path=str(p),sha256=sha(p),template=str(module_template),template_sha256=sha(module_template),tests=str(module_tests),tests_sha256=sha(module_tests),generated_forms=generated)}
 s=prepared['native_skills.rs'];b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{67,68} and len(ids)==74,'scope exceeds67/68');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=74,missing_request_ids=sorted(set(old['missing_request_ids'])-{67,68}),missing_request_count=106)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts changed');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id'] in next_ids];cohort=Path(q['compile_run'])/'cohort.json'
 templates=[module_tests,*[p for rows in supplements.values() for p in rows]];cases=sum(p.read_text('utf-8-sig').count('#[test]')for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=')for p in templates);require((len(files),cases,ignored)==(40,16,1),'case/overlay count differs')
 pins={str(p):sha(p) for p in [Path(__file__).resolve(),module_template,module_tests,T/'templates/native_shape67_window_refresh.rs',T/'templates/native_shape67_window_commands.rs',T/'templates/native_shape67_window_keys.rs',*templates,inspection.parent/'standard-menu-forms.json']}
 contract=dict(schema='E149-shape67-protection68-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof.get('external_readonly_pins',{}),preparation_pins=pins,modules=modules,runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof.get('definitions'),next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=cases,new_resource_opt_in_cases=ignored,previous_written_deferred_cases=322,shape67_menu_or_request_integrated=True,protection68_request_integrated=True,formal_standard_menu_count=96,recipient_without_book_or_local_cast=True,local_close_not_remove=True,token_bound_native_commands=True,consume_before_send=True,server_random_without_control_authority_unchanged=True,LT_base_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=len(files),new_cases=cases,recognized=74,missing=106,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
