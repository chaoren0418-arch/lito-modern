"""Track the exact off-tree live candidate run; terminal is not qualification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--mode',choices=['active','terminal'],required=True);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-candidate') and (a.session is None or a.session>0)
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
assert not c['normal_product_modified'] and not c['candidate_applied']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(n)==h for n,h in c['generated_source_pins'].items())
assert sha(c['candidate'])==c['candidate_sha256'] and sha(c['component_qualification'])==c['component_qualification_sha256']
assert sha(c['controller_original'])==c['controller_original_sha256'] and sha(c['fixture_origin'])==sha(run/'NativePeerHarness.java')==c['fixture_sha256']
assert (run/'native_login.rs').read_text('utf-8-sig')==Path(c['controller_original']).read_text('utf-8-sig').partition(c.get('controller_cut_marker','pub fn probe(\n'))[0]
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(x['pid']) for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
obs=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert obs.returncode==0 and not obs.stderr
live=json.loads(obs.stdout.decode('utf-8-sig')) if obs.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
same=lambda x,y:x['ProcessId']==y['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(y['creation_utc'])).total_seconds())<.001
terminal=a.mode=='terminal'
if terminal:
    assert a.terminal_chunk not in [None,'','unknown','pending'] and a.exit_code is not None
    assert c['recorded_children_terminal'] and not c['owned_processes'] and not any(same(x,y) for x in live for y in rows)
    assert (a.exit_code==0 and c['status']=='executed_off_tree_haste43_live_candidate_both_profiles') or(a.exit_code!=0 and c['status']=='failed_preserved')
    assert not w['active_workloads'] or w['active_workload_sessions']=={'haste43_live_candidate':a.session}
else:
    assert a.session is not None
    assert a.terminal_chunk is None and a.exit_code is None and any(same(x,c['pipeline']) for x in live)
    assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'reader.py').write_bytes(Path(__file__).read_bytes());(out/'owned-CIM.json').write_bytes(obs.stdout)
record=dict(status='candidate_live_terminal_scope_qualification_pending' if terminal and a.exit_code==0 else 'candidate_live_failure_preserved' if terminal else 'candidate_live_running',run=str(run),source_manifest_sha256=source,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=a.exit_code,recorded_original_identities=len(rows),recorded_original_identities_terminal=terminal,candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],normal_product_modified=False,candidate_applied=False,normal_product_qualification_granted=False,GUI_verified=False,whole_goal_complete=False)
record.update(scenario=c.get('scenario','self-peer'),fixture_profile=c.get('fixture_profile','melee'))
record['controller_cancel_repair_candidate']=c.get('controller_cancel_repair_candidate')
if terminal:
    files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not any(part.startswith('checkpoint-') for part in p.relative_to(run).parts)}
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
    record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'evidence.json').write_bytes(data(record))
initial_note='原fixture既有heal19-refused初始Mage maxMP2，其餘角色／原技能書保留，無執行期authority修改；' if c.get('scenario')=='mp-refused' else '正式記憶體初始角色未改；'
work='#43加速術候選正式75隔離探針（'+c.get('scenario','self-peer')+'）'+('原批次已終止，原始結果／程序／來源保存，待獨立範圍讀回，不移記產品或GUI資格。' if terminal else '沿原批次編譯／執行，固定正常來源與對應QA；'+initial_note+'原生控制器主體只讀衍生、候選請求與選擇模組隔離引用。')+'完整library兩版、85資源與房間／入口／旅館及EOF兩版限定資格保留；完整GUI與其餘開發仍待，先自行驗證才交人工。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_native_haste43_live_candidate']=record;j.update(active_workloads={} if terminal else {'haste43_live_candidate':record},active_workload_sessions={} if terminal else {'haste43_live_candidate':a.session},source_and_corresponding_QA_frozen=not terminal,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(checkpoint=str(out),status=record['status'],normal_product_modified=False),ensure_ascii=False))
