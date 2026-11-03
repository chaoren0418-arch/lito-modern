"""Read back original code, relocations, source binding and every queue snapshot."""
from pathlib import Path
from datetime import datetime,timezone
from collections import defaultdict
import argparse,csv,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def digest(b):return hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-queue-lifecycle')
assert a.session_id>0 and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='original_background_queue_lifecycle_executed_pending_independent_readback'
ready=r.get('include_ready_filter',False);expected_snapshots=18 if ready else 16
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
archive=read(run/'archive-readback.json');assert sha(run/'raw.zip')==archive['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(archive['files']) and set(z.namelist())==set(archive['files'])
    for name,h in archive['files'].items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
        if name!='private-x86-oracle.exe':assert sha(run/name)==h
    binary=z.read('private-x86-oracle.exe');h=struct.unpack_from('<I',binary,60)[0]
    assert binary[:2]==b'MZ' and binary[h+4:h+6]==b'\x4c\x01' and binary[h+24:h+26]==b'\x0b\x01'
    assert digest(binary)==r['helper_sha256']
PE=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
ASM=ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm'
assert sha(PE)==r['original_PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
data=PE.read_bytes();h=struct.unpack_from('<I',data,60)[0];optional=h+24;base=struct.unpack_from('<I',data,optional+28)[0]
sections=[]
for i in range(struct.unpack_from('<H',data,h+6)[0]):
    at=optional+struct.unpack_from('<H',data,h+20)[0]+40*i;sections.append(struct.unpack_from('<III',data,at+12))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
added=read(run/'added-fragments.json');assert added==r['added_fragments']
expected_added={'background':(0x4878c0,44,False),'initial_slice':(0x47e26b,21,True),
    'rearm_slice':(0x4a9946,19,True),'stop_slice':(0x47e4c0,12,True)}
if ready:expected_added['ready_filter']=(0x4d64f0,119,False)
assert set(added)==set(expected_added)
for name,(start,size,ret) in expected_added.items():
    m=added[name];assert (m['start'],m['original_bytes'],m['appended_return'])==(start,size,ret)
    assert m['original_sha256']==digest(get(start,size))
addresses={int(x,16):int(y,16) for x,y in (line.split() for line in (run/'addresses.tsv').read_text().splitlines())}
relocations=defaultdict(list)
for line in (run/'relocations.tsv').read_text().splitlines():
    name,kind,offset,target,next_offset=line.split()
    relocations[name].append((kind,int(offset),int(target,16),int(next_offset)))
functions={};original_bytes=0
for line in (run/'functions.tsv').read_text().splitlines():
    name,start,size=line.split();start=int(start,16);size=int(size);assert name not in functions
    functions[name]=(start,size);before=(run/(name+'-original.bin')).read_bytes();after=(run/(name+'-relocated.bin')).read_bytes()
    padding=1 if name in expected_added and expected_added[name][2] else 0
    original_bytes+=size-padding
    assert before==get(start,size-padding)+(b'\xc3' if padding else b'')
    allowed=set()
    for kind,offset,target,next_offset in relocations[name]:
        assert kind in ['relative','absolute'] and offset+4<=size-padding
        assert allowed.isdisjoint(range(offset,offset+4));allowed.update(range(offset,offset+4))
        expected=addresses[target]
        if kind=='relative':expected=(expected-addresses[start]-next_offset)&0xffffffff
        assert struct.unpack_from('<I',after,offset)[0]==expected
    assert len(before)==len(after)==size and all(x==y or i in allowed for i,(x,y) in enumerate(zip(before,after)))
assert len(functions)==(15 if ready else 14) and original_bytes==(1336 if ready else 1217)
keep=list(struct.unpack('<9I',(run/'keep-original.bin').read_bytes()));assert keep==list(struct.unpack('<9I',get(0x6365e0,36)))
keep[6]=addresses[0x4878c0];assert (run/'keep-relocated.bin').read_bytes()==struct.pack('<9I',*keep)
binding=read(run/'source-binding.json');assert sha(ASM)==binding['original_ASM_sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=ASM.read_bytes().decode('utf-8-sig').split('\n')
for span in binding['source_spans']:
    assert span['text']=='\n'.join(lines[span['first_LF_line']-1:span['last_LF_line']]).replace('\r','')
slot=get(0x4b3470+210,1)[0];target=struct.unpack('<I',get(0x4b3208+slot*4,4))[0]
assert binding['packet_opcode']==210 and slot==binding['packet_table_slot']
assert target==binding['packet_target']==0x4b2f43 and get(target,7).hex()==binding['packet_target_bytes']
states={}
for row in csv.DictReader((run/'state-vectors.csv').open(encoding='ascii',newline='')):
    name=row.pop('scenario');assert name not in states;states[name]=tuple(map(int,row.values()))
expected_states={
 'initial':(65485,65535,1,0,0,0,2,1000,2,0),'initial_again':(65486,65536,2,0,0,0,4,1000,4,0),
 'rearm':(65485,65535,1,0,0,0,5,1000,7,0),'stop':(65485,65535,1,0,0,0,4,1000,7,0),
 'other_owner_due':(65486,65536,1,0,0,0,4,1003,9,1),
 'map_before':(65485,65535,1,0,0,0,13,1000,13,0),'map_after':(65485,65535,1,0,0,0,11,1000,14,0),
 'scene_before':(65485,65535,1,0,0,0,14,1000,14,0),'scene_after':(65485,65535,1,0,0,0,8,1000,14,0),
 'late':(65486,65536,0,0,0,0,1,1014,3,1),'late_early':(65486,65536,0,0,0,0,1,1070,4,0),
 'late_due':(65487,65537,0,0,0,0,1,1071,6,1),'duplicate':(65488,65538,0,0,0,0,2,1021,5,1),
 'mixed':(65486,65536,2,21,0,1,3,1028,7,1),'wrap':(0,65536,0,0,0,0,1,4294967280,1,0),
 'stopped_idle':(65484,65534,0,0,0,0,0,0,0,0)}
if ready:expected_states.update(ready_before=(65485,65535,1,0,0,0,14,1000,14,0),ready_after=(65485,65535,1,0,0,0,2,1000,14,0))
assert states==expected_states
queues=defaultdict(list)
for row in csv.DictReader((run/'queue-vectors.csv').open(encoding='ascii',newline='')):
    name=row['scenario'];assert name in states and int(row['index'])==len(queues[name])
    queues[name].append((int(row['deadline']),row['callback'],int(row['owner'])))
kept=[(4000+i,'background' if i==6 else 'keep'+str(i),i+1) for i in range(9)]
expected_queues={
 'initial':[(1050,'green',0),(1064,'background',0)],
 'initial_again':[(1050,'green',0)]*2+[(1064,'background',0)]*2,
 'rearm':[(1003,'background',9),(1004,'note1',0),(1005,'phase',0),(1050,'green',0),(1064,'background',0)],
 'stop':[(1003,'background',9),(1004,'note1',0),(1005,'phase',0),(1050,'green',0)],
 'other_owner_due':[(1004,'note1',0),(1005,'phase',0),(1050,'green',0),(1067,'background',0)],
 'map_before':[(1050,'green',0),(1064,'background',0),(1070,'phase',0),(1080,'note1',0)]+kept,
 'map_after':[(1064,'background',0)]+kept+[(11000,'map_aux',0xffffffff)],
 'scene_before':[(1050,'green',0),(1060,'map_aux',0xffffffff),(1064,'background',0),(1070,'phase',0),(1080,'note1',0)]+kept,
 'scene_after':[(1064,'background',0)]+kept[:7],
 'late':[(1071,'background',0)],'late_early':[(1071,'background',0)],'late_due':[(1135,'background',0)],
 'duplicate':[(1071,'background',0),(1078,'background',0)],
 'mixed':[(1057,'green',0),(1078,'background',0),(1121,'phase',0)],'wrap':[(48,'background',0)],'stopped_idle':[]}
if ready:
    expected_queues['ready_before']=expected_queues['scene_before']
    expected_queues['ready_after']=[(4003,'keep3',4),(4004,'keep4',5)]
for name,expected in expected_queues.items():assert queues[name]==expected,(name,queues[name],expected)
for name,values in states.items():assert len(queues[name])==values[6]
native=read(run/'native-stdout.log');assert native['snapshots']==expected_snapshots and native['guard_failures']==0
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
q=dict(status='independently_qualified_original_background_queue_and_selected_lifecycle_slices',
    original_session_id=a.session_id,observed_terminal_exit_code=0,source_manifest_sha256=r['source_manifest_sha256'],
    original_functions_and_slices=len(functions),original_code_bytes=original_bytes,appended_return_instructions=3,snapshots=expected_snapshots,
    queue_rows=sum(map(len,queues.values())),original_scheduler_cancel_dispatch_and_filters_executed=True,
    D2_table_and_selected_rearm_slice_verified=True,exact_owner0_cancel_verified=True,
    background_survives_generic_map_and_scene_filters=True,clock_source_stubbed=True,
    distinct_ready_reuse_filter_executed=ready,ready_reuse_filter_removes_background=ready,
    CRT_memory_operations_stubbed=True,audio_dispatch_tail_stubbed=True,secondary_map_vector_supplied_empty=True,
    entire_entry_cleanup_or_packet_handler_executed=False,modern_scene_binding_verified=False,
    GUI_or_real_clock_timing_verified=False,product_source_changed=False,actual_formal_sessions=0,whole_goal_complete=False,
    raw_archive_sha256=archive['archive_sha256'],raw_files=archive['files'],observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'qualified.json').exists()
(run/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','original_code_bytes','snapshots','queue_rows']},ensure_ascii=False))
