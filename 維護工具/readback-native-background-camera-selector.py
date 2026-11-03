"""Independent source/relocation/case-domain readback of the camera increment fragment."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,csv,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
origin=ap.add_mutually_exclusive_group(required=True);origin.add_argument('--session-id',type=int);origin.add_argument('--exec-chunk-id')
ap.add_argument('--terminal-exit-code',type=int,required=True)
ap.add_argument('--qualification-name',choices=['qualified.json','qualified-origin-corrected.json'],default='qualified.json');a=ap.parse_args()
assert a.session_id is None or a.session_id>0
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-camera-selector') and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='isolated_original_camera_increment_selector_compared'
assert r['owned_process'] is None and len(r['processes'])==2 and all(v['exit_code']==0 for v in r['processes'])
retained=read(run/'archive-readback.json');assert sha(run/'raw.zip')==retained['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(retained['files']) and set(z.namelist())==set(retained['files'])
    for name,h in retained['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    helper=z.read('isolated-x86-oracle.exe');at=int.from_bytes(helper[60:64],'little')
    assert helper[:2]==b'MZ' and helper[at+4:at+6]==b'\x4c\x01' and helper[at+24:at+26]==b'\x0b\x01'
    assert hashlib.sha256(helper).hexdigest().upper()==r['helper_sha256']
iq=Path(r['initializer_qualification']);q=read(iq)
assert sha(iq)==r['initializer_qualification_sha256'] and q['status']=='independently_qualified_original_camera_initializer_tables'
PE=Path(r['original_PE']);assert sha(PE)==r['original_PE_sha256']==q['original_PE_sha256']
data=PE.read_bytes();u16=lambda n:struct.unpack_from('<H',data,n)[0];u32=lambda n:struct.unpack_from('<I',data,n)[0]
header=u32(60);optional=header+24;base=u32(optional+28);sections=[]
for i in range(u16(header+6)):
    at=optional+u16(header+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
addresses={k:int(v) for k,v in (line.split() for line in (run/'addresses.tsv').read_text().splitlines())}
functions=read(run/'functions.json');assert functions==r['functions'] and len(functions)==6
for name,info in functions.items():
    raw=(run/(name+'-original.bin')).read_bytes();relocated=(run/(name+'-relocated.bin')).read_bytes()
    assert raw==get(info['start'],info['bytes']) and hashlib.sha256(raw).hexdigest().upper()==info['sha256']
    assert len(relocated)==len(raw)+1 and relocated[-1]==0xc3
    allowed={i for item in info['relocations'] for i in range(item['offset'],item['offset']+4)}
    assert all(x==y or i in allowed for i,(x,y) in enumerate(zip(raw,relocated)))
    for item in info['relocations']:
        target=item['target']
        if item['kind']=='memory':
            if 0x834c68<=target<0x834da8:expected=addresses['tables']+target-0x834c68
            else:expected=addresses[{0x75a998:'camera_x',0x75a99c:'camera_y',0x834ee4:'actor_pointer'}[target]]
        else:
            assert item['kind']=='relative' and name=='selector'
            destination=addresses['scroll_stub'] if target==0x4874d0 else addresses['fallback_stub'] if target==0x4e8e30 else None
            if destination is None:
                assert target==0x4e2d07;destination=addresses[name]+info['bytes']
            expected=(destination-addresses[name]-item['next'])&0xffffffff
        assert struct.unpack_from('<I',relocated,item['offset'])[0]==expected
rows=list(csv.DictReader((run/'vectors.csv').open(encoding='ascii',newline='')));domain=set()
for row in rows:
    count,heading,phase,bx,by,ax,ay,sx,sy,calls,fallback=map(int,row.values())
    key=(count,heading,phase,bx,by);assert key not in domain;domain.add(key)
    if count==12:table='table_834D28'
    elif count==8:table='table_834CA8' if phase%2 else 'table_834C68'
    elif count==6:table='table_834CE8'
    else:assert count==4;table='table_834D68'
    dx,dy=q['initializer_tables'][table][heading]
    assert (ax,ay)==(bx+dx,by+dy) and (sx,sy)==(-dx,-dy) and calls==1 and fallback==0
assert domain=={(n,h,p,x,y) for n in [4,6,8,12] for h in range(8) for p in [1,2] for x,y in [(48,48),(0,0),(-4096,4096)]}
native=read(run/'native-stdout.log');assert native['cases']==len(rows)==192 and native['guard_failures']==0
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,h in pins.items():assert sha(ROOT/name)==h
result=dict(status='independently_qualified_original_camera_increment_selector_fragment',original_session_id=a.session_id,observed_terminal_exit_code=0,
    original_exec_tool_chunk_id=a.exec_chunk_id,
    source_manifest_sha256=r['source_manifest_sha256'],original_PE_sha256=r['original_PE_sha256'],initializer_qualification_sha256=sha(iq),
    selector_start=functions['selector']['start'],selector_end=functions['selector']['end'],selector_original_bytes=functions['selector']['bytes'],
    original_initializer_bytes=sum(v['bytes'] for name,v in functions.items() if name!='selector'),
    cases=192,source_counts=[4,6,8,12],heading_cases=8,actor13_representatives=[1,2],camera_origins=[[48,48],[0,0],[-4096,4096]],
    camera_addition_and_negated_scroll_arguments_match=True,actor_and_table_writes=0,guard_failures=0,scroll_tail_stubbed=True,
    fallback_branch_reached=0,other_source_counts_qualified=False,first_and_final_callback_paths_executed=False,
    whole_callback_executed=False,modern_Pose_binding_verified=False,normal_scene_map303_enabled=False,
    raw_archive_sha256=retained['archive_sha256'],raw_files=retained['files'],product_source_changed=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/a.qualification_name).exists()
(run/a.qualification_name).write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/('independent-reader.py' if a.qualification_name=='qualified.json' else 'independent-reader-origin-corrected.py')).write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:result[k] for k in ['status','selector_original_bytes','cases','guard_failures']},ensure_ascii=False))
