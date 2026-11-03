"""Bind primary S23 fields to type7 constructor arguments; no implementation."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
V=ROOT/'驗證紀錄/current'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211'
inputs=read(normal/'compiled-inputs.json')
for name,digest in inputs.items():assert sha(ROOT/name)==digest
parent=V/'native-eruption-factory-binding/20261018-064959-138715/binding.json';binding=read(parent)
pe=Path(binding['primary_PE']['path']).read_bytes();assert hashlib.sha256(pe).hexdigest().upper()==binding['primary_PE']['sha256']
offset=struct.unpack_from('<I',pe,60)[0];count=struct.unpack_from('<H',pe,offset+6)[0];sections=offset+24+struct.unpack_from('<H',pe,offset+20)[0]
def raw(address,length):
    for index in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+index*40+8)
        if start<=address-0x400000 and address-0x400000+length<=start+min(size,physical):return pe[at+address-0x400000-start:at+address-0x400000-start+length]
    raise ValueError(hex(address))
decoder=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges={'header':(0x4a5ed0,0x4a5f4d),'tail':(0x4a5f4d,0x4a5fe0),'type7':(0x4a60ff,0x4a6166),'constructor':(0x4238f0,0x4239fc)}
decoded={name:list(decoder.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()};instructions={i.address:i for rows in decoded.values()for i in rows}
assert raw(0x634390,7)==b'cddccd\0'and raw(0x634384,9)==b'hchhhhcs\0'
expected={0x4a5f0e:('push','0x634390'),0x4a5f1b:('call','0x4a2a50'),0x4a5f78:('push','0x634384'),0x4a5f7e:('call','0x4a2a50'),0x4a5f89:('and','eax, 0xffff'),0x4a5f94:('and','edx, 0xffff'),0x4a5f9a:('lea','ecx, [eax + eax - 0x8000]'),0x4a5fa4:('lea','ebx, [edx + edx - 0x8000]'),0x4a6131:('push','ecx'),0x4a6147:('push','esi'),0x4a6148:('push','ebx'),0x4a6149:('push','edi'),0x4a614a:('push','ecx'),0x4a615c:('call','0x4238f0')}
for address,operation in expected.items():assert (instructions[address].mnemonic,instructions[address].op_str)==operation,hex(address)
branch=[(i.mnemonic,i.op_str)for i in decoded['type7']]
sequence=[('mov','edx, dword ptr [ebp - 0x20]'),('movsx','ecx, word ptr [ecx + edx*2]'),('push','ecx'),('mov','ecx, dword ptr [ebp - 0x1c]'),('push','ecx'),('mov','ecx, dword ptr [ebp - 0x14]'),('push','ecx'),('mov','ecx, dword ptr [ebp - 0x28]'),('and','ecx, 0xff'),('push','ecx'),('mov','ecx, dword ptr [ebp - 0x2c]'),('push','esi'),('push','ebx'),('push','edi'),('push','ecx'),('mov','ecx, dword ptr [ebp - 0x24]'),('and','ecx, 0xff'),('push','ecx'),('push','edx'),('mov','edx, dword ptr [ebp - 0x18]'),('mov','ecx, eax'),('push','edx'),('call','0x4238f0')]
cursor=0
for operation in sequence:
    cursor=branch.index(operation,cursor)+1
modern=ROOT/'LiTo381-Modern/src/modern_lito/native_combat.rs';text=modern.read_text('utf-8-sig')
for fragment in ['action: b[1]','b[2..6]','b[6..10]','damage_marker: b[10]','heading: b[11]','b[12..16]','let graphics = h(16)','flight_type: b[18]','from: [h(19), h(21)]','to: [h(23), h(25)]']:
    assert fragment in text,fragment
arguments=[('projectile_id',8,12,4),('graphics',12,16,2),('flight_type',16,18,1),('source_native_x',20,19,2),('source_native_y',24,21,2),('destination_native_x',28,23,2),('destination_native_y',32,25,2),('heading',36,11,1),('target_id',40,6,4),('damage_marker',44,10,1),('child_graphics',48,None,None)]
out=V/'native-eruption-wire-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in decoded[name])+'\n',encoding='utf-8')
record=dict(status='primary_type7_S23_constructor_field_binding_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],primary_PE=binding['primary_PE'],parent_factory_binding=dict(path=str(parent),sha256=sha(parent)),current_decoder=dict(path=str(modern),sha256=sha(modern)),header_format='cddccd',tail_format='hchhhhcs',constructor='0x4238F0',constructor_arguments=[dict(name=name,ebp_offset=offset,wire_offset=wire,wire_bytes=size)for name,offset,wire,size in arguments],native_coordinate_conversion='NativeX=2*u16WireX-32768 in signed32; NativeY=u16WireY. Fixed16 overflow/step behavior is a separate proof.',child_graphics_source='signed word table647E50 indexed by u16 graphics; not another wire field',heading_constructor_binding_qualified=True,entry_recomputes_and_overwrites_heading=True,final_image_direction_consumer_qualified=False,target_lifecycle_runtime_qualified=False,current_TW_typed_actions_qualified=False,pending=['Execute exact current Rust selected catalogue parser and selected SPR/WAV loading after reserved verification work is terminal.','Prove complete type7 frame/direction consumer, recursive segment lifetimes and type6 child/hurt guards before implementation.','Preserve current target incarnation/owner/epoch retirement and exact scheduling rather than generic single-particle substitution.'],normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={path.name:sha(path)for path in out.iterdir()if path.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for name,digest in inputs.items():assert sha(ROOT/name)==digest
print(json.dumps(dict(binding=str(out/'binding.json'),constructor_arguments=len(arguments),heading_wire_offset=11,renderer_qualified=False,new_casts=0),ensure_ascii=False))
