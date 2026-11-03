"""Read-only original gate provenance; never compiles or executes native code."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,struct,zipfile
import capstone
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-084451-903086/normal-qualified'
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
PE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1002
for n,h in pins.items():assert sha(ROOT/n)==h,n
for v in read(NORMAL/'qualified.json')['profiles'].values():assert sha(Path(v['path']))==v['sha256']
assembly=ASM.read_bytes();assert digest(assembly)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
raw=PE.read_bytes();assert digest(raw)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
lines=assembly.decode('utf-8',errors='replace').split('\n')
names=['sub_488860','sub_487E10','sub_4BCFB0','sub_4BDCE0','sub_4BDE70','sub_4BE000','sub_4BF0B0','sub_4BF500','sub_4C0390','sub_4C05D0','sub_4C14C0','sub_4C1670']
selected={}
for name in names:
    a=next(i for i,l in enumerate(lines) if l.startswith(name+' proc near'))
    b=next(i for i in range(a+1,len(lines)) if lines[i].startswith(name+' endp'))
    selected[name]={'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
refs=[]
for i,line in enumerate(lines):
    if '6476C4' in line.upper():refs.append({'newline_line':i+1,'assembly':'\n'.join(lines[max(0,i-5):i+7])})
assert len(refs)==16
world=selected['sub_488860']['assembly']
assert all(x in world for x in ['mov     edi, dword_6476C4','mov     ecx, 15CD2h','rep stosd','cmp     edi, 12Fh','call    sub_4BE000','call    sub_4C14C0','call    sub_4C1670'])
for name in ['sub_4BE000','sub_4C14C0','sub_4C1670']:
    text=selected[name]['assembly']
    assert all(x in text for x in ['mov     edx, dword_6476C4','mov     eax, 1010101h','rep stosd','rep stosb'])
for name in ['sub_4BCFB0','sub_4BF0B0','sub_4BF500','sub_4C0390','sub_4C05D0']:
    text=selected[name]['assembly'];assert re.search(r'cmp\s+byte ptr \[[^\]]+\], 0',text) and '6476C4' in text
eh=struct.unpack_from('<I',raw,60)[0];optional=eh+24;base=struct.unpack_from('<I',raw,optional+28)[0]
assert base==0x400000 and struct.unpack_from('<H',raw,optional)[0]==0x10b
sections=[struct.unpack_from('<IIII',raw,optional+struct.unpack_from('<H',raw,eh+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',raw,eh+6)[0])]
def at(va,size):
    rva=va-base
    for _,start,length,offset in sections:
        if start<=rva and rva+size<=start+length:return raw[offset+rva-start:offset+rva-start+size]
    raise ValueError(hex(va))
out=ROOT/'驗證紀錄/current/native-map303-gate-static'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True,exist_ok=False)
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
functions={}
for name,start,limit in [('rle',0x4be000,0x4be110),('right',0x4c14c0,0x4c1670),('left',0x4c1670,0x4c1820)]:
    ins=[]
    for i in cs.disasm(at(start,limit-start),start):
        ins.append(i)
        if i.mnemonic.startswith('ret'):break
    assert ins[-1].mnemonic.startswith('ret');end=ins[-1].address+ins[-1].size
    reloc=[];indirect_calls=0
    for i in ins:
        if i.mnemonic.startswith('j'):
            assert i.operands[0].type==capstone.x86.X86_OP_IMM and start<=i.operands[0].imm<end
        if i.mnemonic=='call':
            assert i.operands[0].type==capstone.x86.X86_OP_MEM and i.operands[0].mem.disp==0x80
            indirect_calls+=1
        for op in i.operands:
            if op.type==capstone.x86.X86_OP_MEM and op.mem.base==0 and op.mem.index==0:
                address=op.mem.disp&0xffffffff
                assert address in [0x6476c4,0x6476d0],(name,i.mnemonic,i.op_str)
                assert i.disp_size==4
                reloc.append({'offset':i.address-start+i.disp_offset,'original_address':address})
    assert indirect_calls==(0 if name=='rle' else 1)
    code=at(start,end-start);(out/(name+'-original.bin')).write_bytes(code)
    functions[name]={'start':start,'bytes':len(code),'sha256':digest(code),'relocations':reloc,'indirect_vtable80_calls':indirect_calls,
       'return_stack_bytes':int(ins[-1].op_str,0),'instructions':[{'address':i.address,'mnemonic':i.mnemonic,'operands':i.op_str} for i in ins]}
save(out/'selected-assembly.json',selected);save(out/'gate-global-references.json',refs);save(out/'extracted-code.json',functions)
proof={'status':'readonly_map303_gate_provenance_statically_retained','source_files':1002,'source_manifest_sha256':sha(NORMAL/'source-inputs.json'),
 'original_PE_sha256':digest(raw),'original_ASM_sha256':digest(assembly),'global_pointer':'0x6476C4','global_reference_count':len(refs),
 'allocation_bytes':0x64440,'world_frame_clear_bytes':0x15cd2*4,'addressed_row_stride_bytes':31*3*8,
 'observed_static_dataflow':['World clears the shared byte plane before terrain processing.',
 'Map303 equality is passed to terrain RLE and left/right raw tile writers.',
 'Enabled terrain writers copy colors and fill corresponding gate runs with byte1.',
 'Selected later background/sprite readers skip nonzero gate bytes.',
 'Previously isolated normal color kernels skip zero gate bytes in gated mode.'],
 'inference':'Gate provenance belongs to terrain write coverage and draw order; a generic all-opaque RGBA canvas cannot establish it.',
 'native_gate_producer_executed':False,'full_world_draw_verified':False,'map303_product_complete':False,
 'product_changes':0,'actual_new_game_sessions':0,'whole_goal_complete':False,
 'next_action':'After owned GUI exits, execute the three isolated gate writers with private plane/framebuffer and stub final surface Unlock, then compare coverage, colors, guards and exact relocated instructions. Trace map303 background resources/draw order before wiring product.'}
save(out/'audit.json',proof)
files={n:out/n for n in ['selected-assembly.json','gate-global-references.json','extracted-code.json','audit.json','rle-original.bin','right-original.bin','left-original.bin']}
files.update({'auditor.py':Path(__file__),'source-inputs.json':NORMAL/'source-inputs.json','normal-qualified.json':NORMAL/'qualified.json'})
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw-evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw-evidence.zip') as z:
    assert set(z.namelist())==set(files)
    for n,h in hashes.items():assert digest(z.read(n))==h,n
for n,h in pins.items():assert sha(ROOT/n)==h,n
proof.update(archive_readback_verified=True,archive_sha256=sha(out/'raw-evidence.zip'),file_pins=hashes,functions={n:{k:v for k,v in row.items() if k!='instructions'} for n,row in functions.items()})
save(out/'qualified-static.json',proof);save(APP/'map303-gate-static-current.json',{'path':str(out),'qualification':str(out/'qualified-static.json'),'status':proof['status'],'map303_product_complete':False})
print(json.dumps({'path':str(out),'status':proof['status'],'functions':{n:v['bytes'] for n,v in functions.items()},'archive_sha256':proof['archive_sha256']}))
