"""Bind fresh native identities for recursive type7 nodes and the type6 child."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';inputs=read(normal/'compiled-inputs.json')
for name,digest in inputs.items():assert sha(ROOT/name)==digest
parent=V/'native-eruption-wire-binding/20261018-081317-270930/binding.json';binding=read(parent)
pe=Path(binding['primary_PE']['path']).read_bytes();assert hashlib.sha256(pe).hexdigest().upper()==binding['primary_PE']['sha256']
offset=struct.unpack_from('<I',pe,60)[0];count=struct.unpack_from('<H',pe,offset+6)[0];sections=offset+24+struct.unpack_from('<H',pe,offset+20)[0]
def raw(address,length):
    for index in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+index*40+8)
        if start<=address-0x400000 and address-0x400000+length<=start+min(size,physical):return pe[at+address-0x400000-start:at+address-0x400000-start+length]
    raise ValueError(hex(address))
decoder=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges={'recursive_constructor_and_branch':(0x423b25,0x423c50),'child_constructor_and_entry':(0x423c6b,0x423cc0),'identity_assignment':(0x4e78b0,0x4e78dd)}
decoded={name:list(decoder.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()};ins={i.address:i for rows in decoded.values()for i in rows}
expected={0x423b6d:('push','0'),0x423b71:('call','0x4238f0'),0x423c1b:('je','0x423c44'),0x423c1d:('cmp','dword ptr [esi + 0x154], 1'),0x423c24:('jl','0x423c44'),0x423c28:('call','0x4e78b0'),0x423c2f:('call','0x423a40'),0x423c4e:('call','dword ptr [edx]'),0x423ca5:('push','0'),0x423ca9:('call','0x4235f0'),0x423cbd:('call','dword ptr [edx + 4]'),0x4e78bc:('mov','eax, dword ptr [esi + 8]'),0x4e78c1:('je','0x4e78cd'),0x4e78c3:('call','0x4e77e0'),0x4e78cd:('mov','eax, dword ptr [0x834f24]'),0x4e78d2:('push','ebx'),0x4e78d3:('inc','eax'),0x4e78d5:('mov','dword ptr [0x834f24], eax'),0x4e78da:('mov','dword ptr [esi + 8], eax')}
for address,operation in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==operation,hex(address)
out=V/'native-eruption-node-identity-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in decoded[name])+'\n',encoding='utf-8')
record=dict(status='primary_type7_recursive_and_child_fresh_object_identity_bound_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],primary_PE=binding['primary_PE'],parent_wire_binding=dict(path=str(parent),sha256=sha(parent)),
    observations=['The first type7 constructor argument is the native object+8 ID; the network-root constructor receives the S23 projectile ID through the separately qualified wire mapping.','Recursive type7 construction explicitly passes0 as this first argument. A continuing node calls4E78B0, whose zero-ID path increments global834F24 and writes a new local object+8 ID.','The endpoint/exhaustion branch deletes the just-constructed type7 node before registration/advancement; no frame1 draw or audio is emitted by that discarded candidate node.','The terminal type6 child constructor also receives0 as the first argument. Its separately bound slot4 entry calls4E78B0, so it acquires a fresh local identity before advancing.','Each retained node therefore has its own animation source counter and native local identity; the endpoint child is distinct from both the root cast and the continuing nodes.'],
    implementation_inference='A modern presentation ledger must preserve node identity within an owner/cast lifetime, rather than collapse every recursive source-frame sound into the wire projectile ID. Qualify this separately with actual sound modes, duplicate-frame rejection and scene/owner retirement; native global IDs do not authorize network actors.',
    original_native_audio_ledger_implementation_inferred=False,modern_node_ledger_implemented=False,sound_mode_pool_runtime_qualified=False,SPR_pixels_loaded=False,normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for name,digest in inputs.items():assert sha(ROOT/name)==digest
print(json.dumps(dict(binding=str(out/'binding.json'),recursive_and_child_fresh_local_IDs_bound=True,modern_node_ledger_implemented=False,new_casts=0),ensure_ascii=False))
