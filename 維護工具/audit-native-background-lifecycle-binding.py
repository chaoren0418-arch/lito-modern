"""Preserve actual packet/result routing and distinct ready-reuse queue policy."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,zipfile
import capstone
ROOT=Path(__file__).resolve().parents[1]
PE=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
ASM=ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm'
def sha(p):
    with p.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def digest(b):return hashlib.sha256(b).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
assert sha(PE)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
data=PE.read_bytes();h=struct.unpack_from('<I',data,60)[0];optional=h+24;base=struct.unpack_from('<I',data,optional+28)[0]
sections=[]
for i in range(struct.unpack_from('<H',data,h+6)[0]):
    at=optional+struct.unpack_from('<H',data,h+20)[0]+40*i;sections.append(struct.unpack_from('<III',data,at+12))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
def jump_table(va,size):
    instructions=list(cs.disasm(get(va,size),va))
    jumps=[i for i in instructions if i.mnemonic=='jmp' and i.operands[0].type==capstone.x86.X86_OP_MEM]
    assert len(jumps)==1 and jumps[0].operands[0].mem.scale==4
    return jumps[0].operands[0].mem.disp
secondary_table=jump_table(0x4b313f,30);result_table=jump_table(0x4904dc,20)
def target(table,index):return struct.unpack('<I',get(table+4*index,4))[0]
primary={op:target(0x4b3208,get(0x4b3470+op,1)[0]) for op in [0xd1,0xd2]}
secondary={op:target(secondary_table,get(0x4b36b4+op,1)[0]) for op in [3,7]}
assert primary=={0xd1:0x4b2f37,0xd2:0x4b2f43}
assert secondary=={3:0x4b3175,7:0x4b3169} and target(result_table,3)==0x4910f2
asm=ASM.read_bytes().decode('utf-8-sig').split('\n')
spans={'result_parser':(304930,305033),'result3_initialize':(306571,306581),
    'ready_parser_self_and_branch':(353741,353813),'ready_reuse_body':(269268,269375),
    'ready_reuse_filter':(448620,448693),'primary_packet_dispatch':(372380,372418),
    'primary_ready_and_battle':(373821,373844),'secondary_dispatch':(374425,374470),
    'initial_background_and_green':(269148,269163),'battle_background_rearm':(354153,354162)}
texts={name:'\n'.join(asm[a-1:b]).replace('\r','') for name,(a,b) in spans.items()}
assert 'movsx   eax, [ebp+var_1]' in texts['result_parser']
assert 'push    offset aCd ' in texts['result_parser']
assert 'cmp     [ebp+var_1], 1' in texts['ready_parser_self_and_branch']
assert 'push    offset dword_64ABF8' in texts['ready_parser_self_and_branch']
assert 'call    sub_4D3A80' in texts['ready_parser_self_and_branch']
assert 'call    sub_47E360' in texts['ready_parser_self_and_branch']
assert 'call    sub_47CF00' in texts['ready_parser_self_and_branch']
assert 'call    sub_4D64F0' in texts['ready_reuse_body'] and 'mov     byte_64BC2D, bl' in texts['ready_reuse_body']
assert 'cmp     eax, offset sub_4159B0' in texts['ready_reuse_filter']
assert 'cmp     eax, offset sub_4C32A0' in texts['ready_reuse_filter']
assert 'sub_4878C0' not in texts['ready_reuse_body']
source_names=['native_game.rs','native_room.rs','native_login.rs','native_environment.rs','native_water_clock.rs']
source_dir=ROOT/'LiTo381-Modern/src/modern_lito'
source_pins={name:sha(source_dir/name) for name in source_names}
assert '3 | 0xcb => match native_source_identity::decode(body)?' in (source_dir/'native_game.rs').read_text(encoding='utf-8-sig')
assert '0xd1 => {' in (source_dir/'native_room.rs').read_text(encoding='utf-8-sig')
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-background-lifecycle-binding'/stamp;out.mkdir(parents=True)
(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
for name in source_names:(out/('current-'+name)).write_bytes((source_dir/name).read_bytes())
save(out/'assembly-spans.json',{name:dict(first_LF_line=spans[name][0],last_LF_line=spans[name][1],text=text) for name,text in texts.items()})
ranges={'primary-routing-table':(0x4b3208,0x4b3470-0x4b3208),'primary-byte-table':(0x4b3470,255),
    'secondary-routing-table':(secondary_table,0x4b36b4-secondary_table),'secondary-byte-table':(0x4b36b4,253),
    'result-routing-table':(result_table,119*4),'ready-filter':(0x4d64f0,119)}
for name,(start,size) in ranges.items():assert size>0 and size<2048;(out/(name+'.bin')).write_bytes(get(start,size))
result=dict(status='readonly_original_background_lifecycle_routing_and_ready_filter_distinguished',
    original_PE_sha256=sha(PE),original_ASM_sha256=sha(ASM),primary_opcode_targets=primary,
    secondary_opcode_targets=secondary,secondary_jump_table_address=secondary_table,result_jump_table_address=result_table,
    accepted_result3_target=target(result_table,3),result3_is_not_directory_opcode3=True,
    primary_D1_is_named_participant_readiness=True,ready_full_or_reuse_requires_self_name_and_state1=True,
    ready_full_or_reuse_selector_globals=[0x648eb8,0x834c18],ready_mode_domain=[2,3,6],
    full_initialization=0x47cf00,reuse_initialization=0x47e360,reuse_filter=0x4d64f0,
    reuse_filter_preserves_callbacks=[0x4159b0,0x4c32a0],reuse_filter_removes_background_green_water_and_aux=True,
    reuse_path_has_no_direct_background_start=True,indirect_starts_not_ruled_out=True,
    D2_selected_rearm=0x4a98a0,generic_map_and_scene_filters_are_distinct=[0x4d63a0,0x4d6470],
    current_source_pins=source_pins,current_product_dispatch_slot6_is_unbound=True,
    remaining='Qualify the distinct ready-reuse filter, then bind accepted self-ready/classic-entry/battle events with GAME ownership. Do not start on directory count, peer readiness, or terrain-window reload. Source selector globals and whole caller paths still require qualification.',
    native_execution=False,whole_packet_handlers_executed=False,modern_controller_binding_verified=False,
    product_source_changed=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'observation.json',result)
files={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(files)
    for name,value in files.items():assert digest(z.read(name))==value
save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=files))
for name,value in source_pins.items():assert sha(source_dir/name)==value
print(json.dumps(dict(output=str(out),result3_is_not_directory_opcode3=True,ready_filter_preserves_only_two=True,product_source_changed=False),ensure_ascii=False))
