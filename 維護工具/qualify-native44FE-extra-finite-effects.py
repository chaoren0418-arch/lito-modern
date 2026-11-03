"""Independent raw readback of candidate resources and retained scene regressions."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
from PIL import Image
import argparse,ctypes,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.session>0 and a.terminal_chunk not in ['pending','unknown','']
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-extra-finite-effects')
c=read(run/'receipt.json');assert c['status']=='executed_candidate_resources_pending_independent_qualification'and c['owned_process']is None and c['asset_metadata_unchanged']
assert c['source_overlays_used']and not c['normal_product_modified']and c['actual_sessions']==c['actual_casts']==0
build=Path(c['compile']);b=read(build/'receipt.json');assert sha(build/'receipt.json')==c['compile_receipt_sha256']
assert b['status']=='compiled_isolated_lazy_finite_effect_variant'and b['candidate_manifest_sha256']==c['candidate_manifest_sha256']==sha(build/'candidate-inputs.json')
assert b['base_source_manifest_sha256']==c['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in read(build/'candidate-inputs.json')['candidate_files'].items():assert sha(build/n)==h,n
for p in b['products'].values():assert sha(p['path'])==p['sha256']
assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256']
normal=read(Path(b['normal_qualification']).parent.parent/'cohort.json')
for r in normal['profiles'].values():assert sha(r['normal']['path'])==r['normal']['sha256']
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
counts={'scene23':23,'additional11':1}
if b.get('ability_request_candidate_included'):counts.update(ability5=5,ability_world2=2)
for label,count in counts.items():
    stdout=(run/(label+'.stdout')).read_text('utf-8-sig')
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',stdout)==[(str(count),'0','0','0','0')]
    assert len(re.findall(r'^test \S+ \.\.\. ok$',stdout,re.M))==count
work=Path(c['runtime']);spec=read(run/'original-effect-definitions.json');graphics=read(run/'candidate-graphics.json');result=read(work/'resource-result.json')
assert len(graphics)==11 and sorted(r['graphics']for r in result['cases'])==graphics
assert result['fixture_authority_unchanged_by_renderer']and not result['audio_device_playback_verified']and not result['gameplay_verified']
assert result['lifecycle']==dict(pre_cancel_preserves_old_model=True,same_map_new_generation_retires_bank=True,reused_sequence_fresh_sound_once=True,fresh_sound=116,late_native_callback_preserves_dwell=True)
sources={r['graphics']:r for r in spec['rows']};pictures={};total_sound=0
for row in result['cases']:
    gfx=row['graphics'];first=row['first_visible'];receipt=first['receipt'];bank=receipt['additional_bank'];final=row['final_receipt']
    assert first['pixels_changed']and bank['loaded']and bank['load_error']is None and receipt['load_error']is None
    assert bank['encoded_bytes']>0 and bank['retained_RGBA_bytes']>0 and bank['sources']and bank['sound_PCM_bytes']>0
    assert all(s['bytes']>0 and re.fullmatch('[0-9a-fA-F]{64}',s['sha256'])for s in bank['sources'])
    assert final['active']==0 and final['omitted']==0
    expected=sum(len(f['events'])for f in sources[gfx]['action0']['frames']);assert row['source_sound_events']==expected;total_sound+=expected
    if gfx==2251:
        assert row['reviewed_blank_start_without_pixels']and row['source_sound_events']==1
        pinned=next(s for s in bank['sources']if s['name']=='2251-0.spr');assert pinned['bytes']==157993 and pinned['sha256'].upper()=='B90C450146296E77043090948B11075C0731FC7B15093734DC37935A114D975B'
    p=Path(first['png']).resolve(strict=True);assert p.parent.samefile(work)and p.name==f'effect-{gfx}.png'
    with Image.open(p)as im:assert im.size==(800,600)and im.mode=='RGBA';im.load()
    pictures[gfx]=dict(path=str(p),sha256=sha(p))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetExitCodeProcess.argtypes=[wintypes.HANDLE,ctypes.POINTER(wintypes.DWORD)];k.CloseHandle.argtypes=[wintypes.HANDLE]
identities=[b['original_runner'],c['runner']]+b['process_history']+c['process_history'];assert len(identities)==(12 if b.get('ability_request_candidate_included')else 7)
for row in identities:
    if 'exit_code'in row:assert row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')
    handle=k.OpenProcess(0x1000,False,row['pid'])
    if not handle:assert ctypes.get_last_error()==87;continue
    try:
        ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        created=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
        if created==row['creation_utc']:
            code=wintypes.DWORD();assert k.GetExitCodeProcess(handle,ctypes.byref(code))and code.value!=259
    finally:k.CloseHandle(handle)
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={f'run/{p.name}':p for p in run.iterdir()if p.is_file()}
files.update({f'compile/{p.name}':p for p in build.iterdir()if p.is_file()})
files['runtime/resource-result.json']=work/'resource-result.json'
files.update({f'images/{Path(r["path"]).name}':Path(r['path'])for r in pictures.values()});pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'qualified.json',dict(status='isolated_candidate_original_resource_scope_qualified',profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],candidate_manifest_sha256=c['candidate_manifest_sha256'],
    original_session=a.session,observed_terminal_chunk=a.terminal_chunk,observed_terminal_exit_code=0,scene_regressions_passed=23,new_graphics_verified=11,original_PNGs=pictures,source_sound_events=total_sound,
    recorded_identities=len(identities),recorded_original_owned_processes_terminal=True,ability_packet_component_passed=counts.get('ability5',0),ability_world_cases_passed=counts.get('ability_world2',0),raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),source_overlays_used=True,normal_product_modified=False,
    synthetic_S37_inputs=True,SPR_decode_verified=True,PCM_decode_verified=True,audio_device_playback_verified=False,actual_sessions=0,actual_casts=0,normal_GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=c['profile'],new_graphics=11,scene_regressions=23,normal_product_modified=False),ensure_ascii=False))
