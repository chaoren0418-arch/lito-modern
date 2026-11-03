"""Bind type7 folded image and paired shadow sampling from exact primary bytes."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
from capstone.x86_const import X86_OP_MEM,X86_REG_ESI,X86_REG_ECX
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';inputs=read(normal/'compiled-inputs.json')
for name,digest in inputs.items():assert sha(ROOT/name)==digest
parent=V/'native-eruption-entry-binding/20261018-072356-944599/binding.json';binding=read(parent)
pe=Path(binding['primary_PE']['path']).read_bytes();assert hashlib.sha256(pe).hexdigest().upper()==binding['primary_PE']['sha256']
offset=struct.unpack_from('<I',pe,60)[0];count=struct.unpack_from('<H',pe,offset+6)[0];sections=offset+24+struct.unpack_from('<H',pe,offset+20)[0]
def raw(address,length):
    for index in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+index*40+8)
        if start<=address-0x400000 and address-0x400000+length<=start+min(size,physical):return pe[at+address-0x400000-start:at+address-0x400000-start+length]
    raise ValueError(hex(address))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);md.detail=True
ranges={'entry':(0x423cf0,0x423d9d),'advance':(0x423a40,0x423cd0),'timer':(0x423cd0,0x423ce1),'frame_increment':(0x4e5b50,0x4e5bf3),'frame_select':(0x4e5c00,0x4e63b0),'shadow101_parser':(0x43e444,0x43e463),'direction_zero':(0x4b9210,0x4b9237)}
decoded={name:list(md.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()};ins={i.address:i for rows in decoded.values()for i in rows}
expected={0x423d33:('call','0x4b9210'),0x423d38:('cmp','al, 3'),0x423d3a:('mov','byte ptr [esi + 0x11], al'),0x423d3f:('sub','al, 4'),0x423d41:('mov','byte ptr [esi + 0x11], al'),0x423a6d:('call','0x4e5b50'),0x423a8c:('call','0x4cd9d0'),0x423aa4:('or','edx, 5'),0x423ab2:('call','0x4cd9d0'),0x4e5b58:('inc','bl'),0x4e5bc9:('mov','byte ptr [ecx + 0x13], dl'),0x4e5bcc:('call','0x4e5c00'),0x4e5be1:('call','0x4e5c00'),0x4e5d50:('cmp','byte ptr [eax], 0'),0x4e5d55:('movsx','eax, byte ptr [esi + 0x11]'),0x4e5d59:('add','edi, eax'),0x4e5ff4:('mov','edx, edi'),0x4e5ff6:('mov','ecx, ebx'),0x4e5ff8:('call','0x4406c0'),0x4e6014:('mov','dword ptr [esi + 0x88], edx'),0x4e604d:('mov','dl, byte ptr [ecx + edx*4 + 0x11]'),0x4e6056:('mov','byte ptr [esi + 0x19], dl'),0x43e444:('cmp','edi, 0x65'),0x43e451:('mov','edx, dword ptr [0x647e70]'),0x43e45a:('mov','word ptr [edx + ecx*2], ax'),0x4b9230:('mov','eax, dword ptr [ebp + 0xc]')}
for address,operation in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==operation,hex(address)
for name in ['advance','timer','frame_increment','frame_select']:
    for instruction in decoded[name]:
        for operand in instruction.operands:
            assert not(operand.type==X86_OP_MEM and operand.mem.base in [X86_REG_ESI,X86_REG_ECX]and operand.mem.disp==0x11 and operand.access&capstone.CS_AC_WRITE),(name,hex(instruction.address))
tw=V/'native-eruption-current-tw-binding/20261018-072559-347946';tj=read(tw/'binding.json')
assert tj['decoded_sha256']=='7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008'
for row in tj['definitions']:assert sha(tw/(str(row['graphics_id'])+'.definition.txt'))==row['sha256']
body=(tw/'129.definition.txt').read_bytes();assert b'#129  4' in body and b'0.attack(1 10,'in body and b'101.shadow(130)'in body
assert b'<'not in body and b'>'not in body
child=(tw/'123.definition.txt').read_bytes();assert b'#123  1'in child and b'0.attack(0 8,'in child and b'101.shadow(124)'in child
out=V/'native-eruption-image-consumer-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in decoded[name])+'\n',encoding='utf-8')
record=dict(status='primary_type7_four_image_consumer_and_paired_shadow_sampling_bound_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],primary_PE=binding['primary_PE'],parent_entry_binding=dict(path=str(parent),sha256=sha(parent)),current_TW_raw_binding=dict(path=str(tw/'binding.json'),sha256=sha(tw/'binding.json')),legal_direction_0_through_7_to_parent_image=[0,1,2,3,0,1,2,3],rule='Constructor and entry each use one signed greater-than3 subtraction4, not unrestricted modulo. Entry calls4B9210 first; coincident points return its caller fallback. Generic frame selector adds folded byte11 to the directional source image0.',
    findings=['Current raw129 action0 is directional with four declared images; legal native directions therefore select images0..3. Recursive constructor inherits the folded byte without re-entering the direction routine.','4E5C00 uses the exact selected image for the secondary88 bank. Parser101 stores the shadow ID in647E70, and the draw routine samples parent subframe19 at the same2C/30 with flagsOR5. No standalone shadow action0 is required.','Raw123 action0 is nondirectional and has one declared image with shadow124; type6 constructor initializes direction11 to0.','4E5B50 increments the source before selection and wraps to source0. Type7 and type6 independently retain source0 terminal dwell before retiring; this artifact does not qualify host time units.','No direct write of owner byte11 exists in the retained advance, timer or common frame functions; arbitrary transitive behavior and malformed wire direction domains are not generalized.'],
    main_direction_image_mapping_bound=True,paired_shadow_parent_frame_sampling_bound=True,current_TW_Rust_typed_parser_executed=False,SPR_pixels_loaded=False,external_scheduler_units_qualified=False,complete_renderer_qualified=False,normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for name,digest in inputs.items():assert sha(ROOT/name)==digest
print(json.dumps(dict(binding=str(out/'binding.json'),legal_direction_to_image=record['legal_direction_0_through_7_to_parent_image'],paired_shadow_sampling_bound=True,renderer_qualified=False),ensure_ascii=False))
