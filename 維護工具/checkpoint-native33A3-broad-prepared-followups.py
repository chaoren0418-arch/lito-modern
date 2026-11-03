"""Verified wait on the same broad process; record separate unexecuted followups."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(path):
    for i in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33A3-default116')
c=read(run/'cohort.json');assert c['status']=='executing_current_default116_profile' and c['active_profile']=='Release'
source=c['source_manifest_sha256'];assert source=='33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21'
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048
for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
for name,digest in c['validation_pins'].items():assert sha(T/name)==sha(run/name)==digest,name
pid=c['pipeline_pid'];ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig'))
assert live['ProcessId']==pid and abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
debug=read(c['profiles']['Debug']['qualification']);assert c['profiles']['Debug']['qualified'] and debug['counts']==dict(passed=538,failed=0,ignored=23)
assert sha(c['profiles']['Debug']['qualification'])==c['profiles']['Debug']['qualification_sha256']
console=run/'Release-console/stdout.log';matches=[x.removeprefix('CURRENT1047_DEFAULT116=')for x in console.read_text('utf-8-sig').splitlines()if x.startswith('CURRENT1047_DEFAULT116=')]
assert len(matches)==1;release=Path(matches[0]).resolve(strict=True)
assert release.is_relative_to(V/'default-native-regression')
r=read(release/'receipt.json');assert r['source_manifest_sha256']==source and r['profile']=='Release' and r['status']in ['compiling_default_targets','running_default_targets']
executed=r['results'];assert all(row['exit_code']==0 and row['failed']==0 and not row['timed_out']for row in executed.values())
log=(release/'build/stdout.log').read_bytes();rows=[]
for line in log[:log.rfind(b'\n')+1].splitlines():
    if not line.startswith(b'{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and row.get('executable') and row.get('profile',{}).get('test'):rows.append(row['executable'])
assert len(rows)==len(set(rows)) and 0<len(rows)<118
pending={}
for name in ['pending-current-audio-tooling.json','pending-current-hotel-tooling.json','pending-counted-completion-observer.json','pending-current-hotel-departure-contract.json','pending-completed-checkpoint-scope-cleanup.json']:
    path=run/name;record=read(path)
    if isinstance(record.get('tooling_sha256'),dict):
        for tool,digest in record['tooling_sha256'].items():assert sha(T/tool)==digest,tool
    elif 'tooling'in record:
        assert sha(record['tooling'])==record['tooling_sha256']
    if 'new_observer'in record:assert sha(record['new_observer'])==record['new_sha256'] and sha(record['original_observer'])==record['original_sha256']
    if 'report'in record:assert sha(record['report'])==record['report_sha256']
    if 'record'in record:assert sha(record['record'])==record['record_sha256']
    if 'tool'in record:assert sha(record['tool'])==record['tool_sha256']
    pending[name]=dict(path=str(path),sha256=sha(path),qualified=False,executed=False)
queue=read(ROOT/'工作佇列.json');assert queue['active_workload_sessions']=={'broad_default':a.session} and queue['source_and_corresponding_QA_frozen']
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
out=run/('checkpoint-prepared-followups-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='current_Debug_qualified_Release_same_original_active_followups_staged',run=str(run),source_manifest_sha256=source,original_session_id=a.session,pipeline=dict(pid=pid,creation_utc=c['pipeline_creation_utc']),observed_live=live,Debug_counts=debug['counts'],Release_run=str(release),Release_status=r['status'],Release_compiled_test_images_observed=len(rows),Release_executed_targets_observed=len(executed),Release_partial_counts={key:sum(row[key]for row in executed.values())for key in ['passed','failed','ignored']},Release_qualified=False,original_wrapper_terminal_observed=False,pending_followups=pending,original_source1048_and_validation_pins_unchanged=True,qualification_granted=False,whole_goal_complete=False)
save(out/'evidence.json',record);(out/'live-process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
stage='編譯，已觀察'+str(len(rows))+'/117測試映像，尚未執行測試'if r['status']=='compiling_default_targets'else'執行，117測試映像已產生，已讀回'+str(len(executed))+'/117目標、目前'+str(record['Release_partial_counts']['passed'])+'過0敗'
text='當版33A3CCF7廣域Debug538過0敗23忽略，Release沿原'+str(a.session)+stage+'，尚未授予整批Release資格；來源與原對應QA固定。已另備正常旅館八案及Presentation非空音效／同端點租裝服務獨立工具，未編譯／執行，不計通過。整批終止後先獨立讀回234實際映像，再執行同來源完整library及各服務流程。開發期間不請使用者登入，登入器與重建版產物獨立且相鄰只讀，8秒／60秒／Release與G11預算不變，完整G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for name in before:
    path=ROOT/name
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value.update(current_native33A3_broad_active=record,pending_native33A3_followups=pending,active_workloads={'broad_default':record},active_workload_sessions={'broad_default':a.session},source_and_corresponding_QA_frozen=True,next_task=text,next_focus=text,last_goal_turn_classification='verified_wait',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current33A3-Release-broad',work=text,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in value:value['updated_date']=date
        if name.endswith('next-validation-action.json'):value['next_action']=text
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+text+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name)for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
continuation=read(run.parent/'current-continuation.json');continuation.update(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=True);save(run.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),original_session=a.session,Release_compiled_images=len(rows),Debug_passed=538,qualification_granted=False),ensure_ascii=False))
