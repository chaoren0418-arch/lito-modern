"""Preserve primary type6 entry anchor, draw, frame-flag and lifetime branches."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile

ROOT=Path(__file__).resolve().parents[1]
V=ROOT/'驗證紀錄/current'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211'
inputs=read(normal/'compiled-inputs.json')
for name,digest in inputs.items():assert sha(ROOT/name)==digest
parent=V/'native-eruption-entry-binding/20261018-072356-944599/binding.json'
binding=read(parent)
pe=Path(binding['primary_PE']['path']).read_bytes()
assert hashlib.sha256(pe).hexdigest().upper()==binding['primary_PE']['sha256']
offset=struct.unpack_from('<I',pe,60)[0]
count=struct.unpack_from('<H',pe,offset+6)[0]
sections=offset+24+struct.unpack_from('<H',pe,offset+20)[0]
def raw(address,length):
    for index in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+index*40+8)
        if start<=address-0x400000 and address-0x400000+length<=start+min(size,physical):
            return pe[at+address-0x400000-start:at+address-0x400000-start+length]
    raise ValueError(hex(address))
ranges={'constructor':(0x4235f0,0x4236b7),'advance':(0x423700,0x423890),'timer':(0x423890,0x4238a1),'entry':(0x4238b0,0x4238f0)}
decoder=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
decoded={name:list(decoder.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()}
instructions={i.address:i for rows in decoded.values()for i in rows}
expected={0x423645:('mov','byte ptr [esi + 0x11], bl'),0x42364c:('mov','byte ptr [esi + 0x10], bl'),0x42364f:('mov','byte ptr [esi + 0x13], bl'),0x42369f:('mov','byte ptr [esi + 0x13c], bl'),0x423715:('call','0x4e5b50'),0x423734:('call','0x4cd9d0'),0x42374c:('or','edx, 5'),0x42375a:('call','0x4cd9d0'),0x42376e:('cmp','byte ptr [ecx], 0'),0x42379d:('test','byte ptr [ecx + eax*4 + 0x12], 1'),0x4237ad:('call','0x4e8160'),0x4237bc:('cmp','byte ptr [edi + 0x10], 8'),0x423847:('call','0x4e45f0'),0x423878:('call','0x4d65f0'),0x423884:('mov','byte ptr [esi + 0x13c], 1'),0x423893:('call','0x4e01a0'),0x42389a:('call','0x423700'),0x4238bf:('call','0x4e8160'),0x4238cb:('mov','dword ptr [esi + 0x2c], ecx'),0x4238d1:('mov','dword ptr [esi + 0x30], edx'),0x4238d7:('mov','dword ptr [esi + 0x3c], ecx'),0x4238dd:('mov','dword ptr [esi + 0x40], edx')}
for address,operation in expected.items():assert (instructions[address].mnemonic,instructions[address].op_str)==operation,hex(address)
out=V/'native-eruption-type6-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in decoded[name])+'\n',encoding='utf-8')
record=dict(status='primary_type6_entry_anchor_frame_flag_and_retirement_branches_bound_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],primary_PE=binding['primary_PE'],parent_type7_binding=dict(path=str(parent),sha256=sha(parent)),constructor='0x4235F0',entry='0x4238B0',advance='0x423700',timer='0x423890',
    observations=[
        'Constructor initializes action10, direction11, source13 and terminal13C to zero.',
        'Entry optionally resolves descriptor68 through4E8160 and copies current coordinates into2C/30 and3C/40, then calls4E78B0/423700. Its timer contains only4E01A0 then423700; no direct entry-position lookup.',
        'Advance increments/selects a frame through4E5B50 and submits main84 and optional shadow88 using parent subframe19 and the same2C/30. Shadow submission ORs parent flags with5.',
        'The frame-flag branch requires descriptor68, a nonzero byte at descriptor+8, and bit1 at the selected native frame+12h before resolving the descriptor ID again.',
        'The resolved actor then passes additional action/status/graphics/callee guards before native action2 dispatch. This artifact retains those exact branches without collapsing their semantics.',
        'Terminal13C first causes4E8230 and deletion. Otherwise the selected dwell schedules timer423890 through4D65F0; source13 zero sets terminal13C after scheduling, retaining the terminal dwell.'
    ],native_filtered_graphics_immediates=[0x4dd,0x636,0x7d1,0x15f4],complete_hurt_guard_semantics_qualified=False,transitive_callee_position_behavior_qualified=False,scheduler_external_time_units_qualified=False,
    pending=['Bind action flags, all transitive guard semantics and authority cue identity to current native incarnation/owner checks.','Bind primary main/shadow image and audio fields to exact current TW typed definitions and selected SPR/WAV assets.','Retain source0 dwell and scene/owner retirement; prove fake-sink playback and current-source C02 before integration.'],
    normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record)
(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins={path.name:sha(path)for path in out.iterdir()if path.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in pins:archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as archive:assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for name,digest in inputs.items():assert sha(ROOT/name)==digest
print(json.dumps(dict(binding=str(out/'binding.json'),entry=record['entry'],complete_hurt_guards_qualified=False,new_casts=0),ensure_ascii=False))
