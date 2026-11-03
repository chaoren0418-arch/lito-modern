"""Record or independently qualify only the explicit shared_resource71 resource selection."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,subprocess,time,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
LEGACY={'modern_lito::disintegrate_effect::tests::shared_one_shots_match_csharp_composites_and_release_owned_assets', 'modern_lito::world_scene::tests::real_disintegrate_layers_share_world_depth_and_survive_target_removal', 'modern_lito::world_runtime::npc_fire_storm_runtime_tests::actual_npc70_runtime_preloads_once_and_renders_one_owned_group_without_replay', 'modern_lito::disintegrate_assets::tests::all_four_shared_sprites_match_independent_csharp_pixels', 'modern_lito::tw_action_catalog::tests::shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback'}
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(v):return(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int)
ap.add_argument('--mode',choices=['active','terminal'],required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--terminal-exit-code',type=int)
a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-shared-resource-optin')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
base=Path(c['library']);assert sha(base/'completion-observation.json')==c['library_completion_sha256']
assert w['current_full_default_library_verified']and w['current_native_catalog_library']['run']==str(base)
assert sha(run/'runner.py')==sha(R/'維護工具/run-installed-shared-resource-library.py')
for n,h in read(base/'compiled-inputs.json').items():assert sha(R/n)==h,n
rows=[c['pipeline'],*c['profiles'].values()];ids=','.join(str(row['pid'])for row in rows)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+' OR '.join('ProcessId='+str(row['pid'])for row in rows)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert probe.returncode==0 and not probe.stderr
actual=json.loads(probe.stdout.decode('utf-8-sig'))if probe.stdout.strip()else[];actual=[actual]if isinstance(actual,dict)else actual
live=[row for row in rows if any(x['ProcessId']==row['pid']and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)]
profiles={};qualified=False
if a.mode=='active':
    assert a.session is not None and a.session>0
    assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
    assert c['status']=='running_explicit_shared_resource71_resource_optin'and c['pipeline']in live
    assert a.terminal_chunk is None and a.terminal_exit_code is None
else:
    assert a.terminal_chunk not in [None,'','pending','unknown']and a.terminal_exit_code is not None and not live
    assert (w['active_workload_sessions']=={'library_shared_resource_optin':a.session}
            or (not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']))
    if a.terminal_exit_code==0:
        assert c['status']=='executed_both_current_shared_resource71_resource_optin_selections'and c['owned_process']is None
        for profile in ['Debug','Release']:
            row=c['profiles'][profile];q=read(row['qualification']);product=read(base/profile/'product-retention.json')
            assert row['exit_code']==0 and q['source_manifest_sha256']==source
            assert sha(row['qualification'])==row['qualification_sha256']
            assert row['product_sha256']==q['actual_test_sha256']==sha(product['product'])
            audit=read(base/profile/'inventory-audit/ignored-source-reasons.json')
            plan=read(run/'plan.json');assert sha(run/'plan.json')==c['plan_sha256']==sha(c['plan'])
            assert sha(run/'test-bodies.json')==plan['selected_bodies_sha256']
            names={x['test']for x in plan['selected_tests']};assert len(names)==71
            available={x['test']for x in audit['tests']if len(x.get('candidates',[]))==1 and 'LITO_SHARED_RESOURCE_ROOT'in x['candidates'][0]['reason']}
            assert len(available)==76 and LEGACY<=available and names==available-LEGACY
            assert sha(run/'shared-helper-bodies.json')==plan['shared_helpers_sha256']
            review=read(run/'review.json');assert review['review_completed']and review['plan_sha256']==sha(run/'plan.json')and review['source_manifest_sha256']==source
            assert sha(run/'review.json')==c['review_sha256']
            expected={x['test']:x for x in audit['tests']if x['test']in names}
            assert set(expected)==names
            listed=(run/profile/'selection-list.txt').read_text('utf-8-sig')
            assert {line[:-6]for line in listed.splitlines()if line.endswith(': test')}==names
            assert row['command']==[str(Path(product['product']).resolve()),*sorted(names),'--exact','--ignored','--test-threads=1']
            assert c['resource_metadata_unchanged'] and c['diagnostic_historical_outputs_restored']
            assert len(row['diagnostic_PNGs'])>7
            for n,h in row['diagnostic_PNGs'].items():
                png=run/profile/'diagnostic-PNG'/n;assert sha(png)==h
                header=png.read_bytes()[:24];assert header[:8]==b'\x89PNG\r\n\x1a\n'and header[12:16]==b'IHDR'
                assert 0<int.from_bytes(header[16:20],'big')<=4096 and 0<int.from_bytes(header[20:24],'big')<=4096
            assert len(expected)==71 and row['expected_tests']==expected
            log=(run/profile/'tests.log').read_text('utf-8-sig')
            found=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',log,re.M)
            assert len(found)==71 and {n for n,_ in found}==set(expected)and all(result=='ok'for _,result in found)
            assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('ok','71','0','0','0',str(q['inventory']-71))]
            assert sha(run/profile/'tests.log')==row['stdout_sha256']and sha(run/profile/'stderr.log')==row['stderr_sha256']
            profiles[profile]=dict(passed=71,failed=0,ignored=0,selected_tests=list(expected),actual_test_sha256=row['product_sha256'],
                default_passed=q['passed'],default_ignored=q['ignored'],remaining_unexecuted_default_ignored=q['ignored']-71,
                default_library_qualification=row['qualification'],default_library_qualification_sha256=row['qualification_sha256'])
        qualified=True
    else:assert c['status']=='failed_preserved'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(probe.stdout)
record=dict(status='independently_qualified_both_current_shared_resource71_resource_optin_limited'if qualified else'current_shared_resource71_resource_optin_running'if a.mode=='active'else'current_shared_resource71_resource_optin_failed_preserved',
    run=str(run),source_manifest_sha256=source,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=a.terminal_exit_code,
    qualified=qualified,returned_without_persistent_session=a.session is None,profiles=profiles,recorded_original_identities=len(rows),recorded_original_owned_processes_terminal=a.mode=='terminal',
    GUI_verified=False,physical_audio_verified=False,live_cast_verified=False,historical_TLS_SQL_executed=False,whole_goal_complete=False)
if qualified:
    record['remaining_unexecuted_ignored_after_disjoint_native77']=407 if w.get('current_native_library_resource_optin',{}).get('qualified')else None
    files={'cohort.json':run/'cohort.json','runner.py':run/'runner.py','checkpoint.py':out/'checkpoint.py','original-owned-CIM.json':out/'original-owned-CIM.json'}
    for profile in ['Debug','Release']:
        for name in ['tests.log','stderr.log']:files[profile+'/'+name]=run/profile/name
    for profile in ['Debug','Release']:
        for n in ['selection-help.txt','selection-list.txt']:files[profile+'/'+n]=run/profile/n
        for n in c['profiles'][profile]['diagnostic_PNGs']:files[profile+'/diagnostic-PNG/'+n]=run/profile/'diagnostic-PNG'/n
    for n in ['plan.json','test-bodies.json','shared-helper-bodies.json','review.json','resource-metadata-before.json','diagnostic-before.json','diagnostic-before.zip']:files[n]=run/n
    hashes={n:sha(p)for n,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
    record.update(raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
    (out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record))
work=('當版完整library兩版預設資格保持；'+('原 session '+str(a.session)if a.session is not None else'當次同步工具呼叫')+'之額外現行共用資源71項資源測試'+('兩版各71過0敗，已逐名／來源／實際PE／原始日誌與全部記錄程序終止獨立核對。'if qualified else'執行中，尚未授予資格。'if a.mode=='active'else'已失敗並保留原始收據。')+'忽略本文另列、非GUI／實體裝置／真實施法資格；接續正常GUI全流程與剩餘開發，先自行測試才交人工，不要求使用者登入。')
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);assert v['current_product_identity']['source_manifest_sha256']==source
        v['current_native_library_shared_resource_optin']=record
        v.update(active_workloads={'library_shared_resource_optin':record}if a.mode=='active'else{},active_workload_sessions={'library_shared_resource_optin':a.session}if a.mode=='active'else{},
            source_and_corresponding_QA_frozen=a.mode=='active',source_freeze_while_Cargo=False,
            next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='current_native_library_shared_resource_optin',work=work,applies_to_current_source=True,goal_complete=False)
        v['current_source_goal_validation'].setdefault('G03-current-regression',{}).update(complete=False,resource_optin=record)
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),qualified=qualified,profiles=profiles,original_session=a.session),ensure_ascii=False))
