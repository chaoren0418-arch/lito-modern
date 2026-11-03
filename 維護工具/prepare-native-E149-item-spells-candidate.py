"""Develop typed inventory spell targets and UI/controller flow; no tests."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,importlib.util,json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def replace(text,before,after,count=1):
 if text.count(before)!=count and '\n' in before and text.count(before.replace('\n','\r\n'))==count:
  before=before.replace('\n','\r\n');after=after.replace('\n','\r\n')
 require(text.count(before)==count,'source anchor differs: '+before[:90]);return text.replace(before,after)

def prepare(inspection):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle state required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 inspection=owned(inspection.resolve(strict=True));api=read(inspection);require(api['status']=='original_item_spell_bytecode_only' and api['formal_target_sha256']==FORMAL and all(x['exit_code']==0 for x in api['commands']),'original item spell API differs');check_pins(api['outputs'])
 helper_path=T/'prepare-native-E149-slow29-candidate.py';spec=importlib.util.spec_from_file_location('_item_spell_original_methods',helper_path);helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
 prior_ref=w['current_native_advance79_development'];require(sha(prior_ref['qualification'])==prior_ref['qualification_sha256'],'advance candidate qualification changed');prior_q=read(prior_ref['qualification']);prior=Path(prior_q['candidate']);require(sha(prior)==prior_q['candidate_sha256'],'advance candidate changed');base=read(prior);check_pins({str(prior.parent/n):h for n,h in base['candidate_files'].items()})
 contract_inspection=read(V/'native-E149-slow29-advance79-contract/20261024-002909-204866/inspection.json');check_pins(contract_inspection['original_disassembly_pins']);catalog=(V/'native-E149-slow29-advance79-contract/20261024-002909-204866/BattleSpellCatalog.stdout.txt');catalog_text=catalog.read_text('utf-8-sig');definitions={}
 for cls,key in [('ItemWeaponEnchantSpell','enchant-weapon'),('ItemArmorEnchantSpell','blessed-armor')]:
  record=(inspection.parent/('BattleSpellCatalog$'+cls+'.stdout.txt')).read_text('utf-8-sig');fields=re.findall(r'(?m)^  private final [^ ]+ (\w+);',record);end=catalog_text.index('// String '+key);start=catalog_text.rfind('new ',0,end);block=catalog_text[start:catalog_text.index('invokespecial',end)].replace('lconst_0','iconst_0').replace('lconst_1','iconst_1');values=helper.literals(block);require(len(fields)==len(values),'actual record component count differs');row=dict(zip(fields,values));definitions[str(row['skillId'])]=row
 require(set(definitions)=={'12','21'} and all(x['mpCost']==20 and x['reuseDelayMillis']==10 and x['durationSeconds']==1800 and x['invalidItemMessage']==79 and x['successMessage']==161 and x['expirationMessage']==308 for x in definitions.values()),'formal spell semantics differ')
 client=next(Path(n) for n in contract_inspection['original_disassembly_pins'] if Path(n).name=='ClientSession.stdout.txt');client_text=client.read_text('utf-8-sig');bodies={name:helper.section(client_text,' '+name+'(') for name in ['nativeMaterialItemWeaponEnchantBody','useItemArmorEnchantSpell']}
 for body in bodies.values():require(all(k in body for k in ['Protocol.objectSpellRequest:','Protocol$ObjectSpellRequest.targetObjectId:','Protocol.effectObject:','durationSeconds:']),'original item target/effect path differs')
 original_list=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(original_list)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','original list.spr changed');raw=original_list.read_bytes();effects=[]
 for gfx,n,ticks,sound in [(747,19,4,113),(748,20,3,137)]:
  matches=list(re.finditer(rb'(?m)^#'+str(gfx).encode()+rb'\b[^\r\n]*',raw));require(len(matches)==1,'original effect row absent/ambiguous');header=matches[0].group();match=re.search(rb'0[.]fire\(0 '+str(n).encode()+rb',([^)]*)\) 104[.]attr\(8\)$',header);require(match is not None,'original effect grammar differs');frames=re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',match[1]);require(len(frames)==n,'original effect frame count differs')
  expected=[(b'0',str(n-1).encode(),str(ticks).encode(),b'')]+[(b'0',str(i).encode(),str(ticks).encode(),str(sound).encode() if i==0 else b'') for i in range(n-1)];require(frames==expected and re.sub(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',b'',match[1]).strip()==b'','original finite sequence differs')
  effects.append(dict(graphics=gfx,header_bytes=list(header),frames=[dict(image=int(a),subframe=int(b),ticks=int(c),sound=int(d) if d else None) for a,b,c,d in frames]))
 original={n:(prior.parent/n).read_bytes() for n in base['candidate_files']};after=dict(original);base_files=dict(base['base_files']);extra_sources={}
 for registry,name in [('cancel-recovery','native_login.rs'),('keyboard','native_login_window.rs')]:
  ref=w['current_isolated_development']['candidates'][registry];path=Path(ref['path']);require(sha(path)==ref['sha256'],'selected controller/window predecessor changed');old=read(path);n='LiTo381-Modern/src/modern_lito/'+name;source=path.parent/n;require(sha(source)==old['candidate_files'][n],'predecessor source changed');original[n]=source.read_bytes();after[n]=original[n];base_files[n]=sha(source);extra_sources[n]=str(source)
 skills='LiTo381-Modern/src/modern_lito/native_skills.rs';selection='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';controller='LiTo381-Modern/src/modern_lito/native_login.rs';window='LiTo381-Modern/src/modern_lito/native_login_window.rs';finite='LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs'
 text=after[skills].decode('utf-8-sig');text=replace(text,'pub fn ground_request(', (T/'templates/native_item_spell_target.rs').read_text('utf-8')+'\npub fn ground_request(')
 text=replace(text,'impl Pending {','impl Pending {\n    /// Invalid-item79 is only a response in the inventory-spell domain.\n    pub fn observe_item(&self,world:&World,now:Instant)->Feedback {\n        let feedback=self.observe(world,now);\n        if matches!(feedback,Feedback::Pending|Feedback::NoResponse) &&world.message_count>self.messages\n            &&world.last_message.as_ref().is_some_and(|(code,_)|*code==79) {Feedback::Rejected(79)}else{feedback}\n    }')
 text=replace(text,'Self::Rejected(299) =>','Self::Rejected(79) => "伺服器拒絕此物品附魔",\n            Self::Rejected(299) =>');after[skills]=text.encode('utf-8')
 text=after[selection].decode('utf-8-sig');text=replace(text,'                        14 =>','                        12 => "#12 武器附魔（選取背包物品）".into(),\n                        21 => "#21 防具附魔（選取背包物品）".into(),\n                        14 =>')
 text=replace(text,'    pub fn choice(&self, connection:', '    pub fn selected_skill(&self,connection:u64,world:&World)->Result<u16> {\n        ensure!(self.owner.is_some()&&self.owner==Owner::current(connection,world),"技能畫面已失效");\n        let skill=self.selected.context("請先選擇已學會的技能")?;\n        ensure!(world.learned_skills.contains(&skill),"技能清單已更新");Ok(skill)\n    }\n    pub fn choice(&self, connection:');after[selection]=text.encode('utf-8')
 text=after[controller].decode('utf-8-sig');text=replace(text,'    skill_pending: Option<super::native_skills::Pending>,','    skill_pending: Option<super::native_skills::Pending>,\n    skill_item_pending: bool,');text=replace(text,'            skill_pending: None,','            skill_pending: None,\n            skill_item_pending: false,')
 text=replace(text,'self.cast_request(skill, target, None, frame)','self.cast_request(skill, target, None, None, frame)');text=replace(text,'self.cast_request(skill, None, Some(position), frame)','self.cast_request(skill, None, Some(position), None, frame)')
 text=replace(text,'    fn cast_request(', '    pub fn cast_item(&mut self,skill:u16,target:&super::native_skills::ItemTarget,frame:&super::native_world_scene::Frame)->Result<()> {\n        ensure!(self.item_pending.is_none()&&self.pickup_pending.is_none(),"等待目前物品操作回覆");\n        self.cast_request(skill,None,None,Some(target),frame)\n    }\n    fn cast_request(')
 text=replace(text,'        ground: Option<[u16; 2]>,\n        frame:', '        ground: Option<[u16; 2]>,\n        item: Option<&super::native_skills::ItemTarget>,\n        frame:')
 text=replace(text,'        let body = match ground {','        ensure!(item.is_none()||ground.is_none()&&target.is_none(),"混合施法目標");\n        let body = match ground {')
 text=replace(text,'            None => super::native_skills::request(&self.state.world, skill, target)?,','            None => if let Some(item)=item {super::native_skills::item_request(&self.state,self.transport.snapshot().epoch,skill,item)?}else{super::native_skills::request(&self.state.world, skill, target)?},')
 text=replace(text,'        self.skill_pending = Some(super::native_skills::Pending {','        self.skill_item_pending=item.is_some();\n        self.skill_pending = Some(super::native_skills::Pending {')
 text=replace(text,'if let Some(pending) = &self.skill_pending {\n            let feedback = if self.state.phase.active_world() {\n                pending.observe(&self.state.world, Instant::now())','if let Some(pending) = &self.skill_pending {\n            let feedback = if self.state.phase.active_world() {\n                if self.skill_item_pending {pending.observe_item(&self.state.world,Instant::now())}else{pending.observe(&self.state.world,Instant::now())}')
 text=text.replace('self.skill_pending = None;','self.skill_pending = None;self.skill_item_pending=false;').replace('self.skill_pending=None;','self.skill_pending=None;self.skill_item_pending=false;');after[controller]=text.encode('utf-8')
 text=after[window].decode('utf-8-sig');text=replace(text,'    inventory_choice: Option<super::native_item_actions::Choice>,','    inventory_choice: Option<super::native_item_actions::Choice>,\n    skill_item_target: Option<super::native_skills::ItemTarget>,');text=replace(text,'                inventory_choice: None,','                inventory_choice: None,\n                skill_item_target: None,')
 text=replace(text,'        let choice = if matches!(skill, 58 | 63) {','        if super::native_skills::item_targeted(skill) {\n            let epoch=self.controller.transport().snapshot().epoch;\n            ensure!(self.skills.selected_skill(epoch,&self.controller.snapshot().world)?==skill,"技能已變更");\n            ensure!(self.inventory_view&&!self.ground_view&&!self.closing,"請開啟背包並選取附魔物品");\n            let target=self.skill_item_target.clone().context("請重新選取要附魔的背包物品")?;\n            let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;\n            return self.controller.cast_item(skill,&target,frame);\n        }\n        let choice = if matches!(skill, 58 | 63) {')
 anchor='''        let choice = self
            .skills
            .choice_at(epoch, &snapshot.world, self.target, self.ground);'''
 replacement='''        if !self.inventory_view||self.ground_view||self.closing {self.skill_item_target=None;}
        if self.skill_item_target.as_ref().is_some_and(|t|t.validate(snapshot,epoch).is_err()) {
            self.skill_item_target=None;
            if self.skills.selected().is_some_and(super::native_skills::item_targeted) {
                self.inventory_choice=None;send(self.control(INVENTORY_LIST),LB_SETCURSEL,WPARAM(usize::MAX),LPARAM(0));
            }
        }
        let choice_valid=if self.skills.selected().is_some_and(super::native_skills::item_targeted) {
            self.inventory_view&&!self.ground_view&&!self.closing&&self.skills.selected_skill(epoch,&snapshot.world)
                .and_then(|skill|self.skill_item_target.as_ref().context("請選取背包物品")
                    .and_then(|item|super::native_skills::item_request(snapshot,epoch,skill,item))).is_ok()
        }else{self.skills.choice_at(epoch,&snapshot.world,self.target,self.ground).is_ok()};'''
 text=replace(text,anchor,replacement);text=replace(text,'            && choice.is_ok()','            && choice_valid')
 text=replace(text,'}else{self.skills.choice_at(epoch,&snapshot.world,self.target,self.ground).is_ok()};','}else{!self.inventory_view&&self.skills.choice_at(epoch,&snapshot.world,self.target,self.ground).is_ok()};')
 text=replace(text,'SKILLS | CAST_SKILL => self.world_view && !self.result_view && !self.inventory_view,','SKILLS | CAST_SKILL => self.world_view && !self.result_view && (!self.inventory_view||!self.ground_view),')
 text=replace(text,'SKILL_LABEL => self.world_view && !self.result_view && !self.inventory_view && !self.rental.open,','SKILL_LABEL => self.world_view && !self.result_view && (!self.inventory_view||!self.ground_view) && !self.rental.open,')
 text=replace(text,'self.control(ITEM_FEEDBACK),\n                px(16),\n                area.bottom - px(200),','self.control(ITEM_FEEDBACK),\n                px(16),\n                area.bottom - px(if self.inventory_view&&!self.ground_view{240}else{200}),')
 text=replace(text,'self.control(INVENTORY_LIST),\n                px(16),\n                area.bottom - px(175),\n                area.right - px(32),\n                px(120),','self.control(INVENTORY_LIST),\n                px(16),\n                area.bottom - px(if self.inventory_view&&!self.ground_view{215}else{175}),\n                area.right - px(32),\n                px(if self.inventory_view&&!self.ground_view{110}else{120}),')
 text=replace(text,'if self.inventory_view { 286 } else if self.result_view','if self.inventory_view {if self.ground_view{286}else{326}} else if self.result_view')
 text=replace(text,'    unsafe fn item_choice(&self)', '    unsafe fn sync_skill_item_target(&mut self) {\n        self.skill_item_target=if self.inventory_view&&!self.ground_view&&!self.closing {\n            self.inventory_choice.as_ref().and_then(|choice|super::native_skills::ItemTarget::capture(\n                self.controller.snapshot(),self.controller.transport().snapshot().epoch,choice.item.id).ok())\n        }else{None};\n    }\n    unsafe fn item_choice(&self)')
 anchor='''                let choice = self.skills.choice_at(
                    self.controller.transport().snapshot().epoch,
                    &self.controller.snapshot().world,
                    self.target,
                    self.ground,
                )?;
                self.cast_skill(choice.skill)?;'''
 text=replace(text,anchor,'                let skill=self.skills.selected_skill(self.controller.transport().snapshot().epoch,&self.controller.snapshot().world)?;\n                self.cast_skill(skill)?;')
 text=replace(text,'                    ui.sync_item_choice();\n                    ui.refresh_item_actions();','                    ui.sync_item_choice();\n                    ui.sync_skill_item_target();\n                    ui.refresh_item_actions();\n                    ui.refresh_skills();');after[window]=text.encode('utf-8')
 text=after[finite].decode('utf-8-sig');text=replace(text,'6342,752];','6342,752,747,748];');entries=''
 for effect in effects:
  entries+='Spec{sprite:'+str(effect['graphics'])+',bank:'+str(effect['graphics'])+',header:&['+','.join(map(str,effect['header_bytes']))+'],attrs:&[(104,&[56])],frames:&['+','.join('(0,'+str(f['subframe'])+','+str(f['ticks'])+','+('&['+str(f['sound'])+']' if f['sound'] is not None else '&[]')+')' for f in effect['frames'])+']},\n'
 text=replace(text,'];\n\npub fn qualify',entries+'];\n\npub fn qualify');after[finite]=text.encode('utf-8')
 test='LiTo381-Modern/tests/native_item_spells.rs';require(not (R/test).exists(),'new QA target exists');after[test]=(T/'templates/native_item_spells.rs').read_bytes()
 out=owned(T/'candidates'/('native-E149-item-spells-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir()
 contract=dict(formal_target_sha256=FORMAL,API_inspection=str(inspection),API_inspection_sha256=sha(inspection),original_definitions=definitions,original_ClientSession=str(client),original_ClientSession_sha256=sha(client),original_handlers=bodies,original_catalog=str(catalog),original_catalog_sha256=sha(catalog),original_list=str(original_list),original_list_sha256=sha(original_list),effects=effects,
  wire='C14 01/03 or02/04 + owned inventory object u32 LE; separate from actor and ground forms',authority='inventory owner+full item+revision+map+InputStamp; actual row selection arms target; refresh never rearms',eligibility_cost_effect_and_1800_seconds_server_owned=True,invalid79_feedback_scoped_to_item_spell=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract))
 for n,blob in after.items():
  target=owned(out/n,out);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(blob)
  if blob!=original.get(n):(out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original.get(n,b'').decode('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='predecessor/'+n,tofile='item-spell-candidate/'+n)),encoding='utf-8')
 for name,files in [('before.zip',original),('candidate.zip',after)]:
  with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
   for n,blob in files.items():z.writestr(n,blob)
  with zipfile.ZipFile(out/name) as z:require(all(z.read(n)==blob for n,blob in files.items()),'archive differs')
 runtime=list(base['changed_runtime_files'])+[controller,window];record=dict(status='unselected_item_spell_request_UI_effect_development',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_advance79_candidate=str(prior),predecessor_advance79_candidate_sha256=sha(prior),predecessor_blind20_candidate=base['predecessor_blind20_candidate'],extra_graph_base_sources=extra_sources,contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),candidate_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in after.items()},base_files=base_files,immediate_predecessor_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in original.items()},changed_runtime_files=runtime,new_tests_written=8,retained_skill_tests_written=18,tests_executed=0,GUI_verified=False,PEs_executed=False,production_applied=False,registered_overlay_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(candidate=str(out/'candidate.json'),runtime_files=len(runtime),candidate_files=len(after),new_tests_written=8,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);a=p.parse_args();prepare(a.inspection)
