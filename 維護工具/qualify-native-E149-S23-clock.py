"""Qualify compiler-only fireball class5 clock and exact-frame APIs and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-S23-clock-origin-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_S23_history_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==29 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==5,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  base=Path(baseline['candidate_paths'].get(n,str(R/n)));needles=list(dict.fromkeys([str(base),base.as_posix()]));found=[needle for needle in needles if modern.count(needle)==1]
  require(candidate['base_files'][n]==sha(base),'raw module predecessor changed')
  if not found:
   alias=c['module_predecessor_aliases'].get(n);require(alias is not None and alias['raw_source']==str(base) and alias['raw_source_sha256']==sha(base),'generated predecessor alias absent');actual=owned(alias['path']);require(alias['sha256']==old['input_pins'].get(str(actual))==sha(actual)==sha(base) and actual.read_bytes()==base.read_bytes(),'generated predecessor alias differs');needles=[str(actual),actual.as_posix()];needles=list(dict.fromkeys(needles));found=[needle for needle in needles if modern.count(needle)==1]
  require(len(found)==1 and sum(modern.count(needle) for needle in needles)==1,'normal module mapping differs');modern=modern.replace(found[0],(cp.parent/n).as_posix())
 require(not c['module_predecessor_aliases'],'new runtime mapping unexpectedly needs aliases')
 require(c['paired_runtime_sources']==old['paired_runtime_sources'],'six nested module paths changed')
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond the reviewed runtime path')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 require(c['paired_tests']==old['paired_tests'] and len(c['paired_tests'])==7,'retained seven integrations changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-S23-clock-origin-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 names={'LiTo381-Modern/src/modern_lito/'+n for n in ['light_arrow_effect.rs','native_spell_projectiles.rs','native_world_scene.rs','native_world_view.rs','native_audio_delivery.rs']};require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])==names,'runtime scope differs');tests=0;ignored=0
 for name,replacements in contract['source_changes'].items():
  source=Path(baseline['candidate_paths'].get(name,str(R/name))).read_text('utf-8-sig')
  for before,after in replacements:require(source.count(before)==1,'reviewed source anchor differs');source=source.replace(before,after)
  if name in contract['QA_templates']:
   template=contract['QA_templates'][name];require(sha(template['path'])==template['sha256'],'QA template changed');qa=Path(template['path']).read_text('utf-8-sig');tests+=len(re.findall(r'#\[test\]',qa));ignored+=qa.count('#[ignore=');source+=qa
  require(source==(cp.parent/name).read_text('utf-8-sig'),'runtime differs beyond reviewed changes')
 require(tests==12 and ignored==1 and contract['profiles']==[167,1583,1799,1797] and contract['movement_ms']==60 and all(contract[k] for k in ['class4_flight_before_secondary','class5_secondary_before_flight','flight_cycles_per_movement','secondary_native_terminal_dwell','typed_audio_occurrence_identity','active_audio_sequence_retention','existing_history_and_source_retirement_preserved','old_rational_inspection_APIs_retained','pending_shared_latch_preserved','weak_history_owns_no_payload','native_global_millisecond_origin']),'source/QA scope differs')
 analysis_path=Path(contract['base_reconstruction']);require(sha(analysis_path)==contract['base_reconstruction_sha256'],'static reconstruction changed');analysis=read(analysis_path);require(sha(analysis['analysis_source'])==analysis['analysis_source_sha256'] and sha(analysis['PE'])==analysis['PE_sha256'],'static sources changed')
 require(analysis['payload_bytes']==1189423 and analysis['marker_F9_count']==analysis['embedded_declared_records']==5894 and analysis['raw_instruction_split_count']==1 and analysis['base_1_5894_or_11297_not_runtime_qualified'] and not analysis['modern_graphics_binding_verified'] and not analysis['PE_executed'] and not analysis['native_instructions_executed'] and analysis['tests_executed']==0,'static reconstruction scope changed')
 import pefile,struct
 pe=pefile.PE(analysis['PE'],fast_load=True);parts=[]
 for row in analysis['chunks']:
  data=pe.get_data(row['address']-pe.OPTIONAL_HEADER.ImageBase,row['bytes']);require(hashlib.sha256(data).hexdigest().upper()==row['sha256'],'embedded chunk changed');parts.append(data)
 payload=b''.join(parts);require(len(payload)==analysis['payload_bytes'] and hashlib.sha256(payload).hexdigest().upper()==analysis['payload_sha256'],'embedded payload changed');table=bytes.fromhex(analysis['CRC_table']['hex']);require(table==pe.get_data(analysis['CRC_table']['address']-pe.OPTIONAL_HEADER.ImageBase,len(table)),'CRC table changed');words=struct.unpack('<256H',table);crc=0
 for byte in payload:crc=(((crc&255)<<8)^words[(crc>>8)&255]^byte)&0xffff
 require(crc==analysis['original_checksum']==analysis['reconstructed_checksum']==18638 and payload.count(b'\xf9')==5894 and payload.count(b'\xb3')==1 and payload[-1]==0xb3,'primary data reconstruction differs')
 for block in analysis['ranges'].values():
  raw_bytes=pe.get_data(block['start']-pe.OPTIONAL_HEADER.ImageBase,block['end']-block['start']);require(hashlib.sha256(raw_bytes).hexdigest().upper()==block['sha256'],'original instruction range changed');at=block['start']
  for instruction in block['instructions']:
   data=bytes.fromhex(instruction['hex']);require(instruction['address']==at and data==pe.get_data(at-pe.OPTIONAL_HEADER.ImageBase,len(data)),'original instruction changed');at+=len(data)
  require(at==block['end'],'original range incomplete')
 pe.close();require(sha(analysis['LT_reference'])==analysis['LT_reference_sha256'] and sha(analysis['shared_catalog'])==analysis['shared_catalog_sha256'],'original resource reference changed')
 provenance_path=analysis_path.with_name('PE-provenance.json');provenance=read(provenance_path);require(sha(provenance['source'])==provenance['source_sha256'] and Path(provenance['source']).resolve(strict=True)==Path(analysis['PE']+'.json').resolve(strict=True) and json.loads(Path(provenance['source']).read_text('utf-8-sig'))==provenance['body'] and provenance['body']['output_sha256']==analysis['PE_sha256'] and not provenance['body']['runnable_claim'],'PE provenance changed')
 previous=read(candidate['previous_skill_qualification']['qualification']);coverage=previous['current_source_coverage'];require(coverage['recognized_count']==52 and coverage['missing_request_count']==128,'request coverage changed')
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-S23-clock-origin.py','compile-native-E149-S23-clock-origin.py','qualify-native-E149-S23-clock.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-S23-clock-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_S23_native_callback_clock_and_pending_latch_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=5,effective_overlay_files=29,new_tests_written=12,retained_skill_tests_written=97,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),current_source_coverage=coverage,native_S23_clock_source_integrated=True,native_global_millisecond_origin=True,superseded_relative_origin_cohort=str(V/'native-E149-S23-clock-latch-metadata/20261024-064820-504167/cohort.json'),superseded_relative_origin_cohort_sha256=sha(V/'native-E149-S23-clock-latch-metadata/20261024-064820-504167/cohort.json'),profiles=contract['profiles'],movement_ms=60,class4_flight_before_secondary=True,class5_secondary_before_flight=True,secondary_terminal_dwell_ms={'219':41,'218':41,'1800':166,'1798':125},typed_callback_audio=True,active_sequence_retention=True,pending_shared_latch_preserved=True,weak_history_owns_no_payload=True,existing_history_preserved=True,superseded_clock_cohort=str(V/'native-E149-S23-clock-metadata/20261024-064039-042798/cohort.json'),superseded_clock_cohort_sha256=sha(V/'native-E149-S23-clock-metadata/20261024-064039-042798/cohort.json'),old_source_success_transferred=False,base_reconstruction=str(analysis_path),base_reconstruction_sha256=sha(analysis_path),PE_provenance=str(provenance_path),PE_provenance_sha256=sha(provenance_path),original_graphics_base_not_proven=True,original_nested_paths_preserved=6,module_predecessor_aliases=c['module_predecessor_aliases'],
  source_pins={str(p):sha(p) for p in [*tools,*[T/'templates'/n for n in ['native_S23_clock_tests.rs','native_S23_clock_origin_integration_tests.rs','native_S23_clock_core.rs','native_S23_clock_assets.rs','native_S23_clock_origin_update.rs','native_S23_clock_origin_draw.rs','native_S23_latch_tests.rs']],T/'native-E149-S23-clock-origin.md',analysis_path,provenance_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_S23_history_qualification=w['current_E149_S23_history_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; twelve new and ninety-seven preceding written cases remain unrun','native S23 clock/cue/terminal/source-history integration and weak latch source fix only; current E149 casts/resources/GUI/device playback unqualified','old public rational inspection APIs retain their historical scope; no runtime success transferred','original graphics base initialization/relative no-hurt family and G11 maximum live history budget remain unproved'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'source-pins/'+Path(p).relative_to(R).as_posix():Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 archive['previous-S23-history-qualification.json']=Path(candidate['previous_skill_qualification']['qualification'])
 sup_path=Path(record['superseded_clock_cohort']);sup=read(sup_path);require(sup['status']=='eighteen_metadata_compilers_complete_no_tests' and [row['exit_code'] for row in sup['compiler_runs']]==[0]*18,'superseded compile changed');check_pins(sup['input_pins']);sup_cp=Path(sup['candidate']);sup_candidate=read(sup_cp)
 archive['superseded-clock/cohort.json']=sup_path;archive['superseded-clock/candidate.json']=sup_cp;archive['superseded-clock/contract.json']=Path(sup_candidate['contract']);archive['superseded-clock/superseded-source.json']=sup_path.with_name('superseded-source.json')
 for n in sup_candidate['candidate_files']:archive['superseded-clock/candidate/'+n]=sup_cp.parent/n
 for n in ['prepare-native-E149-S23-clock.py','compile-native-E149-S23-clock.py','native-E149-S23-clock.md']:archive['superseded-clock/tools/'+n]=T/n
 for n in ['native_S23_clock_tests.rs','native_S23_clock_integration_tests.rs','native_S23_clock_core.rs','native_S23_clock_assets.rs','native_S23_clock_update.rs','native_S23_clock_draw.rs']:archive['superseded-clock/templates/'+n]=T/'templates'/n
 for row in sup['compiler_runs']:
  check_pins(row['logs']);require(sha(row['product'])==row['product_sha256'],'superseded product changed')
  for log in row['logs']:archive['superseded-clock/'+row['profile']+'-'+row['kind']+'/'+Path(log).name]=Path(log)
 relative_path=Path(record['superseded_relative_origin_cohort']);relative=read(relative_path);require(relative['status']=='eighteen_metadata_compilers_complete_no_tests' and [row['exit_code'] for row in relative['compiler_runs']]==[0]*18,'relative-origin cohort changed');check_pins(relative['input_pins']);relative_cp=Path(relative['candidate']);relative_candidate=read(relative_cp)
 archive['superseded-relative-origin/cohort.json']=relative_path;archive['superseded-relative-origin/candidate.json']=relative_cp;archive['superseded-relative-origin/contract.json']=Path(relative_candidate['contract']);archive['superseded-relative-origin/superseded-source.json']=relative_path.with_name('superseded-source.json')
 for n in relative_candidate['candidate_files']:archive['superseded-relative-origin/candidate/'+n]=relative_cp.parent/n
 for n in ['prepare-native-E149-S23-clock-latch.py','compile-native-E149-S23-clock-latch.py','native-E149-S23-clock-latch.md']:archive['superseded-relative-origin/tools/'+n]=T/n
 for row in relative['compiler_runs']:
  check_pins(row['logs']);require(sha(row['product'])==row['product_sha256'],'relative-origin product changed')
  for log in row['logs']:archive['superseded-relative-origin/'+row['profile']+'-'+row['kind']+'/'+Path(log).name]=Path(log)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('S23四profile167／1583／1799／1797改接原生clock：首步立即、先floor全域毫秒再相減作clock origin，60ms一次只進一格，延遲不補跑、每movement循環source selector；class4先flight再secondary，冰箭class5相反。'
  'secondary含source0停留219／21841ms、1800166ms、1798125ms；exact source取圖、draw不推時鐘、實際arrival只release一次hurt，保留heading／arrival anchor／incarnation／來源退役與重建history。'
  '改用typed callback audio occurrence並保留active sequences；history以Weak辨識canonical payload，相同Arc duplicate不消耗待播原latch，不同payload同key仍退役且不保有PCM；初版與相對起點版18metadata均封存，不移記修正版。'
  '五Runtime差分共29檔十五Runtime／十一Rust QA／三Java，兩版library／library-test／七skill整合18metadata退出0，新12＋前97案未跑。原public rational API僅保留inspection範圍，不移記舊資格；請求識別仍52／缺128。'
  '原gfx base初始化／LT hurt家族與G11最大history預算仍待，不能拿1／5894／11297直接代入。未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放，正常1049輸入／所有PE、registry及G06 binding／16案640次不變；測試與桌面操作持續暫緩。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_S23_clock_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='S23_native_clock_and_pending_shared_audio_latch',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
