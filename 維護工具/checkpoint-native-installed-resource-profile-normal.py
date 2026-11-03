"""Track the exact current normal build/test process without implying acceptance."""
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
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);ap.add_argument('--qualification',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-normal')and a.session>0;c=read(run/'cohort.json');terminal=a.terminal_chunk is not None
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1049 and sha(run/'source-inputs.json')==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
queue=read(ROOT/'工作佇列.json');assert queue['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']and queue['active_workload_sessions']in [{},{'catalog_normal':a.session}]
identities=[c['runner'],*c['process_history']];filters=' OR '.join('ProcessId='+str(pid)for pid in sorted({row['pid']for row in identities}));cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]);actual=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];actual=[actual]if isinstance(actual,dict)else actual
def live(row):return any(x['ProcessId']==row['pid']and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
if terminal:
    assert a.exit_code in [0,1]and a.terminal_chunk not in ['','pending','unknown']and c['owned_process']is None and not any(live(row)for row in identities)
    assert c['status']in ['executed_installed_resource_profile_normal_related_both_profiles','failed_preserved']
else:
    assert a.exit_code is None and live(c['runner'])and c['status']in ['building_installed_resource_profile_normal','compiling_installed_resource_profile_related','running_installed_resource_profile_related']
qualified=a.qualification is not None
if qualified:
    assert terminal and a.exit_code==0
    qp=a.qualification.resolve(strict=True);assert qp==(run/'qualified/qualified.json').resolve()
    q=read(qp);assert q['status']=='independently_qualified_installed_resource_profile_normal_related_both_profiles'and q['cohort_sha256']==sha(run/'cohort.json')
    assert q['source_manifest_sha256']==c['source_manifest_sha256']and q['original_session']==a.session and q['terminal_tool_chunk']==a.terminal_chunk
    assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']and q['passed_per_profile']==19 and q['actual_sessions']==0
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']and sha(q['compiled_source_archive'])==q['compiled_source_archive_sha256']
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+('terminal-'if terminal else'active-')+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='catalog_normal_related_scope_qualified_limited'if qualified else'catalog_normal_terminal_readback_pending'if terminal else'original_catalog_normal_build_and_related_tests_live',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session=a.session,runner=c['runner'],terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,recorded_owned_processes_terminal=terminal,profiles_observed=c['profiles'],qualification_granted=qualified,qualification=str(qp)if qualified else None,qualification_sha256=sha(qp)if qualified else None,source_overlays_used=False,normal_product_build=True,actual_sessions=0,current75_same_endpoint_verified=False,GUI_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record);(out/'process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
stage='已actual exit0與獨立讀回：兩版正常EXE及每版19既有冒煙案通過；新opt-in測試尚未編譯／執行。'if qualified else'已actual exit'+str(a.exit_code)+'，原程序已終止，獨立讀回仍待。'if terminal else'沿原程序建置正常兩版及19相關冒煙案，來源固定，不重啟。'
work='來源'+c['source_manifest_sha256'][:8]+'正常客戶端驗證原'+str(a.session)+'：'+stage+'當版完整library、必要原資源、新API實際封包、正常GUI租裝入口／旅館返回與傳送恢復、最終Release廣域回歸仍待。先前來源的完整library、資源、GUI及控制器限定資格保留原版；不移記、不改JAR或時限、不要求使用者操作。'

date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        for profile,info in c['profiles'].items():
            if 'normal'in info:
                assert sha(info['normal']['path'])==info['normal']['sha256'];identity[profile+'_normal_sha256']=info['normal']['sha256'];identity[profile+'_normal_source_manifest_sha256']=c['source_manifest_sha256'];identity[profile+'_normal_qualified_for_current_source']=qualified
        if all('normal'in c['profiles'].get(profile,{})for profile in ['Debug','Release']):identity['existing_normal_source_manifest_sha256']=c['source_manifest_sha256']
        identity['source_qualification_pending']=not qualified
        if qualified:
            value['current_native_installed_resource_profile_normal_qualification']=record
            value['current_source_goal_validation']['G01-current-baseline']=dict(state='catalog_normal_two_profile_related_baseline_qualified_limited',complete=True,evidence=record)
            value['current_source_goal_validation']['G03-current-regression']=dict(state='catalog_related19_each_qualified_broad_and_library_current_source_pending',complete=False,evidence=record)
            value['current_source_goal_validation']['G06-room-service-branches']=dict(state='catalog_components_qualified_current75_actual_handoff_and_other_branches_pending',complete=False,evidence=record)
            for item in value.get('items',[]):
                if item.get('id')=='G01-current-baseline':item.update(state='catalog_normal_two_profile_related_baseline_qualified_limited',complete=True,current_evidence=record,current_evidence_applies_to_current_source=True)
        value.update(current_native_installed_resource_profile_normal_work=record,active_workloads={}if terminal else{'catalog_normal':record},active_workload_sessions={}if terminal else{'catalog_normal':a.session},source_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=not terminal,next_task=work,next_focus=work,last_goal_turn_classification='progress'if terminal else'verified_wait',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='installed-resource-profile-normal-build-and-existing-protocol-smoke-tests',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        staged[n]=(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,payload in staged.items():(ROOT/n).write_bytes(payload)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=not terminal,next_action='Poll original normal build/test job. After actual terminal, independently bind both normal products and affected test names; then final-source broad/library and focused normal GUI keyboard regression; preserve prior-source qualifications and capture failures separately.'))
print(json.dumps(dict(checkpoint=str(out),original_session=a.session,terminal=terminal,qualification_granted=qualified),ensure_ascii=False))
