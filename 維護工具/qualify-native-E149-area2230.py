"""Qualify compiler-only S39 snapshot class1 area2230 source work and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-area2230-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_class1_origin_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==26 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==4,'merged source projection differs')
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
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-area2230-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 many='LiTo381-Modern/src/modern_lito/native_attack_many.rs';skills='LiTo381-Modern/src/modern_lito/native_skills.rs';selection='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';world='LiTo381-Modern/src/modern_lito/native_world.rs';require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])=={many,skills,selection,world},'runtime scope differs')
 template=T/'templates/native_area2230_tests.rs';qa=template.read_text('utf-8-sig');require(sha(template)==contract['QA_template_sha256'] and len(re.findall(r'#\[test\]',qa))==9 and qa.count('#[ignore=')==1,'QA source scope changed')
 for name,replacements in contract['source_changes'].items():
  source=Path(baseline['candidate_paths'].get(name,str(R/name))).read_text('utf-8-sig')
  for before,after in replacements:require(source.count(before)==1,'reviewed source anchor differs');source=source.replace(before,after)
  if name==many:source+=qa
  require(source==(cp.parent/name).read_text('utf-8-sig'),'runtime differs beyond reviewed changes')
 require(contract['graphics']==[1804,1805] and contract['S39_action']==18 and contract['S39_type']==8 and contract['snapshot_at_packet_receipt'] and contract['costs_damage_and_radius_server_owned'],'area semantics scope changed')
 native_path=Path(contract['native_snapshot_contract']);require(sha(native_path)==contract['native_snapshot_contract_sha256'],'native snapshot contract changed');native=read(native_path);require(sha(native['PE'])==native['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9' and not native['PE_executed'] and not native['SQL_executed'],'original native evidence changed')
 import pefile
 pe=pefile.PE(native['PE'],fast_load=True)
 for block in native['ranges'].values():
  raw=pe.get_data(block['start']-pe.OPTIONAL_HEADER.ImageBase,block['end']-block['start']);require(hashlib.sha256(raw).hexdigest().upper()==block['sha256'],'native range changed');at=block['start']
  for row in block['instructions']:
   data=bytes.fromhex(row['hex']);require(row['address']==at and data==pe.get_data(at-pe.OPTIONAL_HEADER.ImageBase,len(data)),'native instruction bytes changed');at+=len(data)
  require(at==block['end'],'native range incomplete')
 pe.close();require(native['first_target_snapshot_at_packet_receipt'] and native['first_hit_flag_does_not_change_anchor'] and native['missing_first_target_fallback_one_step'],'native capture semantics differ')
 require(sha(contract['corrected_start_contract'])==contract['corrected_start_contract_sha256'],'corrected start evidence changed');start_proof=read(contract['corrected_start_contract']);require(all(start_proof['graphic_traces'][str(gfx)]['reached']==0x45001a for gfx in [1804,1805]),'class1 area origin is not direct start')
 catref=w['current_E149_skill_catalog_development'];require(candidate['current_catalog_qualification']==catref and sha(catref['qualification'])==catref['qualification_sha256'],'catalog context changed');cq=read(catref['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog facts changed');facts=read(cq['facts']);require(contract['definitions']=={str(row['skill_id']):row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id'] in [22,30]},'E149 definitions differ')
 api_path=Path(native['E149_API']);require(sha(api_path)==native['E149_API_sha256'],'original javap inspection changed');api=read(api_path);require(api['formal_target_sha256']==FORMAL and all(row['exit_code']==0 for row in api['commands']) and api['game_classes_instantiated']==0,'original javap scope differs');check_pins(api['outputs'])
 protocol=next(Path(p) for p in api['outputs'] if p.endswith('Protocol.stdout.txt'));text=protocol.read_text('utf-8-sig');heads=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',text,re.M));wrappers=[text[m.start():heads[i+1].start()] for i,m in enumerate(heads) if ' attackMany(' in m[0] and m[0].count('int')==5 and 'java.util.List' in m[0]];require(len(wrappers)==1 and re.search(r': bipush +8\n',wrappers[0]) and 'attackMany:(IJIIIJIILjava/util/List;)[B' in wrappers[0],'default attackMany type8 differs')
 request=(cp.parent/skills).read_text('utf-8-sig');body=request[request.index('pub fn request('):request.index('pub fn reuse_millis(')];recognized=set()
 for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):recognized.update(map(int,re.findall(r'\d+',match[1])))
 ground=re.search(r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',request);item=re.search(r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',request);require(ground and item,'request syntax differs');recognized.update(map(int,re.findall(r'\d+',ground[1])));recognized.update(map(int,re.findall(r'\d+',item[1])))
 old_coverage=candidate['previous_skill_qualification']['current_source_coverage_unchanged'];missing=sorted(set(old_coverage['missing_request_ids'])-{22,30});require(len(recognized)==51 and recognized==set(old_coverage['recognized_ids'])|{22,30} and len(missing)==129,'source recognition changes exceed22/30')
 coverage=dict(request_source=str(cp.parent/skills),request_source_sha256=sha(cp.parent/skills),recognized_ids=sorted(recognized),recognized_count=51,catalog_literal_ids=180,missing_request_ids=missing,missing_request_count=129,source_recognition_only=True,learned_book_or_runtime_verified=False)
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-area2230.py','compile-native-E149-area2230.py','qualify-native-E149-area2230.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-area2230-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_area2230_receipt_snapshot_S39_layers_audio_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=4,effective_overlay_files=26,new_tests_written=9,retained_skill_tests_written=65,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),original_definitions=contract['definitions'],display_names=contract['display_names'],S39_action=18,S39_type=8,graphics=[1804,1805],frames=contract['frames'],sounds=contract['sounds'],hit_source_frames=contract['hit_source_frames'],native_snapshot_contract=str(native_path),native_snapshot_contract_sha256=sha(native_path),corrected_start_contract=contract['corrected_start_contract'],corrected_start_contract_sha256=contract['corrected_start_contract_sha256'],current_catalog_ancestor=catref,current_source_coverage=coverage,remaining_area_skills=[25],original_nested_paths_preserved=6,
  source_pins={str(p):sha(p) for p in [*tools,T/'templates/native_area2230_tests.rs',T/'native-E149-area2230.md',native_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_class1_origin_qualification=w['current_E149_class1_origin_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; no current E149 gameplay/resource/audio/GUI execution','source integration only; E149 books/casts/low-MP/range/LOS/SPR/audio/GUI remain unrun','twenty-six-file projection retains corrected class1 origins, tornado four-copy source and G06 Java','class5 area25 still requires S39 integration; no historical runtime qualification is transferred'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 archive['original-native-snapshot.json']=native_path;archive['corrected-native-start.json']=Path(contract['corrected_start_contract'])
 for path in api['outputs']:archive['original-javap/'+Path(path).name]=Path(path)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('接通#22寒冰氣息／#30巖牢：C14 02/05或03/05接unsigned actor ID與x/y，range6／8、reuse300／500，MP20／24等成本／命中／傷害由E149決定。'
  'S39僅新增action18-type8-1804/1805；在接收時凍結第一target座標，不看hit位，首目標缺席／空名單按wire origin沿原一步，不重查或選第二人。'
  '1804原21frame／Sound714／Hit8；1805原17frame／Sound718／Hit4與7、body Over＋1806 shadow＋1807 Screen smoke全載入，整份列表沿hit／incarnation與history／世代退役，音效用bank準備原生抑制群。'
  '四Runtime差分含九案（一案原資源），最新26檔十二Runtime／十一Rust QA／三Java；兩版library／library-test／七skill整合18metadata退出0，新9與前65案未跑。'
  '候選請求識別由49加22／30成51，180目錄尚129無入口，非施法／資源／GUI通過率；#25仍待S39 class5首目標／flight／218secondary／全名單及arrival時序後再接請求選單，G06仍優先主線。'
  '未套用／選入registry／連結或執行PE／Java／SQL／測試／GUI／播放，正常1049輸入／所有PE與G06 binding／16案640次不變。使用者恢復測試後先元件／G06、當版完整library兩版、Release與完整GUI自行確認後才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_area2230_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='area2230_receipt_target_snapshot_and_original_layered_S39',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
