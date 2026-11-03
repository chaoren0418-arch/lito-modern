"""Prepare #61 corpse intent, dead-life consent UI, and original finite visuals."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-resurrection61-poll-20261024-123400';PREFIX='LiTo381-Modern/src/modern_lito/'
def prepare():
 w=read(R/'工作佇列.json');ref=w['current_E149_blizzard59_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);base=read(q['candidate']);require(sha(q['candidate'])==q['candidate_sha256'],'prior candidate differs');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);inspection=V/'native-E149-resurrection61-inspection/20261024-122925-908479/inspection.json';proof=read(inspection);check_pins(proof['source_pins']);require(not C.exists(),'candidate exists');C.mkdir()
 nums=lambda a:'['+','.join(map(str,a))+']';specs=''
 for gfx,attrs in [(3944,[(102,b'0'),(104,b'8'),(109,b'1 -1')]),(230,[(104,b'8')])]:
  raw=Path(proof['raw_definitions'][str(gfx)]['path']).read_bytes().rstrip(b'\r\n');frames=[(int(a),int(f),int(t),[] if not s else [int(s)]) for a,f,t,s in re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',raw)];specs+='Spec{sprite:'+str(gfx)+',bank:'+str(gfx)+',header:&'+nums(raw.splitlines()[0])+',attrs:&['+','.join('('+str(k)+',&'+nums(b)+')' for k,b in attrs)+'],frames:&['+','.join('('+','.join([str(a),str(f),str(t),'&'+nums(s)])+')' for a,f,t,s in frames)+']},\n'
 controller_api='''    pub(crate) fn resurrection_prompt(&self)->Option<&super::native_resurrection61::Prompt>{self.resurrection.current(&self.state,self.transport.snapshot().epoch)}
    pub(crate) fn answer_resurrection(&mut self,token:&super::native_resurrection61::Token,yes:bool)->Result<()> {
        let body=self.resurrection.answer(&self.state,self.transport.snapshot().epoch,token,yes)?;
        if let Some(body)=body {if let Err(error)=self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(body)){self.fail(format!("resurrection answer send: {error:#}"));return Err(error);}}
        Ok(())
    }
'''
 ui_refresh='''    unsafe fn refresh_resurrection(&mut self) {
        let prompt=self.controller.resurrection_prompt().cloned();let token=prompt.as_ref().map(|p|p.token());
        if token!=self.resurrection_token {self.held.clear();self.held_attack.clear();if token.is_some(){self.inventory_view=false;self.ground_view=false;self.rental.open=false;self.target=None;self.ground=None;self.skill_item_target=None;self.summon_choice=None;}self.resurrection_token=token;self.layout(self.parent,false);}
        if let Some(p)=prompt {
            let t=p.token();let _=SetWindowLongPtrW(self.control(RESURRECT_YES),GWLP_ID,isize::from(t.command_id(true)));let _=SetWindowLongPtrW(self.control(RESURRECT_NO),GWLP_ID,isize::from(t.command_id(false)));
            self.text(RESURRECT_LABEL,if p.waiting(){"已同意，等待伺服器復活結果；可放棄等待。"}else{"收到復活詢問。是否同意？"});
            self.text(RESURRECT_NO,if p.waiting(){"放棄等待"}else{"不同意"});let _=EnableWindow(self.control(RESURRECT_YES),!p.waiting()&&!self.closing);let _=EnableWindow(self.control(RESURRECT_NO),!self.closing);
        }
    }
'''
 ui_command='''                WM_COMMAND if (wparam.0>>16)==BN_CLICKED as usize && lparam.0!=0
                    && [ui.control(RESURRECT_YES).0 as isize,ui.control(RESURRECT_NO).0 as isize].contains(&lparam.0) => {
                    let yes=lparam.0==ui.control(RESURRECT_YES).0 as isize;
                    if let Some(token)=ui.resurrection_token.clone().filter(|t|t.command_id(yes)==(wparam.0&0xffff)as u16) {
                        ui.held.clear();ui.held_attack.clear();if let Err(e)=ui.controller.answer_resurrection(&token,yes){ui.text(STATUS,&format!("{e:#}"));}
                    }
                    ui.refresh();return LRESULT(0);
                }
'''
 changes={
 'native_skills.rs':[('        41 => {','        41 | 61 => {')],
 'native_skill_selection.rs':[('if skill!=41 {','if !matches!(skill,41|61) {'),('                        41 =>','                        61 => "#61 復活術（選取屍體，畫面內；玩家須同意）".into(),\n                        41 =>')],
 'native_world_scene.rs':[('if skill==Some(41)&&!world.learned_skills.contains(&41){return None;}','if skill.is_some_and(|s|matches!(s,41|61)&&!world.learned_skills.contains(&s)){return None;}')],
 'native_additional_finite_effects.rs':[('3943,2230];','3943,2230,3944,230];'),('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+specs),('spec.sprite==3943','matches!(spec.sprite,3943|3944)')],
 'native_world.rs':[('if graphics==3943 {','if matches!(graphics,3943|3944) {'),('                    target,\n                    position,','                    target:if matches!(graphics,3944|230){None}else{target},\n                    position,')],
 'native_skill_effects.rs':[('if cue.graphics==3943 {','if matches!(cue.graphics,3943|3944) {'),('matches!(cue.graphics,3943|2230)','matches!(cue.graphics,3943|2230|3944|230)')],
 'native_login.rs':[('pub struct LoginController {','pub struct LoginController {\n    resurrection:super::native_resurrection61::Session,'),('            teleport: None,','            resurrection:Default::default(),\n            teleport: None,'),('    pub fn cast(\n',controller_api+'    pub fn cast(\n'),('    pub fn confirm_death(&mut self, frame: &super::native_world_scene::Frame) -> Result<()> {','    pub fn confirm_death(&mut self, frame: &super::native_world_scene::Frame) -> Result<()> {\n        ensure!(self.resurrection_prompt().is_none(),"請先處理復活詢問或放棄等待");'),('        let environment_packet = self.state.environment.accept','        if self.resurrection.receive(&self.state,self.transport.snapshot().epoch,body)?{return Ok(());}\n        let environment_packet = self.state.environment.accept'),('        if let Some(epoch)=self.state.environment.epoch {','        self.resurrection.synchronize(&self.state,self.transport.snapshot().epoch);\n        if let Some(epoch)=self.state.environment.epoch {'),('    fn interrupt_world_actions(&mut self) {','    fn interrupt_world_actions(&mut self) {\n        self.resurrection.retire();'),('        self.transport.start(peer, limits)?;','        self.transport.start(peer, limits)?;\n        self.resurrection.retire();')],
 'native_login_window.rs':[
 ('const SUMMON_OPTIONS:u16=217;','const SUMMON_OPTIONS:u16=217;\nconst RESURRECT_YES:u16=138;\nconst RESURRECT_NO:u16=139;\nconst RESURRECT_LABEL:u16=319;'),
 ('struct Ui {','struct Ui {\n    resurrection_token:Option<super::native_resurrection61::Token>,'),
 ('                skill_item_target: None,','                resurrection_token:None,\n                skill_item_target: None,'),
 ('            (SUMMON_OPTIONS,w!("COMBOBOX"),','            (RESURRECT_LABEL,w!("STATIC"),String::new(),0),\n            (RESURRECT_YES,w!("BUTTON"),"同意復活".into(),0),\n            (RESURRECT_NO,w!("BUTTON"),"不同意".into(),0),\n            (SUMMON_OPTIONS,w!("COMBOBOX"),'),
 ('                SKILL_LABEL =>','                RESURRECT_LABEL|RESURRECT_YES|RESURRECT_NO=>self.world_view&&!self.result_view&&!self.closing&&self.controller.resurrection_prompt().is_some(),\n                SKILL_LABEL|SKILLS|CAST_SKILL|SUMMON_OPTIONS if self.controller.resurrection_prompt().is_some()=>false,\n                SKILL_LABEL =>'),
 ('            let skill_width=(area.right-px(208)).max(px(80));','            let _=MoveWindow(self.control(RESURRECT_LABEL),px(20),area.bottom-px(132),(area.right-px(40)).max(px(240)),px(32),true);\n            let _=MoveWindow(self.control(RESURRECT_YES),px(64),area.bottom-px(90),px(116),px(32),true);\n            let _=MoveWindow(self.control(RESURRECT_NO),px(196),area.bottom-px(90),px(116),px(32),true);\n            let skill_width=(area.right-px(208)).max(px(80));'),
 ('        if self.controller.snapshot().world.death_confirmation_pending {','        if self.controller.resurrection_prompt().is_some(){self.held.clear();self.held_attack.clear();return;}\n        if self.controller.snapshot().world.death_confirmation_pending {'),
 ('    unsafe fn refresh(&mut self) {',ui_refresh+'    unsafe fn refresh(&mut self) {\n        self.refresh_resurrection();'),
 ('let show_summon=self.world_view&&!self.result_view','let show_summon=self.controller.resurrection_prompt().is_none()&&self.world_view&&!self.result_view'),
 ('                WM_COMMAND if (wparam.0 & 0xffff)==rental::LIST',ui_command+'                WM_COMMAND if (wparam.0 & 0xffff)==rental::LIST')]
 }
 supplements={'native_skills.rs':[T/'templates/native_resurrection61_request_tests.rs'],'native_login.rs':[T/'templates/native_resurrection61_controller_tests.rs'],'native_additional_finite_effects.rs':[T/'templates/native_resurrection61_definition_tests.rs'],'native_skill_effects.rs':[T/'templates/native_resurrection61_render_tests.rs']};overrides={}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:
   expected=2 if short=='native_additional_finite_effects.rs' and a=='spec.sprite==3943' else 1;require(s.count(a)==expected,'preflight anchor differs '+short+': '+a[:90]+' count='+str(s.count(a)));s=s.replace(a,b)
 for short,delta in changes.items():
  name=PREFIX+short;old=Path(paths[name]);s=old.read_text('utf-8-sig')
  for a,b in delta:
   count=s.count(a);expected=2 if short=='native_additional_finite_effects.rs' and a=='spec.sprite==3943' else 1;require(count==expected,'source anchor differs '+short+': '+a[:100]+' count='+str(count));s=s.replace(a,b)
  extra=[]
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig');extra.append(dict(path=str(p),sha256=sha(p)))
  p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');files[name]=sha(p);paths[name]=str(p);overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(old),predecessor_sha256=sha(old),changes=delta,supplements=extra)
 template=T/'templates/native_resurrection61.rs';p=C/(PREFIX+'native_resurrection61.rs');p.write_bytes(template.read_bytes());modules={'native_resurrection61':dict(path=str(p),sha256=sha(p),template=str(template),template_sha256=sha(template),generated_suffix='',supplements=[])};files[PREFIX+p.name]=sha(p);paths[PREFIX+p.name]=str(p)
 s=Path(paths[PREFIX+'native_skills.rs']).read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'source recognition differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{61} and len(ids)==68,'scope exceeds61');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=68,missing_request_ids=sorted(set(old['missing_request_ids'])-{61}),missing_request_count=112);cq=read(w['current_E149_skill_catalog_development']['qualification']);facts=read(cq['facts']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog differs');next_ids=cov['missing_request_ids'][:2];next_defs=[x['definitions'][0]['core'] for x in facts['definitions_by_id'] if x['skill_id'] in next_ids];cohort=Path(q['compile_run'])/'cohort.json';templates=[template,*[p for rows in supplements.values() for p in rows]];cases=sum(p.read_text('utf-8-sig').count('#[test]') for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=') for p in templates);require((len(files),cases,ignored)==(36,15,1),'projection/cases differ')
 contract=dict(schema='E149-resurrection61-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),integration_predecessor_cohort=str(cohort),integration_predecessor_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],modules=modules,runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],generated_specs=specs,next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=15,new_resource_opt_in_cases=1,previous_written_deferred_cases=252,source_visuals_detached_before_S11=True,question_command_ids_nonwrapping=True,question_answers_explicit_only=True,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);(C/'contract.json').write_bytes(encoded(contract));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),effective_files=len(files),new_cases=cases,recognized_requests=68,remaining=112,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
