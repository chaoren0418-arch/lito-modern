"""Qualify compiler-only fireball class5 clock and exact-frame APIs and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-fireball25-lifecycle-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_fireball_clock_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==27 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==4,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  base=Path(baseline['candidate_paths'].get(n,str(R/n)));needles=list(dict.fromkeys([str(base),base.as_posix()]));found=[needle for needle in needles if modern.count(needle)==1]
  require(candidate['base_files'][n]==sha(base) and len(found)==1 and sum(modern.count(needle) for needle in needles)==1,'normal module mapping differs');modern=modern.replace(found[0],(cp.parent/n).as_posix())
 require(c['paired_runtime_sources']==old['paired_runtime_sources'],'six nested module paths changed')
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond the reviewed runtime path')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 require(c['paired_tests']==old['paired_tests'] and len(c['paired_tests'])==7,'retained seven integrations changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-fireball25-lifecycle-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 names={"LiTo381-Modern/src/modern_lito/"+n for n in ['native_skills.rs','native_skill_selection.rs','native_attack_many.rs','native_audio_delivery.rs']};require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])==names,'runtime scope differs')
 templates=contract['QA_templates'];require(set(templates)=={n for n in names if n.endswith(('native_attack_many.rs','native_audio_delivery.rs'))},'QA module scope differs')
 tests=0;ignored=0
 for n in names:
  source=Path(baseline['candidate_paths'].get(n,str(R/n))).read_text('utf-8-sig')
  for before,after in contract['source_changes'][n]:require(source.count(before)==1,'source anchor differs');source=source.replace(before,after)
  if n in templates:
   require(sha(templates[n]['path'])==templates[n]['sha256'],'QA template changed');qa=Path(templates[n]['path']).read_text('utf-8-sig');tests+=len(re.findall(r'#\[test\]',qa));ignored+=qa.count('#[ignore=');source+=qa
  require(source==(cp.parent/n).read_text('utf-8-sig'),'runtime differs beyond reviewed changes')
 require(tests==12 and ignored==1 and contract['wire_prefix']==[20,3,0] and contract['flight_source_cycle']==[1,2,3,4,0] and contract['movement_ms']==60 and contract['flight_graphics']==171 and contract['secondary_graphics']==218 and contract['request_S39_world_integration'] and contract['audio_callback_occurrence_identity'],'source/QA scope differs')
 native_path=Path(contract['native_snapshot_contract']);require(sha(native_path)==contract['native_snapshot_contract_sha256'],'native contract changed');native=read(native_path)
 require(sha(native['PE'])==native['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9' and not native['PE_executed'] and native['tests_executed']==0,'native source changed')
 import pefile
 pe=pefile.PE(native['PE'],fast_load=True);instructions={}
 for block in native['ranges'].values():
  raw=pe.get_data(block['start']-pe.OPTIONAL_HEADER.ImageBase,block['end']-block['start']);require(hashlib.sha256(raw).hexdigest().upper()==block['sha256'],'native range changed');at=block['start']
  for row in block['instructions']:
   data=bytes.fromhex(row['hex']);require(row['address']==at and data==pe.get_data(at-pe.OPTIONAL_HEADER.ImageBase,len(data)),'native instruction bytes changed');instructions[at]=row;at+=len(data)
  require(at==block['end'],'native range incomplete')
 pe.close()
 for address,mnemonic,operands in [(0x42f156,'xor','cl, cl'),(0x42f165,'mov','byte ptr [esi + 0x13], cl'),(0x42f60a,'call','0x42f300'),(0x42f4f0,'mov','ecx, 0x3c'),(0x42f4cb,'call','dword ptr [edx + 4]'),(0x42f500,'call','0x4e5b50'),(0x4e5b58,'inc','bl'),(0x4e5b7d,'cmp','dl, byte ptr [edi + eax*4 + 1]'),(0x4e5bc9,'mov','byte ptr [ecx + 0x13], dl'),(0x4e5bcc,'call','0x4e5c00'),(0x4e5be1,'call','0x4e5c00')]:
  require(instructions[address]['mnemonic']==mnemonic and instructions[address]['operands']==operands,'native callback anchor differs')
 require(native['flight_frame_cycle']==[1,2,3,4,0] and native['flight_terminal_frame_not_held'],'cyclic frame evidence differs')
 require(native['movement_callback_milliseconds']==60 and native['movement_starts_immediately'] and native['late_movement_callback_reschedules_from_now'] and native['arrival_secondary_start_precedes_flight_animation_advance'],'native timing evidence differs')
 previous=read(candidate['previous_skill_qualification']['qualification']);previous_coverage=previous['current_source_coverage'];require(previous_coverage['recognized_count']==51 and previous_coverage['missing_request_count']==129,'previous request coverage changed')
 catref=w['current_E149_skill_catalog_development'];require(candidate['current_catalog_qualification']==catref and sha(catref['qualification'])==catref['qualification_sha256'],'catalog context changed');catalog=read(catref['qualification']);require(sha(catalog['facts'])==catalog['facts_sha256'],'catalog facts changed');facts=read(catalog['facts']);require(contract['definition']==next(row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id']==25),'formal #25 definition changed')
 api_path=V/'native-E149-area2253053-contract/inspection.json';api=read(api_path);require(api['formal_target_sha256']==FORMAL and all(row['exit_code']==0 for row in api['commands']) and api['game_classes_instantiated']==0,'original API scope differs');check_pins(api['outputs'])
 request=cp.parent/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=request.read_text('utf-8-sig');body=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];recognized=set()
 for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):recognized.update(map(int,re.findall(r'\d+',match[1])))
 ground=re.search(r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',text);item=re.search(r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',text);require(ground and item,'request parser shape changed');recognized.update(map(int,re.findall(r'\d+',ground[1])));recognized.update(map(int,re.findall(r'\d+',item[1])));missing=sorted(set(previous_coverage['missing_request_ids'])-{25});require(recognized==set(previous_coverage['recognized_ids'])|{25} and len(recognized)==52 and len(missing)==128,'request changes exceed #25')
 coverage=dict(request_source=str(request),request_source_sha256=sha(request),recognized_ids=sorted(recognized),recognized_count=52,catalog_literal_ids=180,missing_request_ids=missing,missing_request_count=128,source_recognition_only=True,learned_book_or_runtime_verified=False)
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 base_path=V/'native-E149-graphics-base-contract/20261024-060132-817609/contract.json';base_proof=read(base_path);require(base_proof['embedded_declared_record_count']==5894 and base_proof['shared_catalog_header']=='11297 0 41210' and not base_proof['modern_graphics_binding_verified'] and not base_proof['embedded_actual_split_record_count_proven'] and not base_proof['PE_executed'] and base_proof['tests_executed']==0 and sha(base_proof['inspection_source'])==base_proof['inspection_source_sha256'],'next graphics base evidence changed')
 base_pe=pefile.PE(base_proof['PE'],fast_load=True)
 for block in base_proof['ranges'].values():
  raw_bytes=base_pe.get_data(block['start']-base_pe.OPTIONAL_HEADER.ImageBase,block['end']-block['start']);require(hashlib.sha256(raw_bytes).hexdigest().upper()==block['sha256'],'base loader range changed');at=block['start']
  for instruction in block['instructions']:
   data=bytes.fromhex(instruction['hex']);require(instruction['address']==at and data==base_pe.get_data(at-base_pe.OPTIONAL_HEADER.ImageBase,len(data)),'base loader instruction changed');at+=len(data)
  require(at==block['end'],'base loader range incomplete')
 for row in base_proof['data'].values():
  data=bytes.fromhex(row['hex']);require(data==base_pe.get_data(row['address']-base_pe.OPTIONAL_HEADER.ImageBase,len(data)) and hashlib.sha256(data).hexdigest().upper()==row['sha256'],'base loader data changed')
 base_pe.close()
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-fireball25-lifecycle.py','compile-native-E149-fireball25-lifecycle.py','qualify-native-E149-fireball25.py','inspect-native-E149-graphics-base.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-fireball25-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_fireball25_request_S39_callback_audio_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=4,effective_overlay_files=27,new_tests_written=12,retained_skill_tests_written=82,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),native_snapshot_contract=str(native_path),native_snapshot_contract_sha256=sha(native_path),current_source_coverage=coverage,request_S39_world_integration=True,audio_callback_occurrence_identity=True,wire_prefix=[20,3,0],definition=contract['definition'],next_graphics_base_contract=str(base_path),next_graphics_base_contract_sha256=sha(base_path),remaining_native_hurt_guard_dynamic_base=True,normal_retirement_does_not_poison_loading=True,superseded_lifecycle_error_cohort=str(V/'native-E149-fireball25-metadata/20261024-055145-882048/cohort.json'),superseded_lifecycle_error_cohort_sha256=sha(V/'native-E149-fireball25-metadata/20261024-055145-882048/cohort.json'),old_source_success_transferred=False,flight_source_cycle=[1,2,3,4,0],original_nested_paths_preserved=6,
  source_pins={str(p):sha(p) for p in [*tools,*[T/'templates'/n for n in ['native_fireball25_lifecycle_tests.rs','native_fireball25_audio_tests.rs','native_fireball25_lifecycle_many.rs']],api_path,*map(Path,api['outputs']),T/'native-E149-fireball25-lifecycle.md',base_path,native_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_fireball_clock_qualification=w['current_E149_fireball_clock_development'],current_catalog_qualification=catref,
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; twelve new and eighty-two preceding written cases remain unrun','request/menu/S39/flight/recipients/audio/draw/history source integration only; no current E149 cast/resource/audio/GUI qualification','dynamic original graphics-base-relative nine-entry no-hurt family remains unresolved; known literal gfx and shared animator guards are applied','original class4/ice rational timelines require separate native branch audit'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'source-pins/'+Path(p).relative_to(R).as_posix():Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 archive['original-native-class5.json']=native_path
 sup_path=Path(record['superseded_lifecycle_error_cohort']);sup=read(sup_path);require(sup['status']=='eighteen_metadata_compilers_complete_no_tests' and [row['exit_code'] for row in sup['compiler_runs']]==[0]*18,'superseded compile changed');check_pins(sup['input_pins']);sup_cp=Path(sup['candidate']);sup_candidate=read(sup_cp)
 archive['superseded-lifecycle/cohort.json']=sup_path;archive['superseded-lifecycle/candidate.json']=sup_cp;archive['superseded-lifecycle/contract.json']=Path(sup_candidate['contract']);archive['superseded-lifecycle/superseded-source.json']=sup_path.with_name('superseded-source.json')
 for n in sup_candidate['candidate_files']:archive['superseded-lifecycle/candidate/'+n]=sup_cp.parent/n
 for n in ['prepare-native-E149-fireball25.py','compile-native-E149-fireball25.py','native-E149-fireball25.md']:archive['superseded-lifecycle/tools/'+n]=T/n
 for n in ['native_fireball25_many.rs','native_fireball25_tests.rs','native_fireball25_audio_tests.rs']:archive['superseded-lifecycle/templates/'+n]=T/'templates'/n
 for row in sup['compiler_runs']:
  check_pins(row['logs']);require(sha(row['product'])==row['product_sha256'],'superseded metadata changed')
  for log in row['logs']:archive['superseded-lifecycle/'+row['profile']+'-'+row['kind']+'/'+Path(log).name]=Path(log)
 archive['previous-clock-qualification.json']=Path(candidate['previous_skill_qualification']['qualification'])
 for p in api['outputs']:archive['original-javap/'+Path(p).name]=Path(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('接通#25火球術來源：C14 14/03/00接角色ID與x/y、range6／reuse500與選單，成本／命中／傷害仍由E149決定；S39只增action18-type8-171。'
  '接收保存首target或wire一步fallback，launch只重查原同incarnation首target後固定60ms循環flight；arrival逐名單解析同incarnation，每個含miss／dead／duplicate均建218凍結anchor，hurt限有效hit再由animator守衛。'
  '每個99有獨立payload並先於當callback98；callback occurrence history保留source1循環重入並拒絕同occurrence重播，原finite API與scope／latch／same-ID政策保留。整組draw預檢所有anchor／queue、history及caster／generation退役已接；正常退役與超長路徑省略不寫持續load_error，初版錯誤來源／cohort保留、不移記成功。'
  '四Runtime差分成27檔十三Runtime／十一Rust QA／三Java；Debug／Release library、library-test與七skill整合18metadata退出0，新12＋前82案全部未跑。請求識別52／180，尚128無入口。'
  '原動態graphics base相對九項禁止hurt家族仍待綁定，已追到內嵌宣告5894、共享表頭11297不可互代，仍須還原實際分隔數及圖庫對應，不能拿原PE的未初始化零值猜gfx；class4／冰箭rational時序仍需原分支稽核。'
  '未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放，正常1049輸入／所有PE、registry與G06 binding／16案640次不變。使用者使用電腦中，測試及桌面操作持續暫緩。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_fireball25_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='fireball25_request_S39_fanout_callback_audio_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
