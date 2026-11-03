"""Reconcile sampled original pixel evidence while preserving the live broad job."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('readback',type=Path);ap.add_argument('--broad-run',type=Path,required=True);ap.add_argument('--session',type=int,required=True);ap.add_argument('--recovery-receipt',type=Path);a=ap.parse_args()
proof=a.readback.resolve(strict=True);assert proof.is_relative_to(V/'native-catalog-hotel-floor-provenance');p=read(proof)
assert p['status']=='current_map501_sampled_gray_pixels_have_original_floor_writers'and p['map_id']==501 and len(p['samples'])==6
assert not p['normal_source_modified']and not p['normal_GUI_verified']and not p['all_maps_framebuffer_verified']and p['actual_new_sessions']==0
for row in p['samples']:
    d=row['original_floor_writer'];assert row['RGBA']==d['TIL']['RGBA']==[82,82,82,255]and d['tile']==0 and d['slot']in [0,1]
    assert d['source_reference']==[d['slot'],0,0,0]and d['TIL']['word']=='294A'and row['other_terrain_writes_at_sample']==0
    assert row['other_terrain_rectangles_covering_sample']==len(row['transparent_object_candidates'])
    for o in row['transparent_object_candidates']:assert not o['encoded_coverage']and o['kind']in [2,3]
for n,h in p['original_resource_pins'].items():assert sha(n)==h,n
assert sha(p['PNG'])==p['PNG_sha256']and sha(p['hotel_qualification'])==p['hotel_qualification_sha256']
assert sha(proof.parent/'raw.zip')==p['raw_archive_sha256']
with zipfile.ZipFile(proof.parent/'raw.zip')as z:
    assert set(z.namelist())==set(p['raw_files'])
    for n,h in p['raw_files'].items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h,n
run=a.broad_run.resolve(strict=True);assert run.is_relative_to(V/'native-catalog-default116')and a.session>0;c=read(run/'cohort.json');w=read(R/'工作佇列.json')
assert c['status']in ['executing_current_default116_profile','independently_reading_current_default116_profile']
assert w['active_workload_sessions']=={'broad_default':a.session}and w['source_and_corresponding_QA_frozen']
assert p['source_manifest_sha256']==c['source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']
for n,h in read(Path(c['normal_qualification']).parent.parent/'compiled-inputs.json').items():assert sha(R/n)==h,n
for n,h in c['validation_pins'].items():assert sha(R/'維護工具'/n)==h,n
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(c['pipeline_pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
z=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert z.returncode==0 and not z.stderr;live=json.loads(z.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds())<.001
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=proof.parent/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'broad-process-CIM.json').write_bytes(z.stdout)
record=dict(status='map501_six_sampled_gray_pixels_original_floor_writers_qualified_limited',source_manifest_sha256=p['source_manifest_sha256'],readback=str(proof),readback_sha256=sha(proof),PNG_sha256=p['PNG_sha256'],gray_samples=6,original_floor='0.til slots0/1 RGB555 294A',overlapping_object_coverage_checked=True,raw_archive_sha256=p['raw_archive_sha256'],normal_source_modified=False,normal_GUI_verified=False,all_maps_framebuffer_verified=False,actual_new_sessions=0,whole_goal_complete=False)
if a.recovery_receipt:
    recovery=a.recovery_receipt.resolve(strict=True);assert recovery.is_relative_to(V);r=read(recovery)
    assert r['status']=='canonical8_restored_byte_exact_to_verified_live_broad_checkpoint'and r['original_broad_session']==a.session and not r['product_source_modified']
    record['prior_checkpoint_recovery']=dict(path=str(recovery),sha256=sha(recovery),failed_attempts_retained=True)
save(out/'evidence.json',record)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work=w['next_task']+' map501灰邊六個抽樣點另已由原S32與LTtile 0.til／RGB555 294A核對為原始地板寫入，非未繪製畫布；重疊203.til物件該點無覆蓋。這只解決抽樣來源疑問，完整相機／GUI及全圖像素仍待，原廣域工作繼續且未重啟。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}
for n in before:
    path=R/n
    if path.suffix=='.json':
        v=read(path);assert v['active_workload_sessions']=={'broad_default':a.session}and v['current_product_identity']['source_manifest_sha256']==p['source_manifest_sha256']
        v['current_native_catalog_map501_gray_provenance']=record
        v['current_source_goal_validation'].setdefault('G07-resource-rendering',{}).update(state='map501_six_sampled_original_floor_writers_qualified_full_resource_and_GUI_scope_pending',complete=False,sampled_gray_pixel_evidence=record)
        v.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        v['current_focus']['work']=work
        if 'updated_date'in v:v['updated_date']=date
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert set(staged)==set(before)and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
current=read(run.parent/'current-continuation.json');current.update(latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),map501_sampled_gray_pixel_evidence=record,must_poll_original_before_restart=True);save(run.parent/'current-continuation.json',current)
print(json.dumps(dict(checkpoint=str(out),gray_samples=6,original_broad_session=a.session,original_pipeline_live=True,normal_GUI_verified=False),ensure_ascii=False))
