"""Bind exact44 original bytes and all numerical/update-order recorder cases."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,csv,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
origin=ap.add_mutually_exclusive_group(required=True);origin.add_argument('--session-id',type=int);origin.add_argument('--exec-chunk-id')
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-offset-callback') and a.terminal_exit_code==0
assert a.session_id is None or a.session_id>0
r=read(run/'receipt.json');assert r['status']=='original_background_offset_callback_compared'
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
retained=read(run/'archive-readback.json');assert sha(run/'raw.zip')==retained['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(retained['files']) and set(z.namelist())==set(retained['files'])
    for name,h in retained['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    exe=z.read('private-x86-oracle.exe');at=int.from_bytes(exe[60:64],'little')
    assert exe[:2]==b'MZ' and exe[at+4:at+6]==b'\x4c\x01' and exe[at+24:at+26]==b'\x0b\x01'
    assert hashlib.sha256(exe).hexdigest().upper()==r['helper_sha256']
PE=Path(r['original_PE']);assert sha(PE)==r['original_PE_sha256']
data=PE.read_bytes();u16=lambda n:struct.unpack_from('<H',data,n)[0];u32=lambda n:struct.unpack_from('<I',data,n)[0]
h=u32(60);optional=h+24;base=u32(optional+28);sections=[]
for i in range(u16(h+6)):
    at=optional+u16(h+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
info=read(run/'callback.json');assert info==r['callback'] and info['bytes']==44
original=(run/'callback-original.bin').read_bytes();relocated=(run/'callback-relocated.bin').read_bytes()
assert original==get(info['start'],info['bytes']) and hashlib.sha256(original).hexdigest().upper()==info['sha256']
addresses={k:int(v) for k,v in (line.split() for line in (run/'addresses.tsv').read_text().splitlines())}
allowed={i for item in info['relocations'] for i in range(item['offset'],item['offset']+4)}
assert len(original)==len(relocated) and all(x==y or i in allowed for i,(x,y) in enumerate(zip(original,relocated)))
assert [v['kind'] for v in info['relocations']].count('memory')==4
assert [v['kind'] for v in info['relocations']].count('self')==1 and [v['kind'] for v in info['relocations']].count('relative')==1
for item in info['relocations']:
    if item['kind']=='memory':expected=addresses[{0x62f4c0:'offset_x',0x62f4c4:'offset_y'}[item['target']]]
    elif item['kind']=='self':assert item['target']==0x4878c0;expected=addresses['callback']
    else:
        assert item['target']==0x4d65f0;expected=(addresses['scheduler']-addresses['callback']-item['next'])&0xffffffff
    assert struct.unpack_from('<I',relocated,item['offset'])[0]==expected
rows=list(csv.DictReader((run/'vectors.csv').open(encoding='ascii',newline='')));matrix=set();sequence=[]
for row in rows:
    group,i,x,y,ax,ay,ox,oy,delay,owner,callback,calls=map(int,row.values())
    assert (ax,ay)==((x+1)&0xffffffff,(y+1)&0xffffffff)
    assert (ox,oy)==(ax,ay) and (delay,owner,callback,calls)==(64,0,addresses['callback'],1)
    if group==0:assert i==len(matrix) and (x,y) not in matrix;matrix.add((x,y))
    else:assert group==1 and i==len(sequence);sequence.append((x,y,ax,ay))
values=[0,1,65484,65534,65535,0x7fffffff,0xffffffff]
assert matrix=={(x,y) for x in values for y in values} and len(rows)==57
assert sequence==[(65484+i,65534+i,65485+i,65535+i) for i in range(8)]
assert any(y==65535 and ay==65536 for _,y,_,ay in sequence)
native=read(run/'native-stdout.log');assert native['cases']==57 and native['guard_failures']==0 and native['scheduler_body_stubbed']
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,h in pins.items():assert sha(ROOT/name)==h
q=dict(status='independently_qualified_original_background_offset_callback',original_session_id=a.session_id,
    original_exec_tool_chunk_id=a.exec_chunk_id,observed_terminal_exit_code=0,source_manifest_sha256=r['source_manifest_sha256'],
    original_PE_sha256=r['original_PE_sha256'],original_code_bytes=44,cases=57,counter_word_bits=32,
    counter_update_before_schedule_verified=True,source_delay_ms=64,source_owner_argument=0,
    callback_self_pointer_verified=True,guard_failures=0,not_truncated_to_u16=True,
    scheduler_body_stubbed=True,actual_clock_or_queue_timing_verified=False,full_start_cancel_rearm_binding_verified=False,
    product_source_changed=False,modern_scene_binding_verified=False,actual_formal_sessions=0,GUI_verified=False,
    raw_archive_sha256=retained['archive_sha256'],raw_files=retained['files'],whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'qualified.json').exists()
(run/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','original_code_bytes','cases','source_delay_ms','counter_update_before_schedule_verified']},ensure_ascii=False))
