"""Execute original queue/callback bodies and selected lifecycle slices privately."""
from pathlib import Path
from datetime import datetime, timezone
import capstone
import argparse
import csv
import hashlib
import json
import os
import struct
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT/'LiTo381-Modern'
OLD = ROOT/'驗證紀錄/current/native-actor-green-clock-original/20261016-150226-468757'
BUILD = ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
PE = ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
ASM = ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm'
def read(p): return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def digest(b): return hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as stream: return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(p,v): p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--include-ready-filter',action='store_true');args=ap.parse_args()
expected_functions=15 if args.include_ready_filter else 14
expected_snapshots=18 if args.include_ready_filter else 16
assert sha(PE) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
old_q, old_archive = read(OLD/'qualified.json'), read(OLD/'run-archive-readback.json')
assert old_q['status'] == 'independently_verified_original_green_callback_and_private_queue'
assert sha(OLD/'run-raw.zip') == old_q['archive_sha256'] == old_archive['archive_sha256']
with zipfile.ZipFile(OLD/'run-raw.zip') as z:
    assert set(z.namelist()) == set(old_archive['files'])
    for name, h in old_archive['files'].items(): assert digest(z.read(name)) == h
    prefix = z.read('oracle.cpp').decode('utf-8-sig').replace('\r\n','\n').split('static void direct_green()',1)[0]
    functions = z.read('functions.tsv').decode().strip()+'\n'
    relocations = z.read('relocations.tsv').decode().strip()+'\n'
    originals = {row.split()[0]+'-original.bin':z.read(row.split()[0]+'-original.bin') for row in functions.splitlines()}
    originals['keep-original.bin'] = z.read('keep-original.bin')
assert 'int wmain' not in prefix and len(originals)==11
def replace_once(old,new):
    global prefix
    assert prefix.count(old)==1, old
    prefix=prefix.replace(old,new,1)
replace_once(',green_counter;',',green_counter,background_x,background_y;')
replace_once('std::array<Slot*,12>','std::array<Slot*,14>')
replace_once('&green_counter};','&green_counter,&background_x,&background_y};')
replace_once('f==address(code.at("green"))||','f==address(code.at("background"))||f==address(code.at("green"))||')
replace_once('    if(callback==address(code.at("green")))return "green";',
    '    if(callback==address(code.at("background")))return "background";\n    if(callback==address(code.at("green")))return "green";')
replace_once('std::map<U,U> targets{{0x75ac40,',
    'std::map<U,U> targets{{0x62f4c0,address(&background_x.value)},{0x62f4c4,address(&background_y.value)},{0x75ac40,')
replace_once('require(code.size()==10,"Fragment inventory mismatch");',
    f'require(code.size()=={expected_functions},"Fragment inventory mismatch");\n    keep[6]=address(code.at("background"));\n    std::ofstream kept(dir/L"keep-relocated.bin",std::ios::binary);kept.write(reinterpret_cast<char*>(keep.data()),sizeof(keep));kept.close();')
if args.include_ready_filter:prefix='#define BACKGROUND_READY_FILTER 1\n'+prefix
data=PE.read_bytes();h=struct.unpack_from('<I',data,60)[0];optional=h+24;base=struct.unpack_from('<I',data,optional+28)[0]
sections=[]
for i in range(struct.unpack_from('<H',data,h+6)[0]):
    at=optional+struct.unpack_from('<H',data,h+20)[0]+40*i
    sections.append(struct.unpack_from('<III',data,at+12))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
for row in functions.splitlines():
    name,start,size=row.split();assert originals[name+'-original.bin']==get(int(start,16),int(size))
assert originals['keep-original.bin']==get(0x6365e0,36)
specs={'background':(0x4878c0,0x4878ec,False),'initial_slice':(0x47e26b,0x47e280,True),
       'rearm_slice':(0x4a9946,0x4a9959,True),'stop_slice':(0x47e4c0,0x47e4cc,True)}
if args.include_ready_filter:specs['ready_filter']=(0x4d64f0,0x4d6567,False)
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
added={}
for name,(start,end,append_return) in specs.items():
    raw=get(start,end-start);instructions=list(cs.disasm(raw,start))
    assert sum(i.size for i in instructions)==len(raw)
    allowed_mnemonics=['mov','inc','xor','push','call','ret']
    if name=='ready_filter':allowed_mnemonics+=['pop','cmp','jle','je','lea','shl','sub','add','dec','jmp','jl']
    assert all(i.mnemonic in allowed_mnemonics for i in instructions)
    rows=[]
    for ins in instructions:
        for op in ins.operands:
            if op.type==capstone.x86.X86_OP_MEM:
                if op.mem.base==op.mem.index==0:
                    memory=[0x62f4c0,0x62f4c4,0x64bc2d]
                    if name=='ready_filter':memory+=[0x834c00,0x834c08,0x4f7228]
                    assert op.mem.disp in memory and ins.disp_size==4
                    rows.append(dict(type='absolute',offset=ins.address-start+ins.disp_offset,target=op.mem.disp,next=0))
            elif op.type==capstone.x86.X86_OP_IMM:
                if ins.mnemonic.startswith('j'):
                    assert start<=op.imm<end
                elif ins.mnemonic=='call':
                    assert op.imm in [0x4d65f0,0x4d6870,0x4878c0,0x4878f0] and ins.imm_size==4
                    rows.append(dict(type='relative',offset=ins.address-start+ins.imm_offset,target=op.imm,next=ins.address-start+ins.size))
                elif op.imm>=0x400000:
                    if name=='ready_filter' and op.imm in [0x4159b0,0x4c32a0]:continue
                    assert op.imm==0x4878c0 and ins.imm_size==4
                    rows.append(dict(type='absolute',offset=ins.address-start+ins.imm_offset,target=op.imm,next=0))
    assert len({r['offset'] for r in rows})==len(rows)
    executable=raw+(b'\xc3' if append_return else b'')
    originals[name+'-original.bin']=executable
    functions+=f'{name} {start:x} {len(executable)}\n'
    relocations+=''.join(f"{name} {r['type']} {r['offset']} {r['target']:x} {r['next']}\n" for r in rows)
    added[name]=dict(start=start,original_bytes=len(raw),appended_return=append_return,original_sha256=digest(raw),relocations=rows,
        instructions=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,bytes=i.bytes.hex()) for i in instructions])
slot=get(0x4b3470+0xd2,1)[0];target=struct.unpack('<I',get(0x4b3208+slot*4,4))[0]
assert target==0x4b2f43 and get(target,7)==bytes.fromhex('8bcee85669ffff')
pins=read(BUILD/'compiled-inputs.json');assert len(pins)==1025
def stable():
    for name,value in pins.items():assert sha(ROOT/name)==value,name
stable()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-queue-lifecycle'/stamp
runtime=PROJECT/'target/current/native-background-queue-lifecycle'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
cases=ROOT/'維護工具/oracle/NativeBackgroundQueueCases.cpp.inc'
for name,content in originals.items():(out/name).write_bytes(content)
(out/'oracle.cpp').write_text(prefix+cases.read_text(encoding='utf-8-sig'),encoding='utf-8',newline='\n')
(out/'functions.tsv').write_text(functions,encoding='ascii')
(out/'relocations.tsv').write_text(relocations,encoding='ascii')
for name,path in [('runner.py',Path(__file__)),('cases.cpp.inc',cases),('compiled-inputs.json',BUILD/'compiled-inputs.json'),
                  ('base-qualification.json',OLD/'qualified.json'),('base-archive-readback.json',OLD/'run-archive-readback.json')]:
    (out/name).write_bytes(path.read_bytes())
asm_lines=ASM.read_bytes().decode('utf-8-sig').split('\n')
spans=[(269148,269163),(269393,269413),(354111,354162),(372380,372418),(373834,373844),(449161,449210)]
save(out/'source-binding.json',dict(original_ASM_sha256=sha(ASM),original_PE_sha256=sha(PE),
    packet_opcode=210,packet_table_byte_address=0x4b3470+210,packet_table_slot=slot,packet_target=target,
    packet_target_bytes=get(target,7).hex(),
    source_spans=[dict(first_LF_line=a,last_LF_line=b,text='\n'.join(asm_lines[a-1:b]).replace('\r','')) for a,b in spans],
    initial_callback_site=0x47e274,rearm_callback_site=0x4a9954,stop_cancel_site=0x47e4c7,
    entire_entry_cleanup_or_packet_handler_executed=False,modern_controller_binding_verified=False))
save(out/'added-fragments.json',added)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[v.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(PROJECT/'target/current/native-lighting-source-oracle'),str(runtime)) for v in command]
assert str(out/'oracle.cpp') in command
save(out/'build-command.json',command)
state=dict(status='prepared_private_background_queue_lifecycle',output=str(out),runtime=str(runtime),
    source_manifest_sha256=sha(BUILD/'source-inputs.json'),original_PE_sha256=sha(PE),base_archive_sha256=sha(OLD/'run-raw.zip'),
    original_functions_and_slices=expected_functions,added_fragments=added,include_ready_filter=args.include_ready_filter,
    clock_source_stubbed=True,CRT_memory_operations_stubbed=True,
    audio_dispatch_tail_stubbed=True,secondary_map_vector_supplied_empty=True,
    entire_entry_cleanup_or_packet_handler_executed=False,modern_scene_binding_verified=False,
    product_source_changed=False,actual_formal_sessions=0,whole_goal_complete=False,processes=[],owned_process=None)
def persist():save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_private_background_queue_lifecycle';persist();execute(command,'compile')
    binary=(runtime/'oracle.exe').read_bytes();bh=struct.unpack_from('<I',binary,60)[0]
    assert binary[bh+4:bh+6]==b'\x4c\x01' and binary[bh+24:bh+26]==b'\x0b\x01'
    state['status']='executing_original_background_queue_lifecycle';persist();execute([str(runtime/'oracle.exe'),str(out)],'native')
    result=read(out/'native-stdout.log');assert result['snapshots']==expected_snapshots and result['guard_failures']==0
    states=list(csv.DictReader((out/'state-vectors.csv').open(encoding='ascii',newline='')))
    assert len(states)==expected_snapshots and len({r['scenario'] for r in states})==expected_snapshots
    stable();state.update(status='original_background_queue_lifecycle_executed_pending_independent_readback',snapshots=expected_snapshots,
        helper_sha256=sha(runtime/'oracle.exe'));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if (runtime/'oracle.exe').exists():files['private-x86-oracle.exe']=runtime/'oracle.exe'
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,value in hashes.items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==value
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
