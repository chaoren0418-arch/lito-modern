"""Bind independently read same-endpoint cases to the current source; checkpoint limited scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]; V=ROOT/'驗證紀錄/current'; F=V/'native-catalog-handoff'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
pending=read(F/'pending-two-profile-readback.json')
# Select the immutable runtime cohort through the existing pointer.
cohorts=list(F.glob('two-profile-runtime-*/cohort.json'));assert len(cohorts)==1
cp=cohorts[0];c=read(cp);assert c['status']=='two_profile_child_runtime_success_independent_readback_pending'
w=read(ROOT/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source==c['source_manifest_sha256']and not w['active_workload_sessions']
profiles={};identities=[];pictures={}
for profile,case in c['cases'].items():
    run=Path(case['run']);qp=run/'qualified/qualified.json';q=read(qp);receipt=read(run/'receipt.json')
    assert sha(run/'receipt.json')==case['receipt_sha256']and q['profile']==profile and q['source_manifest_sha256']==source
    assert q['status']=='independently_qualified_current75_same_endpoint_LOGIN_catalog_rebind_rental_scope'
    for key in ['original_session','terminal_tool_chunk']:assert q[key]==case[key]
    assert q['actual_original_wrapper_exit_code']==case['actual_wrapper_exit_code']==(1 if profile=='Debug'else 0)
    assert q['actual_client_exit_code']==q['actual_server_exit_code']==0 and q['actual_sessions']==6 and q['LOGIN_sessions']==2 and q['GAME_sessions']==4
    assert q['LOGIN_service_directories']==2 and q['GAME_service_directories']==0 and q['exact_C_MOVE']==28
    assert q['recorded_original_identities']==6 and q['recorded_original_owned_processes_terminal']
    assert not any(q[k]for k in ['source_overlays_used','diagnostic_API_extension','normal_channel_continuation_verified','nondefault_stat_continuity_verified','GUI_verified','physical_audio_verified','whole_G06_complete','whole_goal_complete'])
    if profile=='Debug':
        failure=q['wrapper_reader_failure'];assert failure and failure['retained_without_runtime_replay']and sha(failure['path'])==failure['sha256']
    else:assert q['wrapper_reader_failure']is None
    for filename,key in [('raw.zip','raw_archive_sha256'),('product.zip','product_archive_sha256')]:assert sha(qp.parent/filename)==q[key]
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
    with zipfile.ZipFile(qp.parent/'product.zip')as z:
        assert z.namelist()==['probe.exe']
        with z.open('probe.exe')as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==q['product_sha256']
    assert sha(receipt['product'])==q['product_sha256']
    build=Path(receipt['compile']);inputs=read(build/'compiled-inputs.json');assert len(inputs)==1048
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    identities.extend([receipt['runner'],*receipt['process_history'],*read(run/'compile-terminal-observation.json')['identities']])
    for role,pic in q['original_PNGs'].items():assert sha(pic['path'])==pic['sha256'];pictures[profile+'/'+role]=pic
    profiles[profile]=dict(qualification=str(qp),sha256=sha(qp),original_session=q['original_session'],terminal_tool_chunk=q['terminal_tool_chunk'],actual_original_wrapper_exit_code=q['actual_original_wrapper_exit_code'],actual_child_exit_code=0,actual_sessions=6,normal_mains=q['normal_mains'],raw_archive_sha256=q['raw_archive_sha256'],checks=q['checks'])
assert set(profiles)=={'Debug','Release'}and len(identities)==12 and len(pictures)==4
assert len({(x['pid'],x['creation_utc'])for x in identities})==12
normal=read(c['normal_related_qualification']);assert normal['source_manifest_sha256']==source and normal['passed_per_profile']==16
assert all(p['normal_mains']==normal['normal_mains']for p in profiles.values())
for profile,info in normal['profiles'].items():assert sha(info['normal']['path'])==normal['normal_mains'][profile]
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+' OR '.join('ProcessId='+str(pid)for pid in sorted({x['pid']for x in identities}))+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
proc=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert proc.returncode==0 and not proc.stderr
live=json.loads(proc.stdout.decode('utf-8-sig'))if proc.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
for x in live:
    for old in identities:
        if x['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=F/('completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'terminal-CIM.json').write_bytes(proc.stdout)
visual=dict(status='two_actual_current_source_WorldScene_PNGs_visually_read',observed_images=[pictures['Debug/a'],pictures['Release/a']],tool_chunk='45a350',observations=['800x600 map500 renders paving, fountain, lamps, NPCs and the local character with the surrounding-map widget.','The two observed profile images are pixel-identical at this captured point.'],normal_GUI_verified=False,physical_input_verified=False,physical_audio_verified=False)
save(out/'visual-readback.json',visual)
record=dict(status='independently_qualified_two_profile_same_endpoint_LOGIN_catalog_rebind_rental_limited',source_manifest_sha256=source,runtime_cohort=str(cp),runtime_cohort_sha256=sha(cp),profiles=profiles,normal_related_qualification=c['normal_related_qualification'],normal_related_qualification_sha256=sha(c['normal_related_qualification']),actual_sessions=12,actual_LOGIN_sessions=4,actual_GAME_sessions=8,exact_C_MOVE=56,original_PNGs=pictures,visual_readback=str(out/'visual-readback.json'),visual_readback_sha256=sha(out/'visual-readback.json'),recorded_original_identities=12,recorded_original_owned_processes_terminal=True,Debug_original_wrapper_failure_retained=True,runtime_reexecuted=False,source_overlays_used=False,diagnostic_API_extension=False,ordinary_channel_continuation_verified=False,nondefault_stat_continuity_verified=False,normal_GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False,next_action='Run current-source six-branch controller regression, then Presentation audio and remaining service scopes; preserve original CA00 failure and historical-source qualifications.')
save(out/'qualified.json',record);evidence=dict(**record,qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));save(out/'evidence.json',evidence)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
work='來源'+source[:8]+'同端點LOGIN目錄重新綁定已獨立核對Debug／Release兩角色接續：12條實際sessions、56筆移動、租裝全欄及關閉前權威狀態相符，4張原生WorldScene PNG綁定且兩版各一張已目視；12組記錄程序皆終止。Debug外層舊範圍名稱斷言退出1保留，client／server均0且未重跑。只通過同端點租裝服務範圍，不推廣普通頻道、非預設能力、正常GUI或實體音效。正常兩版及每版16相關測試資格維持，舊33A3廣域／library只留原來源；續驗本版六分支控制器、Presentation音效與其他服務。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立；完整G06及G01–G12仍未完成。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value['current_native_catalog_handoff_qualification']=evidence
        scope=dict(state='current_source_same_endpoint_rental_qualified_other_controller_audio_service_and_GUI_scopes_pending',complete=False,evidence=evidence)
        value['current_source_goal_validation']['G06-room-service-branches']=scope
        for item in value.get('items',[]):
            if item.get('id')=='G06-room-service-branches':item.update(**scope,current_evidence=evidence,current_evidence_applies_to_current_source=True)
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='same-endpoint-catalog-handoff-qualified-limited',work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in value:value['updated_date']=date
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(F/'current-continuation.json',dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),active_workloads={},must_poll_original_before_restart=False,next_action=record['next_action'],whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),actual_sessions=12,qualified_scope='same-endpoint rental only',whole_goal_complete=False),ensure_ascii=False))
