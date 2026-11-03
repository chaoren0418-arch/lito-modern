"""Qualify compiler-only fireball class5 clock and exact-frame APIs and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-S23-history-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_fireball25_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==28 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==2,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  base=Path(baseline['candidate_paths'].get(n,str(R/n)));needles=list(dict.fromkeys([str(base),base.as_posix()]));found=[needle for needle in needles if modern.count(needle)==1]
  require(candidate['base_files'][n]==sha(base),'raw module predecessor changed')
  if not found:
   alias=c['module_predecessor_aliases'].get(n);require(alias is not None and alias['raw_source']==str(base) and alias['raw_source_sha256']==sha(base),'generated predecessor alias absent');actual=owned(alias['path']);require(alias['sha256']==old['input_pins'].get(str(actual))==sha(actual)==sha(base) and actual.read_bytes()==base.read_bytes(),'generated predecessor alias differs');needles=[str(actual),actual.as_posix()];needles=list(dict.fromkeys(needles));found=[needle for needle in needles if modern.count(needle)==1]
  require(len(found)==1 and sum(modern.count(needle) for needle in needles)==1,'normal module mapping differs');modern=modern.replace(found[0],(cp.parent/n).as_posix())
 require(set(c['module_predecessor_aliases'])=={'LiTo381-Modern/src/modern_lito/native_world_scene.rs'},'unreviewed generated predecessor alias')
 require(c['paired_runtime_sources']==old['paired_runtime_sources'],'six nested module paths changed')
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond the reviewed runtime path')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 require(c['paired_tests']==old['paired_tests'] and len(c['paired_tests'])==7,'retained seven integrations changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-S23-history-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 spell='LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs';scene='LiTo381-Modern/src/modern_lito/native_world_scene.rs';require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])=={spell,scene},'runtime scope differs')
 template=T/'templates/native_S23_history_tests.rs';qa=template.read_text('utf-8-sig');require(sha(template)==contract['QA_template_sha256'] and len(re.findall(r'#\[test\]',qa))==3 and qa.count('#[ignore=')==1,'QA scope changed')
 for name,replacements in contract['source_changes'].items():
  source=Path(baseline['candidate_paths'].get(name,str(R/name))).read_text('utf-8-sig')
  for before,after in replacements:require(source.count(before)==1,'source anchor differs');source=source.replace(before,after)
  if name==spell:source+=qa
  require(source==(cp.parent/name).read_text('utf-8-sig'),'runtime differs beyond reviewed changes')
 require(all(contract[k] for k in ['same_source_restores_flights','retired_source_preserves_admission_only','merged_admission_requires_same_generation','active_flights_prepare_assets_after_ring_eviction','existing_projectile_clock_unchanged']),'source scope changed')
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
 tools=[T/n for n in ['prepare-native-E149-S23-history.py','compile-native-E149-S23-history.py','qualify-native-E149-S23-history.py','analyze-native-E149-loader-base.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-S23-history-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_S23_Model_history_and_source_admission_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=2,effective_overlay_files=28,new_tests_written=3,retained_skill_tests_written=94,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),current_source_coverage=coverage,existing_S23_clock_unchanged=True,same_source_restores_flights=True,retired_source_keeps_admission_only=True,base_reconstruction=str(analysis_path),base_reconstruction_sha256=sha(analysis_path),PE_provenance=str(provenance_path),PE_provenance_sha256=sha(provenance_path),original_graphics_base_not_proven=True,original_nested_paths_preserved=6,module_predecessor_aliases=c['module_predecessor_aliases'],
  source_pins={str(p):sha(p) for p in [*tools,*[T/'templates'/n for n in ['native_S23_history_tests.rs','native_S23_history.rs']],T/'native-E149-S23-history.md',analysis_path,provenance_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_fireball25_qualification=w['current_E149_fireball25_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; three new and ninety-four preceding written cases remain unrun','S23 history/admission source integration; current rational movement/cue timeline is unchanged and still needs native callback correction','original analysis PE raw split loop conflicts with CRC-correct data; no runtime base 1,5894 or11297 is qualified','no current E149 casts/resources/audio/GUI qualification or normal source application'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'source-pins/'+Path(p).relative_to(R).as_posix():Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 archive['previous-fireball25-qualification.json']=Path(candidate['previous_skill_qualification']['qualification'])
 for name in ['publisher-before.py','failure.json']:archive['original-publication-guard-failure/'+name]=V/'native-E149-S23-history-publication-failures'/'20261024-062521-065601'/name
 failed=V/'native-E149-S23-history-metadata/20261024-061851-440044'
 for n in ['compiler-before.py','original-preparation-failure.json']:archive['original-preparation-failure/'+n]=failed/n
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('補G06相關S23彈體接續：Model FiniteHistory新增generation／seen／path／既有cursor／arrival anchor，僅metadata不帶SPR或PCM；同來源重建續播，來源退役只留admission、不重放舊ring。'
  '取消worker只合併同generation admission，active cue也可在ring／book hint移除後準備資源，原incarnation與世代退役守衛保留。兩Runtime差分共28檔十四Runtime／十一Rust QA／三Java；兩版library／library-test／七skill整合18metadata退出0，新3＋前94案未跑。'
  '原WorldScene生成副本與候選原檔逐byte相同，保留既有六nested參照後編譯；首次準備在Rust前因path guard失敗、原2067ef exit1／0 Rust已封存。'
  '原gfx基準還原：10塊1189423bytes、checksum18638一致，5894個F9與唯一末尾B3，但分析PE字面split只產生1 span；側錄明言不可宣稱可執行。1／5894／11297皆非已驗runtime base，九項相對hurt家族仍未綁定。'
  'S23現有rational時序未改；原class4先flight再secondary、class5相反，下一步按原分支修正60ms／循環與末幀，不套錯順序。請求識別仍52／缺128。'
  '未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放，正常1049輸入／所有PE、registry與G06 binding／16案640次不變。測試及桌面操作持續暫緩。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_S23_history_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='S23_Model_history_and_original_base_reconstruction',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
