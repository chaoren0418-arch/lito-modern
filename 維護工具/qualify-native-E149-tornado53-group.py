"""Qualify compiler-only selector-only tornado53 and original S39 group source work and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-tornado53-group-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_area_fireball_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==24 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==3,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  base=Path(baseline['candidate_paths'].get(n,str(R/n)));needles=list(dict.fromkeys([str(base),base.as_posix()]));found=[needle for needle in needles if modern.count(needle)==1]
  require(candidate['base_files'][n]==sha(base) and len(found)==1 and sum(modern.count(needle) for needle in needles)==1,'normal module mapping differs');modern=modern.replace(found[0],(cp.parent/n).as_posix())
 require(c['paired_runtime_sources']==old['paired_runtime_sources'],'six nested module paths changed')
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond two runtime paths')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 require(c['paired_tests']==old['paired_tests'] and len(c['paired_tests'])==7,'retained seven integrations changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-tornado53-group-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 require(contract['damage_MP_and_radius_server_owned'] and contract['tests_executed']==0,'server authority scope changed')
 many='LiTo381-Modern/src/modern_lito/native_attack_many.rs';skills='LiTo381-Modern/src/modern_lito/native_skills.rs';selection='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])=={many,skills,selection},'runtime scope differs')
 template=T/'templates/native_tornado53_group_tests.rs';qa=template.read_text('utf-8-sig');require(sha(template)==contract['QA_template_sha256'] and len(re.findall(r'#\[test\]',qa))==9 and qa.count('#[ignore=')==1,'new QA count/source differs')
 for name,replacements in contract['source_changes'].items():
  source=Path(baseline['candidate_paths'].get(name,str(R/name))).read_text('utf-8-sig')
  for before,after in replacements:require(source.count(before)==1,'reviewed source anchor differs');source=source.replace(before,after)
  if name==many:source+=qa
  require(source==(cp.parent/name).read_text('utf-8-sig'),'runtime differs beyond reviewed source changes')
 require((contract['wire_prefix'],contract['S39_action'],contract['S39_type'],contract['effect_graphics'],contract['original_sound'],contract['original_frames'],contract['original_hit_source_frame'])==([20,6,4],18,0,758,125,20,1),'original tornado profile differs')
 require(sha(contract['original_area_contract'])==contract['original_area_contract_sha256'],'original area contract changed');prior_contract=read(contract['original_area_contract']);require(contract['raw_original_tornado_hex']==prior_contract['raw_original_effect_blocks_hex']['758'],'original758 definition changed')
 catalog_ref=w['current_E149_skill_catalog_development'];require(candidate['current_catalog_qualification']==catalog_ref and sha(catalog_ref['qualification'])==catalog_ref['qualification_sha256'],'current catalog context changed');cq=read(catalog_ref['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog facts changed');facts=read(cq['facts']);require(contract['original_definition']==next(row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id']==53),'tornado definition differs from primary class data')
 api=read(V/'native-E149-area2253053-contract/inspection.json');require(api['formal_target_sha256']==FORMAL and len(api['commands'])==2 and all(row['exit_code']==0 for row in api['commands']) and api['game_classes_instantiated']==0,'original javap inspection differs');check_pins(api['outputs'])
 dispatch_path=V/'native-E149-area2253053-contract/S39-original-dispatch.json';dispatch=read(dispatch_path);require(sha(dispatch['PE'])==dispatch['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','original native PE changed')
 import pefile
 pe=pefile.PE(str(dispatch['PE']),fast_load=True);last=0x4a6510
 for row in dispatch['instructions']:
  address=int(row['address'],16);raw=bytes.fromhex(row['hex']);require(address==last and raw==pe.get_data(address-pe.OPTIONAL_HEADER.ImageBase,len(raw)),'original native instruction bytes differ');last=address+len(raw)
 require(last>=0x4a6868 and not dispatch['PE_executed'],'original S39 dispatch range incomplete');pe.close()
 request=(cp.parent/skills).read_text('utf-8-sig');body=request[request.index('pub fn request('):request.index('pub fn reuse_millis(')];recognized=set()
 for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):recognized.update(map(int,re.findall(r'\d+',match[1])))
 ground=re.search(r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',request);item=re.search(r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',request);require(ground and item,'recognition syntax differs');recognized.update(map(int,re.findall(r'\d+',ground[1])));recognized.update(map(int,re.findall(r'\d+',item[1])))
 missing=sorted(set(cq['candidate_missing_request_ids'])-{53});require(len(recognized)==49 and recognized==set(read(cq['inspection'])['candidate_request']['all_request_ids'])|{53} and len(missing)==131,'source recognition changes exceed53')
 coverage=dict(request_source=str(cp.parent/skills),request_source_sha256=sha(cp.parent/skills),recognized_ids=sorted(recognized),recognized_count=49,catalog_literal_ids=180,missing_request_ids=missing,missing_request_count=131,source_recognition_only=True,learned_book_or_runtime_verified=False)
 group_path=Path(contract['native_group_contract']);require(sha(group_path)==contract['native_group_contract_sha256'],'native group contract changed');group=read(group_path);require(group['world_offsets']==contract['original_group_world_offsets']==[[2,2],[-2,-2],[-2,2],[2,-2]] and group['directions']==[3,7,5,1] and group['native_distance']==2 and group['three_child_copies_plus_original']==4 and not group['PE_executed'],'native four-copy evidence differs');require(sha(group['original_PE'])==group['original_PE_sha256'] and sha(group['source'])==group['source_sha256'],'native group original/source changed')
 superseded_run=V/'native-E149-tornado53-metadata/20261024-042549-725587';superseded=read(superseded_run/'cohort.json');superseded_record=read(superseded_run/'superseded-source.json');require(superseded_record['cohort_sha256']==sha(superseded_run/'cohort.json') and superseded_record['actual_terminal_tool_chunk']=='dd3507' and superseded_record['actual_exit_code']==0 and [row['exit_code'] for row in superseded['compiler_runs']]==[0]*18 and superseded['tests_executed']==0 and not superseded['outputs_executed'],'superseded original compiler evidence differs');check_pins(superseded['input_pins'])
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-tornado53-group.py','compile-native-E149-tornado53-group.py','qualify-native-E149-tornado53-group.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-tornado53-group-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_tornado53_selector_S39_four_copy_finite_audio_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=3,effective_overlay_files=24,new_tests_written=9,retained_skill_tests_written=51,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),original_definition=contract['original_definition'],wire_prefix=[20,6,4],S39_action=18,S39_type=0,effect_graphics=758,effect_frames=20,original_sound=125,original_hit_source_frame=1,original_group_copies=4,original_group_world_offsets=group['world_offsets'],native_group_contract=str(group_path),native_group_contract_sha256=sha(group_path),superseded_single_visual_cohort=str(superseded_run/'cohort.json'),superseded_single_visual_cohort_sha256=sha(superseded_run/'cohort.json'),superseded_terminal_tool_chunk='dd3507',superseded_actual_exit_code=0,old_source_success_transferred=False,current_catalog_ancestor=catalog_ref,current_source_coverage=coverage,remaining_area_skills=[22,25,30],original_nested_paths_preserved=6,
  source_pins={str(p):sha(p) for p in [*tools,T/'templates/native_tornado53_group_tests.rs',T/'native-E149-tornado53-group.md',T/'inspect-native-E149-tornado-group.py',group_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_area_fireball_qualification=w['current_E149_area_fireball_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; no current E149 gameplay/resource/audio/GUI execution','tornado53 source integration only; current E149 accepted/low-MP/reuse/resource/pixel/audio/GUI runs remain pending','twenty-four-file projection retains G06 Java and the original fireball resource chain','area22/25/30 remain required work; no older runtime qualification is transferred'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 for p in api['outputs']:archive['original-javap/'+Path(p).name]=Path(p)
 archive['original-S39-dispatch.json']=dispatch_path
 archive['original-native-group-contract.json']=group_path;archive['original-native-group-inspector.py']=Path(group['source'])
 archive['superseded/cohort.json']=superseded_run/'cohort.json';archive['superseded/superseded-source.json']=superseded_run/'superseded-source.json'
 oldcp=Path(superseded['candidate']);oldcandidate=read(oldcp);archive['superseded/candidate.json']=oldcp;archive['superseded/contract.json']=Path(oldcandidate['contract'])
 for name in oldcandidate['candidate_files']:archive['superseded/source/'+name]=oldcp.parent/name
 for name in ['prepare-native-E149-tornado53.py','compile-native-E149-tornado53.py','qualify-native-E149-tornado53.py','native-E149-tornado53.md','templates/native_tornado53_tests.rs']:archive['superseded/tools/'+name]=T/name
 for row in superseded['compiler_runs']:
  for path in row['logs']:archive['superseded/logs/'+row['profile']+'-'+row['kind']+'/'+Path(path).name]=Path(path)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('接通#53龍捲風的selector請求14 06 04／選單與S39 action18-type0-gfx758；server MP40基值／reuse1500／半徑4，HP／MP／傷害不在本地預測。'
  '原758單bank／20frame各3tick、source1 Sound125與Hit；原PE start758三child＋original四副本偏移(+2,+2)、(-2,-2)、(-2,+2)、(+2,-2)，原序批次預檢，一組四圖共用cursor／音效／target名單，沿S39一組效果／全target hit與incarnation、零target、history cursor／世代退役與原terminal dwell；125改用bank準備之原生same-ID音效群。'
  '三Runtime差分（九個待測案例含一個原資源前提），最新24檔十Runtime／十一Rust QA／三Java；兩版library／library-test／七既有skill整合共18metadata退出0，新增9與先前51案未跑，原單圖18metadata成功僅留舊來源／cohort，不移記修正版。'
  '候選請求識別僅由48加53成49；E149目錄180項剩131個無入口，這是來源識別，不是施法或GUI通過。'
  '#22／#25／#30仍待S39 class1／class5、首目標anchor與1804／1805分層／171時序，接著補請求／選單；G06仍是優先主線。'
  '未選入registry／套用／連結或執行PE／Java／測試／GUI／播放；正常1049輸入／所有PE、G06 binding／16案640次不變。使用者恢復測試後先元件／G06、當版完整library兩版、Release與完整GUI自行確認後才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_tornado53_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='tornado53_selector_S39_finite_audio_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'checkpoint before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent checkpoint writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_compilers=18,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
