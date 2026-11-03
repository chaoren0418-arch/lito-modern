"""Read back published files/hashes only; never launch products or test bodies."""
from pathlib import Path
import argparse,ast,hashlib,json,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
V=R/'驗證紀錄/current'
def main(master_readback):
 w=read(R/'工作佇列.json');ref=w['current_E149_fireball25_development'];qp=Path(ref['qualification']);require(sha(qp)==ref['qualification_sha256'],'qualification changed');q=read(qp);check_pins(q['source_pins'])
 require(q['status']=='unselected_fireball25_request_S39_callback_audio_metadata_only' and q['source_manifest_sha256']==SOURCE and q['formal_target_sha256']==FORMAL and q['compiler_exit_codes']==[0]*18,'published compiler scope differs')
 require(q['flight_source_cycle']==[1,2,3,4,0] and q['request_S39_world_integration'] and q['audio_callback_occurrence_identity'] and q['normal_retirement_does_not_poison_loading'] and not q['old_source_success_transferred'] and q['new_tests_written']==12 and q['retained_skill_tests_written']==82,'published source scope differs')
 require(q['tests_executed']==q['new_sessions']==q['desktop_inputs']==q['WAV_playbacks']==0 and not any(q[k] for k in ['PEs_linked','outputs_executed','production_applied','canonical_registry_selected','GUI_verified','whole_goal_complete']),'execution scope differs')
 require(q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'] and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'],'existing control context changed')
 require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'],'deferred/idle scope changed')
 cp=Path(q['candidate']);require(sha(cp)==q['candidate_sha256'],'candidate changed');candidate=read(cp);require(len(candidate['effective_overlay_files'])==27 and len(candidate['candidate_files'])==4,'projection count differs');check_pins({candidate['effective_overlay_paths'][n]:h for n,h in candidate['effective_overlay_files'].items()})
 previous=read(q['previous_fireball_clock_qualification']['qualification']);require(set(q['current_source_coverage']['recognized_ids'])==set(previous['current_source_coverage']['recognized_ids'])|{25} and q['current_source_coverage']['recognized_count']==52 and q['current_source_coverage']['missing_request_count']==128,'request source coverage changed')
 run=Path(q['compile_run']);require(sha(run/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');c=read(run/'cohort.json');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and not c['outputs_executed'] and not c['PEs_linked'] and c['tests_executed']==0,'compiler scope differs')
 require(len(c['compiler_runs'])==18,'compiler target count differs')
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and '--emit=metadata' in row['command'] and '--emit=link' not in row['command'] and not row.get('original_handle_cleanup'),'compiler completion differs');check_pins(row['logs']);require(sha(row['product'])==row['product_sha256'] and Path(row['product']).stat().st_size==row['product_bytes'],'metadata changed')
 normal=read(c['base_compiled_inputs']);require(len(normal)==1049,'normal count differs');check_pins({str(R/n):h for n,h in normal.items()})
 base=read(w['current_development_typecheck_qualification']['qualification']);check_pins(base['normal_product_pins']);require(base['normal_product_pins']==c['normal_product_pins'],'normal products differ')
 evidence=Path(w['last_goal_turn_evidence']);e=read(evidence);rb=read(evidence.with_name('readback.json'));require(evidence.parent==qp.parent and sha(evidence)==rb['evidence_sha256'] and e['qualification_sha256']==sha(qp),'checkpoint evidence changed');require(len(rb['after_files'])==8,'checkpoint count differs');check_pins({str(R/n):h for n,h in rb['after_files'].items()})
 with zipfile.ZipFile(qp.parent/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in rb['before_files'].items()),'before archive differs')
 require(sha(e['raw_archive'])==e['raw_archive_sha256'],'raw archive changed')
 with zipfile.ZipFile(e['raw_archive']) as z:
  expected={'qualified-development.json':qp,'cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(q['contract'])}
  expected.update({'source-pins/'+Path(p).relative_to(R).as_posix():Path(p) for p in q['source_pins']});expected.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
  for row in c['compiler_runs']:
   for p in row['logs']:expected[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
  old=Path(q['superseded_lifecycle_error_cohort']);require(sha(old)==q['superseded_lifecycle_error_cohort_sha256'],'superseded cohort changed');old_c=read(old);old_cp=Path(old_c['candidate']);old_candidate=read(old_cp)
  expected.update({'superseded-lifecycle/cohort.json':old,'superseded-lifecycle/candidate.json':old_cp,'superseded-lifecycle/superseded-source.json':old.with_name('superseded-source.json'),'previous-clock-qualification.json':Path(q['previous_fireball_clock_qualification']['qualification'])})
  for n in old_candidate['candidate_files']:expected['superseded-lifecycle/candidate/'+n]=old_cp.parent/n
  require(read(old.with_name('superseded-source.json'))['actual_exit_code']==0 and not read(old.with_name('superseded-source.json'))['source_success_transferred'],'superseded scope changed')
  require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in expected.items()),'raw archive member differs')
 master=R.parent/'LiTo381-總開發報告.md';latest=owned(master_readback.resolve(strict=True),V/'master-report-current-work');report=read(latest);require(sha(master)==report['after_sha256'],'master report changed');body=master.read_text('utf-8-sig');require(all(s in body for s in ['測試與桌面操作暫緩','呈現來源已接','1/2/3/4/0',str(qp).replace('\\','/'),'新12＋前82案']),'master report is not current')
 ast.parse(Path(__file__).read_text('utf-8-sig'))
 result=dict(status='independent_files_and_hashes_read_back_no_tests',qualification=str(qp),qualification_sha256=sha(qp),normal_inputs=1049,overlay_files=27,metadata_compilers=18,tests_executed=0,desktop_inputs=0,normal_PEs_unchanged=True,eight_checkpoints={n:sha(R/n) for n in rb['after_files']},raw_archive_sha256=sha(e['raw_archive']),superseded_lifecycle_source_preserved=True,master_report=str(master),master_report_sha256=sha(master),master_report_lines=len(body.splitlines()),readback_source_sha256=sha(Path(__file__)),whole_goal_complete=False)
 (qp.parent/'final-readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--master-readback',required=True,type=Path);main(parser.parse_args().master_readback)
