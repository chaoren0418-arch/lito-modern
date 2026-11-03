"""Retain qualified Debug while the same original Release pipeline remains live."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33A3-default116')
c=read(run/'cohort.json');assert c['status']in ['executing_current_default116_profile','independently_reading_current_default116_profile']and c['active_profile']=='Release'
info=c['profiles']['Debug'];qp=Path(info['qualification']);q=read(qp)
assert info['qualified']and info['actual_runner_exit_code']==0 and sha(qp)==info['qualification_sha256']
assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['counts']==dict(passed=538,failed=0,ignored=23)
assert q['integration_targets']==116 and q['actual_test_executables']==117 and not q['full_library_scope_inferred']and sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in c['validation_pins'].items():assert sha(ROOT/'維護工具'/n)==h,n
pid=c['pipeline_pid'];command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'))
assert live['ProcessId']==pid and abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
queue=read(ROOT/'工作佇列.json');assert queue['active_workload_sessions']=={'broad_default':a.session}and queue['source_and_corresponding_QA_frozen']
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
followup=V/'native33A3-contract-followup/route-followup.json';f=read(followup);prep=Path(f['candidate_preparation']);candidate=read(prep)
assert f['source_manifest_sha256']==candidate['base_source_manifest_sha256']==c['source_manifest_sha256']and sha(prep)==f['candidate_preparation_sha256']and sha(prep.parent/'probe.rs')==candidate['probe_sha256']
assert not candidate['compiled']and not candidate['tests_executed']and candidate['actual_sessions']==0 and not candidate['normal_source_modified']
out=run/('checkpoint-Debug-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_Debug_default116_qualified_Release_original_live',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session,pipeline=dict(pid=pid,creation_utc=c['pipeline_creation_utc']),observed_live=live,Debug=dict(qualification=str(qp),qualification_sha256=sha(qp),counts=q['counts'],integration_targets=116,actual_test_executables=117,actual_profile_runner_exit_code=0),Release_qualified=False,original_wrapper_terminal_observed=False,qualification_granted=False,full_library_in_this_scope=False,GUI_verified=False,whole_goal_complete=False)
pending=dict(report=str(followup),report_sha256=sha(followup),candidate_preparation=str(prep),candidate_preparation_sha256=sha(prep),compiled=False,executed=False,old_CA00_failure_preserved=True,qualification_granted=False)
save(out/'evidence.json',record);(out/'live-process-CIM.json').write_bytes(raw)
work='當版33A3CCF7廣域Debug已獨立核對117實際測試映像、538過0敗23忽略；忽略本文未執行。Release沿原'+str(a.session)+'執行，整批尚未授予資格，來源與對應QA固定。兩版正常各31相關案、能力雙玩家8sessions及G06控制器96sessions維持各自範圍。完整library與Presentation傳送非空音效／其他服務仍待；同端點已另備依收到租裝服務接續的獨立案例，尚未編譯／執行，原CA00普通頻道失敗保留。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立，完整G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        value.update(current_native33A3_broad_active=record,pending_native33A3_same_endpoint_rental_probe=pending,active_workloads={'broad_default':record},active_workload_sessions={'broad_default':a.session},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current33A3-Release-broad',work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in value:value['updated_date']=date
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
continuation=read(run.parent/'current-continuation.json');continuation.update(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),pending_same_endpoint_probe=pending,must_poll_original_before_restart=True);save(run.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),Debug_passed=538,Release_original_live=True,original_session=a.session),ensure_ascii=False))
