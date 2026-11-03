"""Qualify compiler-only original area fireball class5 resources source work and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-area-fireball-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_object374448_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==23 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==1,'merged source projection differs')
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
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-area-fireball-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 require(not contract['request_activation_added'] and not contract['S39_packet_activation_added'] and not contract['HP_MP_or_damage_prediction_added'] and contract['tests_executed']==0,'resource-only scope changed')
 name='LiTo381-Modern/src/modern_lito/light_arrow_effect.rs';source=(R/name).read_text('utf-8-sig');require(len(contract['original_source_changes'])==6,'source difference count changed')
 for before,after in contract['original_source_changes']:require(source.count(before)==1,'reviewed source anchor differs');source=source.replace(before,after)
 template=T/'templates/native_area_fireball_tests.rs';qa=template.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',qa))==5 and qa.count('#[ignore=')==2,'new resource QA count differs');require(source+qa==(cp.parent/name).read_text('utf-8-sig'),'source differs beyond reviewed profile and QA')
 require((contract['fireball_graphics'],contract['secondary_graphics'],contract['flight_image_banks'],contract['launch_sound'],contract['impact_sound'],contract['flight_subframes'],contract['flight_ticks'])==(171,218,8,98,99,[4,0,1,2,3],[1]*5),'original fireball chain differs')
 catalog_ref=w['current_E149_skill_catalog_development'];require(candidate['current_catalog_qualification']==catalog_ref and sha(catalog_ref['qualification'])==catalog_ref['qualification_sha256'],'current catalog context changed');cq=read(catalog_ref['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog facts changed');facts=read(cq['facts']);require(contract['area_definitions']=={str(row['skill_id']):row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id'] in [22,25,30,53]},'area definitions differ from primary class data')
 api=read(V/'native-E149-area2253053-contract/inspection.json');require(api['formal_target_sha256']==FORMAL and len(api['commands'])==2 and all(row['exit_code']==0 for row in api['commands']) and api['game_classes_instantiated']==0,'original javap inspection differs');check_pins(api['outputs'])
 dispatch_path=V/'native-E149-area2253053-contract/S39-original-dispatch.json';dispatch=read(dispatch_path);require(sha(dispatch['PE'])==dispatch['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','original native PE changed')
 import pefile
 pe=pefile.PE(str(dispatch['PE']),fast_load=True);last=0x4a6510
 for row in dispatch['instructions']:
  address=int(row['address'],16);raw=bytes.fromhex(row['hex']);require(address==last and raw==pe.get_data(address-pe.OPTIONAL_HEADER.ImageBase,len(raw)),'original native instruction bytes differ');last=address+len(raw)
 require(last>=0x4a6868 and not dispatch['PE_executed'],'original S39 dispatch range incomplete');pe.close()
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-area-fireball.py','compile-native-E149-area-fireball.py','qualify-native-E149-area-fireball.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-area-fireball-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_original_area_fireball_class5_resource_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=1,effective_overlay_files=23,new_tests_written=5,retained_skill_tests_written=46,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),original_area_definitions=contract['area_definitions'],resource_chain=[171,218],flight_banks=8,launch_sound=98,impact_sound=99,current_catalog_unchanged=catalog_ref,request_recognized_ids_unchanged=48,missing_request_ids_unchanged=132,original_nested_paths_preserved=6,
  source_pins={str(p):sha(p) for p in [*tools,T/'templates/native_area_fireball_tests.rs',T/'native-E149-area-fireball.md']},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_object374448_qualification=w['current_E149_object374448_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; no current E149 gameplay/resource/audio/GUI execution','original 171 class5 profile only; four area request builders and S39 lifecycle integration remain pending','twenty-three-file projection retains G06 Java and all prior skill/GUI/controller sources','original precompiler Windows-path-spelling failure preserved; no rustc started in that attempt'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 failed=V/'native-E149-area-fireball-metadata/20261024-041249-472190'
 for p in failed.iterdir():
  require(p.is_file(),'unexpected failure archive entry');archive['preparation-failure/'+p.name]=p
 for p in api['outputs']:archive['original-javap/'+Path(p).name]=Path(p)
 archive['original-S39-dispatch.json']=dispatch_path
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('完成範圍技能的當版reader／原資源核對；#22／#25／#30為角色ID＋座標，#53為selector-only；正式回覆走S39全受擊列表。'
  '新增原火球171→218 class5來源載入、八方向全bank、五frame各1tick／Sound98及原218十一frame／Sound99，保留既有三class4與冰箭鏈、取消／budget／reader失敗清除。'
  '一Runtime差分（含五個待測案例，其中兩個原資源前提），最新23檔九Runtime／十一Rust QA／三Java；兩版library／library-test／七既有skill整合共18metadata退出0，新增5與先前46案皆未跑。'
  '原Windows路徑表示差異在Rust前失敗已封存；原生S39分支／constructor bytes沿PE4271D92F核對，不執行原PE。'
  '尚未接範圍請求／選單／S39派送，識別仍48、132個缺口不減；下一步核實一組效果／首目標anchor／全target回呼及1804、1805分層、758與171時序，再完整接四技能。'
  '未選入registry／套用／連結或執行PE／Java／測試／GUI／播放，正常1049輸入／所有PE與G06 binding／16案640次不變；使用者恢復測試後先元件／G06、當版library兩版、Release與完整GUI才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_area_fireball_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='area_fireball_original_class5_resource_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
