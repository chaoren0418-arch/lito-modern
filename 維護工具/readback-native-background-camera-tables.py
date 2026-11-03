"""Independently interpret the retained initializer bytes and compare native results."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-camera-tables') and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='original_camera_initializer_tables_independently_compared'
assert r['owned_process'] is None and len(r['processes'])==2 and all(v['exit_code']==0 for v in r['processes'])
retention=read(run/'archive-readback.json');assert sha(run/'raw.zip')==retention['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(retention['files']) and set(z.namelist())==set(retention['files'])
    for name,h in retention['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    helper=z.read('isolated-x86-oracle.exe');header=int.from_bytes(helper[60:64],'little')
    assert helper[:2]==b'MZ' and helper[header+4:header+6]==b'\x4c\x01' and helper[header+24:header+26]==b'\x0b\x01'
    assert hashlib.sha256(helper).hexdigest().upper()==r['helper_sha256']
PE=Path(r['original_PE']);assert sha(PE)==r['original_PE_sha256']
assert sha(ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')==r['original_ASM_sha256']
data=PE.read_bytes();u16=lambda at:struct.unpack_from('<H',data,at)[0];u32=lambda at:struct.unpack_from('<I',data,at)[0]
pe=u32(60);optional=pe+24;base=u32(optional+28);sections=[]
for i in range(u16(pe+6)):
    at=optional+u16(pe+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
addresses={k:int(v) for k,v in (line.split() for line in (run/'relocation-addresses.tsv').read_text().splitlines())}
native=read(run/'native-stdout.log');functions=read(run/'functions.json');assert functions==r['functions']
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
tables={}
for name,info in functions.items():
    original=(run/(name+'-original.bin')).read_bytes();relocated=(run/(name+'-relocated.bin')).read_bytes()
    assert original==get(info['start'],info['bytes']) and hashlib.sha256(original).hexdigest().upper()==info['sha256']
    registers={};writes={};relocations=[];decoded=list(cs.disasm(original,info['start']))
    assert sum(i.size for i in decoded)==len(original) and decoded[-1].mnemonic=='ret'
    for ins in decoded:
        ops=ins.operands
        if ins.mnemonic=='ret':continue
        if ins.mnemonic=='xor':
            assert ops[0].type==ops[1].type==capstone.x86.X86_OP_REG and ops[0].reg==ops[1].reg
            registers[ops[0].reg]=0
        else:
            assert ins.mnemonic=='mov' and len(ops)==2
            value=ops[1].imm&0xffffffff if ops[1].type==capstone.x86.X86_OP_IMM else registers[ops[1].reg]
            if ops[0].type==capstone.x86.X86_OP_REG:registers[ops[0].reg]=value
            else:
                assert ops[0].type==capstone.x86.X86_OP_MEM and ops[0].mem.base==ops[0].mem.index==0 and ops[0].size==4
                target=ops[0].mem.disp;assert target not in writes and info['table']<=target<info['table']+64
                writes[target]=value
                relocations.append(dict(offset=ins.address-info['start']+ins.disp_offset,target=target))
    assert relocations==info['relocations'] and len(writes)==16
    table=[[struct.unpack('<i',struct.pack('<I',writes[info['table']+8*heading+axis*4]))[0] for axis in range(2)] for heading in range(8)]
    tables[name]=table
    assert len(original)==len(relocated)
    patched={i for v in relocations for i in range(v['offset'],v['offset']+4)}
    assert all(x==y or i in patched for i,(x,y) in enumerate(zip(original,relocated)))
    for v in relocations:assert struct.unpack_from('<I',relocated,v['offset'])[0]==addresses['table_data']+v['target']-0x834c68
assert {row['name']:row['values'] for row in native['tables']}==tables
assert native['guard_failures']==0 and len(tables)==5 and native['heading_vectors']==40
current=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
assert sha(current/'source-inputs.json')==r['source_manifest_sha256']
for name,h in read(current/'compiled-inputs.json').items():assert sha(ROOT/name)==h
q=dict(status='independently_qualified_original_camera_initializer_tables',original_session_id=a.session_id,observed_terminal_exit_code=0,
    original_PE_sha256=r['original_PE_sha256'],original_ASM_sha256=r['original_ASM_sha256'],source_manifest_sha256=r['source_manifest_sha256'],
    initializer_functions=5,original_code_bytes=sum(v['bytes'] for v in functions.values()),heading_vectors=40,guard_failures=0,
    initializer_tables=tables,raw_archive_sha256=retention['archive_sha256'],raw_files=retention['files'],
    table_initializers_executed_in_isolated_x86_process=True,whole_walking_callback_executed=False,
    modern_Pose_to_raw_camera_verified=False,normal_scene_map303_enabled=False,product_source_changed=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'qualified.json').exists()
(run/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','initializer_functions','original_code_bytes','heading_vectors','guard_failures']},ensure_ascii=False))
