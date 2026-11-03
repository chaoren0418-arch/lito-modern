"""Record measured progress and unapplied candidates; do not grant readiness."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile,struct
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
observations={
 'normal':V/'native8CC5-scene-cost/measurement-20261018-123916-229041/cost-observation.json',
 'resource_open':V/'native8CC5-resource-open-profile/measurement-20261018-125048-511953/cost-observation.json',
 'scene_stages':V/'native8CC5-scene-stage-profile/measurement-20261018-125753-045438/cost-observation.json',
 'catalog_v1':V/'native8CC5-scoped-catalog/measurement-20261018-130211-997873/cost-observation.json',
 'catalog_v2':V/'native8CC5-scoped-catalog-v2/measurement-20261018-130557-700443/cost-observation.json',
}
records={name:read(path) for name,path in observations.items()}
source=records['normal']['source_manifest_sha256'];assert all(r['source_manifest_sha256']==source and r['recorded_owned_all_terminal'] for r in records.values())
assert records['resource_open']['observed_wrapper_terminal_exit_code'] is None
assert all(row['first_frame_us']>8_000_000 for row in records['catalog_v2']['rows'])
for r in records.values():assert not r['normal_product_source_modified'] and not r['gameplay_qualification_granted'] and not r['G11_qualified']
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before_path=V/'native8CC5-audio-cohort/20261018-115827-262563/Release-qualified-checkpoint-20261018-122627-442376/readback.json'
before=read(before_path)['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
q=read(ROOT/'工作佇列.json');assert not q['active_workloads'] and q['source_and_corresponding_QA_frozen']
out=V/'native8CC5-scene-cost'/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
component_specs={
 'v1':('native8CC5-scoped-catalog/component-20261018-130456-947171',11,0,'077948',None),
 'v2':('native8CC5-scoped-catalog-v2/component-20261018-130806-089706',13,0,'52f8cb',44740),
}
components={};jobs=[]
for key,(name,count,exit_code,chunk,session) in component_specs.items():
    folder=V/name;r=read(folder/'receipt.json');assert r['status']==f'both_profiles_catalog_component_{count}_passed_each' and r['owned_process'] is None
    assert all(j['exit_code']==0 for j in r['process_history']) and len(r['process_history'])==4
    assert sha(folder/'component.rs')==r['component_source_sha256'] and sha(folder/'shared_action_catalog.tests.rs')==r['test_source_sha256']
    for pin in r['module_sources'].values():assert sha(pin['path'])==pin['sha256']
    for profile,pin in r['profiles'].items():
        assert sha(pin['product'])==pin['sha256'] and sha(pin['sha2']['path'])==pin['sha2']['sha256']
        assert f'{count} passed; 0 failed; 0 ignored' in (folder/(profile+'-tests.stdout.txt')).read_text('utf-8-sig')
        with Path(pin['product']).open('rb') as f:
            assert f.read(2)==b'MZ';f.seek(60);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
    files={p.relative_to(folder).as_posix():sha(p) for p in folder.iterdir() if p.is_file()}
    archive=out/(key+'-component-raw.zip')
    with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
        for n in files:z.write(folder/n,n)
    with zipfile.ZipFile(archive) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in files.items())
    components[key]=dict(path=str(folder),receipt_sha256=sha(folder/'receipt.json'),profiles=['Debug','Release'],passed_each=count,failed_each=0,ignored_each=0,observed_wrapper_exit_code=exit_code,terminal_tool_chunk=chunk,original_session_id=session,raw_archive=str(archive),raw_archive_sha256=sha(archive),raw_files=files,normal_product_source_modified=False)
    jobs += [r['runner_identity'],*r['process_history']]
ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
(out/'component-process-CIM.json').write_bytes(raw)
work='來源8CC5A560正常Model固定捕獲場景首張繪圖約8.05–10.73秒；逐段量測確認技能／效果準備是主要成本，索引重複開啟是其中一部分。隔離同Model共用動作目錄候選v2保留繪圖器32MiB與效果8MiB原上限；Debug／Release元件各13過0敗0忽略，三張固定場景像素與正常版一致；總首Frame仍8.035–8.542秒超過8秒，不具就緒資格且未套用正常來源。下一步減少同次法術資源準備重複Sprite索引解析，維持取消、來源身分、原資源／音效界限及8／60秒；候選通過後才整合正常來源及重驗。Release六分支48sessions與控制器兩版96sessions仍按原限定資格保留；正常GUI／實體音效、完整G06及G01–G12未完成。重建版與登入器產物獨立，相鄰只讀。'
record=dict(status='Debug_readiness_cost_localized_scoped_catalog_candidates_unapplied_still_over_8s',source_manifest_sha256=source,observations={name:dict(path=str(path),sha256=sha(path)) for name,path in observations.items()},components=components,component_recorded_identities=len(jobs),component_recorded_all_terminal=True,candidate_v2_first_frame_us=[row['first_frame_us'] for row in records['catalog_v2']['rows']],candidate_v2_first_draw_us=[row['first_draw_us'] for row in records['catalog_v2']['rows']],candidate_v2_fixed_pixels_match_normal=True,candidate_v2_applied=False,normal_product_source_modified=False,normal_source_and_products_unchanged=True,acceptance_8s_changed=False,acceptance_60s_changed=False,new_sessions=0,new_requests=0,gameplay_qualification_granted=False,G11_qualified=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,next_action=work)
save(out/'evidence.json',record)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==source
        x.update(current_native8CC5_scene_costs=dict(evidence=str(out/'evidence.json'),sha256=sha(out/'evidence.json'),**record),active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='Debug-first-Presentation-resource-cost-candidate',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
for name in ['native8CC5-scene-cost','native8CC5-resource-open-profile','native8CC5-scene-stage-profile','native8CC5-scoped-catalog','native8CC5-scoped-catalog-v2']:
    save(V/name/'current-continuation.json',dict(status=record['status'],source_manifest_sha256=source,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),evidence=str(out/'evidence.json'),all_recorded_measurement_processes_terminal=True,candidate_v2_applied=False,next_action=work,goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),normal_source_unchanged=True,v2_component_passed_each=13,candidate_unapplied=True,first_frame_still_over_8s=True,whole_goal_complete=False),ensure_ascii=False))
