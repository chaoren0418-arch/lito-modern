"""Qualify the combined candidate's final scoped gates and update only trackers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def ref(p):return dict(path=str(p),sha256=sha(p))
def archive(qp,q):
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
run=V/'native-cone-cold-request-contracts/20261018-055044-447530';r=read(run/'receipt.json')
assert r['status']=='executed_both_candidate_request_contracts'and not r['normal_product_modified']
normal=V/'native-class2-normal/validation-20261018-014519-231950'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
jobs=[r['original_runner']];profiles={};files={}
for profile,session,chunk in [('Debug',17348,'42007a'),('Release',28425,'109a78')]:
    item=r['profiles'][profile];bdir=Path(item['build']);b=read(bdir/'receipt.json')
    assert sha(bdir/'receipt.json')==item['build_receipt_sha256']and b['compiler']['exit_code']==0
    assert b['profile']==profile and b['probe_kind']=='core'and len(b['candidate_inputs'])==12
    assert item['process']['exit_code']==0 and sha(b['product'])==b['product_sha256']==item['product_sha256']
    for n,h in read(bdir/'compiled-inputs.json').items():assert sha(ROOT/n)==h
    for pin in b['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']
    assert sha(b['normal_product'])==b['normal_product_sha256']
    for pin in b['libraries'].values():assert sha(pin['path'])==pin['sha256']
    report=read(run/(profile+'-result.json'));assert report['cases']==3 and not report['asset_resources_opened']and report['network_sessions']==0
    jobs.extend([item['process'],b['compiler'],b['original_runner']])
    profiles[profile]=dict(build=str(bdir),product_sha256=b['product_sha256'],cases=3,compile_original_session=session,compile_actual_exit_code=0,compile_terminal_chunk=chunk)
    for f in bdir.iterdir():
        if f.is_file():files[profile+'-build/'+f.name]=f
assert read(run/'Debug-result.json')==read(run/'Release-result.json')
ids=','.join(str(j['pid'])for j in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.check_output(['pwsh','-NoProfile','-Command',command]).decode('utf-8-sig');live=json.loads(result)if result.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
assert len(jobs)==7
for f in run.iterdir():
    if f.is_file():files['request-run/'+f.name]=f
files['qualifier-checkpoint.py']=Path(__file__)
join=V/'native-pending-join-departure/qualified-20261018-054503-521254/qualified.json';j=read(join)
assert j['cases_each']==14 and j['recorded_original_identities_all_terminal'];archive(join,j)
proposal=V/'native-cone-cold-integration/pending-join-20261018-054544-983159/proposal.json';p=read(proposal)
assert p['files']==11 and not p['normal_product_modified']
parent=Path(p['parent_cold_proposal']['path']);assert sha(parent)==p['parent_cold_proposal']['sha256']
assert all(p['proposed_sources'][n]==h for n,h in read(parent)['proposed_sources'].items())
network=[]
for name,profile,accepted in [('20261018-054751-786371/qualified-20261018-054842-833124','Release',0),('20261018-054844-269922/qualified-20261018-054936-961501','Debug',2)]:
    qp=V/'native-cone-cold-class2-regression'/name/'qualified.json';q=read(qp)
    assert q['compile_profile']==profile and q['actual_sessions']==4 and q['actual_accepted_casts']==accepted and q['recorded_owned_all_terminal']
    assert q['proposal']==str(proposal.parent)and q['proposal_sha256']==sha(proposal)
    if not accepted:assert q['per_request'][0]['feedback']=={'Rejected':278}
    archive(qp,q);network.append(dict(**ref(qp),profile=profile,sessions=4,accepted=accepted))
before_path=V/'native-cone-cold-integration/candidate-checkpoint-20261018-053922-859681/readback.json';before=read(before_path)['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=V/'native-pending-join-departure'/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
extra=[V/'native-pending-join-departure/continuation.json',V/'native1034-library-default/current-continuation.json']
backup={**{n:ROOT/n for n in before},**{'extra/'+str(i)+'.json':f for i,f in enumerate(extra)}};pins={n:sha(f)for n,f in backup.items()}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,f in backup.items():z.write(f,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
raw_pins={n:sha(f)for n,f in files.items()}
with zipfile.ZipFile(out/'request-raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(out/'request-raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in raw_pins.items())
next_action='Apply the qualified eleven-source combined overlay only inside the rebuild, carry the three request/S23/selector cases into normal tests and the fourteen join checks into a private test module. Archive/reconcile canonical8 and exact source manifests first. Then build Debug/Release through the unique entry and run affected request/selection/room/animation/motion/environment tests, current-source C02/retirement/G06, broad/default/library; parent-source evidence remains scoped and is never reassigned. No launcher/JAR writes or permission gate.'
record=dict(status='combined_cold_and_pending_join_candidate_scoped_gates_qualified_normal_unapplied',
    base_source_manifest_sha256=p['source_manifest_sha256'],proposal=ref(proposal),pending_join_contracts=ref(join),pending_join_cases_each=14,
    existing34_C02_scopes=network,existing34_C02_sessions=8,existing34_C02_accepted=2,existing34_C02_refused=1,
    request_contracts=dict(run=str(run),profiles=profiles,cases_each=3,original_outer_session=None,original_tool_completed_without_session_id=True,observed_actual_exit_code=0,terminal_tool_chunk='2dc46a',recorded_original_identities=7,recorded_original_identities_all_terminal=True,new_network_sessions=0,raw_files=raw_pins,raw_archive_sha256=sha(out/'request-raw.zip')),
    unchanged_cold_effect_body_parent_scopes=ref(V/'native-cone-cold-integration/candidate-checkpoint-20261018-053922-859681/evidence.json'),
    normal_product_modified=False,normal_product_registered=False,normal_request_support_gap=40,
    next_action=next_action,GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源86E65C7B正常版本未改。冰錐候選原24sessions／8接受2拒絕及兩版真實移動限定資格保留；入房D0／CE順序已在未改控制器重現，兩版有界修補各14案通過。合併十一檔候選另通過C02 Debug既有34正例與Release魔力不足拒絕，共8sessions／2接受1拒絕；新三項請求／S23原子性／選取身分檢查兩版各3過，7記錄程序身分終止。原37509施法前失敗完整保留。下一步只在重建版套入此候選與正常測試，再唯一入口兩版建置及當版相關／C02／G06／廣域library；40項缺口尚未縮減。登入器／正式JAR只讀且產物獨立，替換提案撤回。GUI／實體音效、完整G06及G01–G12未完成；8／60秒、Release與G11不變。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==p['source_manifest_sha256']
        value['current_native_pending_join_and_cold_candidate']=dict(**ref(out/'evidence.json'),**record)
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='combined-cold-join-normal-integration-and-exact-source-validation',work=work,applies_to_current_source=True,normal_product_modified=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for path in extra:
    value=read(path);value.update(status='combined_candidate_scoped_gates_qualified_normal_unapplied',active_profile=None,must_read_original_terminal_before_qualification=False,latest_combined_candidate_checkpoint=ref(out/'evidence.json'),next_action=next_action)
    if 'next_candidate_action'in value:value['next_candidate_action']=next_action
    if 'active_run'in value:value['completed_prior_active_run']=value.pop('active_run')
    save(path,value)
save(out/'readback.json',dict(before_files=before,backup_files=pins,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n)for n in before},extra_after=[ref(f)for f in extra],evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),normal_product_modified=False,pending_join_cases_each=14,request_cases_each=3,C02_sessions=8,whole_goal_complete=False),ensure_ascii=False))
