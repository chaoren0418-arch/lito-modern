"""Publish bounded standalone source work, retaining the 42-file runtime baseline."""
from pathlib import Path
from datetime import datetime, timezone
import json, hashlib, zipfile
from native_E149_recovery_contract import R, FORMAL, read, sha, encoded, require, check_pins
from native_E149_process_birth_readonly import original_terminal
T=R/'維護工具'; V=R/'驗證紀錄/current'
KEY='current_E149_hide97_static_core_development'
EP=V/'native-E149-hide97-static-core-development/20261024-193738-766504/evidence.json'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='97靜態2941新增獨立生命週期核心草稿，兩版metadata退出0；不在42檔候選或371module內，未接S37／Scene／資源／History／GUI，不能宣告97完成。零delay先排隊、同deadline穩定順序、固定sample逐筆先pop再callback；外部callback可先觀察frame0，再由退役callback移除，不保證固定一幀。Drain借用本model，不能跨model移用sample；owner退役清工作並留admission tombstones，overflow／時鐘倒退拒絕。4E2150兩繪圖suffix已核對，但只是到達4E3207後的區域條件，不是全部CFG或實際globals；另確認S37/169自己的4857B0有500ms相對clock drain／draw loop，2941不直接走169分支，全域／巢狀drain仍缺。最新42檔18metadata與459案全未跑、88／剩92不變；未套用正常1049、未連結PE／選registry／測試／桌面／播放。'
NEXT='先接97 canonical靜態圖載入與S37 admission／Scene／History／owner退役，釐清4E2150兩suffix上游、4857B0巢狀drain與全域clock／callback交錯；獨立core僅條件模型，不能當runtime排程或像素資格。不得假設無像素、固定一幀或32秒本機刪除。完成97後續99／100及剩92請求，保留selector_only共用helper與coverage parser。測試／桌面待使用者明確恢復，再依總報告跑G06640次、完整library Debug／Release、Release回歸與租裝等完整GUI。'

def main():
 w=read(R/'工作佇列.json');require(KEY not in w and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase differs')
 require(sha(EP)=='E0E2F1429173F00DDF5C34B89AA88BBED0CF701C2B1F0DD23BE3DB41B749D48B','source evidence differs');e=read(EP);check_pins(e['input_pins']);check_pins(e['normal_product_pins']);require(e['status']=='two_standalone_metadata_complete_no_runtime_integration' and e['tests_executed']==0 and not e['PEs_linked'] and not e['outputs_executed'],'scope differs')
 require(len(e['compiler_runs'])==2 and [r['profile']for r in e['compiler_runs']]==['Debug','Release'],'compiler inventory differs')
 for row in e['compiler_runs']:
  require(row['exit_code']==0 and row['original_handle_signaled'] and '--emit=metadata'in row['command'] and '--crate-type=lib'in row['command'] and '--test'not in row['command'] and sha(row['product'])==row['product_sha256'],'metadata differs');require(sha(EP.parent/(row['profile']+'.stdout'))==row['stdout_sha256'] and sha(EP.parent/(row['profile']+'.stderr'))==row['stderr_sha256'],'compiler log differs')
 observations=[original_terminal(row)for row in [e['pipeline']]+[v['original_process']for v in e['compiler_runs']]]
 ref=w['current_E149_hide97_request_venom98_finite_development'];q=read(ref['qualification']);require(e['review']['base_qualification']==ref and sha(ref['qualification'])==ref['qualification_sha256'],'baseline differs');c=read(Path(q['compile_run'])/'cohort.json');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049,'normal count differs');check_pins({str(R/n):h for n,h in inputs.items()});require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry differs')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(set(before)==set(CHECKPOINTS) and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');out=V/'native-E149-hide97-static-core-checkpoint'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 raw={str(p):sha(p)for p in EP.parent.iterdir()if p.is_file()};raw.update({str(Path(__file__).resolve()):sha(__file__),str(T/'templates/native_hide97_static_lifecycle_draft.rs'):sha(T/'templates/native_hide97_static_lifecycle_draft.rs')})
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
  for p,h in raw.items():require(sha(p)==h,'raw differs');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip')as z:require(all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw readback differs')
 record=dict(schema='E149-hide97-static-core-checkpoint-v1',source_evidence=str(EP),source_evidence_sha256=sha(EP),previous_qualified_runtime=ref,original_process_observations=observations,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw,metadata_exit_codes=[0,0],tests_written=0,tests_executed=0,normal_inputs_checked=1049,current_fixture_binding_unchanged=q['current_fixture_binding_unchanged'],current_formal_suite_unchanged=q['current_formal_suite_unchanged'],canonical_registry_unchanged=q['canonical_registry_unchanged'],note=NOTE,next_action=NEXT,integrated_into_runtime=False,whole_goal_complete=False)
 cp=out/'evidence.json';cp.write_bytes(encoded(record));published=dict(evidence=str(cp),evidence_sha256=sha(cp),source_evidence=str(EP),source_evidence_sha256=sha(EP),status='standalone_conditional_core_compiled_not_integrated',metadata_exit_codes=[0,0],tests_executed=0,whole_goal_complete=False)
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent writer')
  if n.endswith('.json'):
   j=read(p);j[KEY]=published;j['last_goal_turn_evidence']=str(cp);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if n in ['工作佇列.json','監製進度.json']:j['current_focus']=dict(item='hide97_static_core_not_integrated',work=NOTE,next=NEXT,evidence=str(cp));j['next_task']=NEXT;j['progress_updated_utc']=datetime.now(timezone.utc).isoformat()
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-25：'+NOTE+' '+NEXT+' [獨立核心與來源](<'+cp.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n)for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,evidence=str(cp),evidence_sha256=sha(cp),before_archive_sha256=sha(out/'before.zip'),tests_executed=0,whole_goal_complete=False)));print(json.dumps(dict(evidence=str(cp),sha256=sha(cp),readback=str(out/'readback.json'),metadata_exit_codes=[0,0],tests_executed=0),ensure_ascii=False))

if __name__=='__main__':main()
