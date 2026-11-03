"""Stage read-only native clock fragments; preparation never builds or executes."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-071612-845832/normal-qualified'
PE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
CPP=ROOT/'維護工具/oracle/NativeWaterClockOracle.cpp'
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==999
for name,h in pins.items():assert digest((ROOT/name).read_bytes())==h,name
for value in read(NORMAL/'qualified.json')['profiles'].values():assert digest(Path(value['path']).read_bytes())==value['sha256']
data=PE.read_bytes();assert digest(data)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assembly=ASM.read_bytes();assert digest(assembly)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=assembly.decode('utf-8',errors='replace').split('\n')
u16=lambda at:struct.unpack_from('<H',data,at)[0]
u32=lambda at:struct.unpack_from('<I',data,at)[0]
pe=u32(60);optional=pe+24;base=u32(optional+28);assert base==0x400000 and u16(optional)==0x10b
sections=[]
for i in range(u16(pe+6)):
    at=optional+u16(pe+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    rva=va-base
    for start,length,offset in sections:
        if start<=rva and rva+size<=start+length:return data[offset+rva-start:offset+rva-start+size]
    raise ValueError(hex(va))
specs={'scroll':(0x4874d0,0x487510),'phase':(0x487510,0x487590),
       'schedule':(0x4d65f0,0x4d66f0),'dispatch':(0x4d6570,0x4d65f0),
       'cancel':(0x4d6870,0x4d68f0),'mapclear':(0x4d63a0,0x4d6470),
       'map_aux':(0x482370,0x4823a0),'present':(0x4d67f0,0x4d6830)}
absolute={0x64bc2d,0x75ac08,0x75ac1c,0x75ac20,0x75ac24,0x834c00,0x834c04,0x834c08,
          0x64b484,0x64b488,0x4f7228,0x6365e0,0x6365fc,0x636604,0x487510,0x482370}
external={0x4d65f0,0x4d69f0,0x4f118c,0x4f103c,0x77409100,0x4c89f0,0x482370,0x4d67f0}
out=ROOT/'驗證紀錄/current/native-water-clock-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True,exist_ok=False)
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
functions={};relocation_rows=[];manifest_rows=[]
for name,(start,limit) in specs.items():
    instructions=list(cs.disasm(get(start,limit-start),start))
    end=max(i.address+i.size for i in instructions if i.mnemonic.startswith('ret'))
    instructions=[i for i in instructions if i.address<end]
    original=get(start,end-start);relocations=[]
    for i in instructions:
        if i.mnemonic in ['call','jmp'] and i.operands[0].type==capstone.x86.X86_OP_IMM:
            target=i.operands[0].imm&0xffffffff
            if not start<=target<end:
                assert i.mnemonic=='call' and target in external,(name,i.mnemonic,hex(target))
                assert i.imm_size==4
                relocations.append(dict(type='relative',offset=i.address-start+i.imm_offset,target=target,next=i.address-start+i.size))
        for operand in i.operands:
            if operand.type==capstone.x86.X86_OP_MEM:
                value=operand.mem.disp&0xffffffff
                if value in absolute:
                    assert i.disp_size==4
                    relocations.append(dict(type='absolute',offset=i.address-start+i.disp_offset,target=value,next=0))
                elif operand.mem.base==0 and operand.mem.index==0 and value>=0x400000:
                    raise ValueError((name,i.mnemonic,i.op_str,'unmapped absolute memory'))
            elif operand.type==capstone.x86.X86_OP_IMM and i.mnemonic not in ['call','jmp'] and not i.mnemonic.startswith('j'):
                value=operand.imm&0xffffffff
                if value in absolute:
                    assert i.imm_size==4
                    relocations.append(dict(type='absolute',offset=i.address-start+i.imm_offset,target=value,next=0))
    assert len({r['offset'] for r in relocations})==len(relocations)
    (out/(name+'-original.bin')).write_bytes(original)
    for r in relocations:relocation_rows.append(f"{name} {r['type']} {r['offset']} {r['target']:x} {r['next']}\n")
    manifest_rows.append(f'{name} {start:x} {len(original)}\n')
    a=next(i for i,s in enumerate(lines) if s.startswith('sub_'+f'{start:X}'+' proc'))
    b=next(i for i in range(a,len(lines)) if lines[i].startswith('sub_'+f'{start:X}'+' endp'))
    functions[name]=dict(start=start,bytes=len(original),sha256=digest(original),relocations=relocations,
       first_newline_line=a+1,last_newline_line=b+1,assembly='\n'.join(lines[a:b+1]).replace('\r',''))
keep=get(0x6365e0,36);keep_values=list(struct.unpack('<9I',keep));assert 0x487510 not in keep_values
(out/'keep-original.bin').write_bytes(keep)
(out/'functions.tsv').write_text(''.join(manifest_rows),encoding='ascii')
(out/'relocations.tsv').write_text(''.join(relocation_rows),encoding='ascii')
(out/'oracle.cpp').write_bytes(CPP.read_bytes());(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
correction={'status':'static_maploader_filter_removes_water_before_reschedule',
 'assembly_sha256':digest(assembly),'PE_sha256':digest(data),'maploader_filter':'4D63A0',
 'preserved_callback_addresses':keep_values,'preserved_table_sha256':digest(keep),
 'finding':'485800/48BC80 call4D63A0 before water50ms scheduling. It removes callbacks outside the original7+2 lists, including487510, then482370 ensures one10000ms callback for itself/ownerFFFFFFFF using4D67F0. Map reload clears old water chains before rearming; repeated S28 without this map-clear can still produce duplicates.',
 'supersedes_partial_direct_call_inference':'water-lifecycle-20261016-071847-339171 had not followed4D63A0; do not infer that mapload always adds another surviving chain.',
 'native_execution':False,'product_source_changes':0,'whole_goal_complete':False}
save(out/'map-clear-correction.json',correction)
record={'status':'prepared_private_clock_fragments_not_built_or_executed','functions':functions,
 'original_PE':str(PE),'original_PE_sha256':digest(data),'original_assembly_sha256':digest(assembly),
 'source_inputs':999,'normal_qualification':str(NORMAL/'qualified.json'),
 'input_files':{p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()},
 'clock_source_stubbed':True,'CRT_memory_operations_stubbed':True,
 'auxiliary_stubs':['4C89F0 audio dispatch tail'],
 'original_map_auxiliary_482370_and_pair_lookup_4D67F0_extracted':True,
 'game_sessions':0,'whole_client_executed':False,'whole_goal_complete':False}
save(out/'prepared.json',record)
with zipfile.ZipFile(out/'prepared-inputs.zip','x',zipfile.ZIP_DEFLATED) as z:
    for p in out.iterdir():
        if p.is_file() and p.name!='prepared-inputs.zip':z.write(p,p.name)
with zipfile.ZipFile(out/'prepared-inputs.zip') as z:
    for n in z.namelist():assert z.read(n)==(out/n).read_bytes()
save(APP/'water-clock-prepared-current.json',{'output':str(out),'status':record['status'],'native_execution':False,'whole_goal_complete':False})
print(json.dumps({'output':str(out),'function_bytes':{n:v['bytes'] for n,v in functions.items()},'relocations':len(relocation_rows),'native_execution':False}))
