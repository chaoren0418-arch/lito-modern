"""Seal five sampled pixel writers and original native source identities, not GUI coverage."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.terminal_chunk not in ['','pending','unknown'];run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-map531-provenance')
c=read(run/'receipt.json');assert c['status']=='normal_API_pixel_source_inspection_completed'and c['owned_process']is None and not c['source_overlays_used']and not c['normal_source_modified']and c['actual_sessions']==0
b=read(Path(c['normal_library'])/'receipt.json');assert not b['source_overlays_used']and b['source_manifest_sha256']==c['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
inputs=read(Path(c['normal_library'])/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
for path,h in c['original_native_source_pins'].items():assert sha(path)==h,path
assert len(c['original_native_source_pins'])==3 and sha(c['reference_qualification'])==c['reference_qualification_sha256']
q=read(c['reference_qualification']);proof=read(run/'readback.json');assert sha(run/'raw.zip')==proof['raw_archive_sha256']
with zipfile.ZipFile(run/'raw.zip')as z:
    for n,h in proof['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
d=read(run/'result.json');assert sha(run/'result.json')==c['result_sha256']and d['full_RGBA_match']and d['map_id']==531 and d['reference_png_sha256']==c['reference_PNG_sha256']
assert d['reference_png_sha256']==q['raw_files']['owned-output/native-multisector-batch/cpu-531-seam-after.png']and not d['all_map_visibility_qualified']and not d['normal_GUI_verified']
gray=d['points'][:4];brown=d['points'][4];assert len(d['points'])==5
for p in gray:
    assert p['RGBA']==[82,82,82,255]and len(p['writes'])==1
    w=p['writes'][0];assert w['tile_entry']==0 and w['sub_index']in [0,1]and w['depth']is None and w['source_RGBA']==p['RGBA']
    assert len(w['sector_candidates'])==1;cell=w['sector_candidates'][0]['original_floor_reference']
    assert cell['tile_id']==0 and cell['subtile']==w['sub_index']and cell['source_bytes']==[w['sub_index'],0,0,0]
assert brown['RGBA']==[165,132,90,255]and brown['writes'][-1]['tile_entry']==36
member=d['original_gray_member'];assert member['both_original_archive_pins_verified']and member['member']=='0.til'
# Re-read the actual Native old28 index and bank member independently.
index=Path(member['index']).read_bytes();count=int.from_bytes(index[:4],'little');assert len(index)==4+count*28
rows=[index[i:i+28]for i in range(4,len(index),28)];row=next(row for row in rows if row[4:24].split(b'\0',1)[0].lower()==b'0.til')
offset=int.from_bytes(row[:4],'little');size=int.from_bytes(row[24:28],'little');assert (offset,size)==(member['member_offset'],member['member_bytes'])
with Path(member['pak']).open('rb')as f:f.seek(offset);raw=f.read(size)
assert hashlib.sha256(raw).hexdigest().upper()==member['member_sha256'];blocks=int.from_bytes(raw[:4],'little');start=4+(blocks+1)*4
for slot in member['source_slots']:
    i=slot['slot'];begin=int.from_bytes(raw[4+i*4:8+i*4],'little');end=int.from_bytes(raw[8+i*4:12+i*4],'little');block=raw[start+begin:start+end]
    assert len(block)==577 and block[0]==i and hashlib.sha256(block).hexdigest().upper()==slot['block_sha256']
    assert block[1:]==bytes.fromhex('4a29')*288 and slot['source_channel5']==[10,10,10]
assert len(c['process_history'])==2 and all(j['exit_code']==0 and not j.get('cleanup_stopped_original_owned_handle')for j in c['process_history'])
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
for j in [c['runner']]+c['process_history']:
    h=k.OpenProcess(0x1000|0x100000,False,j['pid'])
    if not h:assert ctypes.get_last_error()==87;continue
    try:
        ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        born=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat();assert born!=j['creation_utc']or k.WaitForSingleObject(h,0)==0
    finally:k.CloseHandle(h)
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='sampled_map531_gray_pixels_have_original_native_floor_and_TIL_writers',source_manifest_sha256=c['source_manifest_sha256'],reference_qualification=c['reference_qualification'],reference_qualification_sha256=c['reference_qualification_sha256'],
    terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,recorded_original_owned_identities=3,recorded_original_owned_processes_terminal=True,original_native_source_pins=c['original_native_source_pins'],
    gray_samples=4,non_gray_control_samples=1,gray_original_member='0.til',gray_source_slots=[0,1],gray_original_RGB555_word='294A',gray_RGBA=[82,82,82,255],same_normal_API_full_RGBA_match=True,
    result=str(run/'result.json'),result_sha256=sha(run/'result.json'),raw_archive_sha256=sha(run/'raw.zip'),readback_sha256=sha(run/'readback.json'),runtime_source_overlays_used=False,normal_source_modified=False,
    conclusion='Four sampled gray pixels are original floor references to Native LTtile 0.til slots0/1, each with an opaque original RGB555 writer. They are not untouched canvas pixels. This does not establish all normal-GUI map/camera bounds.',
    remaining_followup='Final normal renderer, camera bounds, navigation/admission and all-map visibility remain unqualified.',actual_sessions=0,normal_GUI_verified=False,all_map_visibility_qualified=False,whole_goal_complete=False)
save(out/'qualified.json',result)
print(json.dumps(dict(qualification=str(out/'qualified.json'),gray_original_tile='0.til:0/1',RGB555='294A',sampled_gray_pixels=4,normal_GUI_verified=False),ensure_ascii=False))
