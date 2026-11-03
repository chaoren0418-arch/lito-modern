"""Read primary type7 entry and propagation branches without renderer inference."""
from pathlib import Path
from datetime import datetime, timezone
import capstone, hashlib, json, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
V = ROOT/'驗證紀錄/current'
def read(path):
    return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()
def save(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')

normal = V/'native-retired-join-normal/validation-20261018-063154-317211'
inputs = read(normal/'compiled-inputs.json')
for name, digest in inputs.items():
    assert sha(ROOT/name) == digest
parent = V/'native-eruption-constructor-binding/20261018-065848-311632/binding.json'
binding = read(parent)
primary = Path(binding['primary_PE']['path'])
pe = primary.read_bytes()
assert hashlib.sha256(pe).hexdigest().upper() == binding['primary_PE']['sha256']
offset = struct.unpack_from('<I', pe, 60)[0]
count = struct.unpack_from('<H', pe, offset+6)[0]
sections = offset+24+struct.unpack_from('<H', pe, offset+20)[0]
def raw(address, length):
    for index in range(count):
        size, start, physical, at = struct.unpack_from('<IIII', pe, sections+index*40+8)
        if start <= address-0x400000 and address-0x400000+length <= start+min(size, physical):
            return pe[at+address-0x400000-start:at+address-0x400000-start+length]
    raise ValueError(hex(address))

decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
ranges = {'entry':(0x423cf0,0x423d9d), 'advance':(0x423a40,0x423cd0), 'timer':(0x423cd0,0x423ce1), 'deleting-slot0':(0x423a00,0x423a21)}
decoded = {name:list(decoder.disasm(raw(lo, hi-lo), lo)) for name,(lo,hi) in ranges.items()}
ops = {name:[(i.mnemonic,i.op_str) for i in instructions] for name,instructions in decoded.items()}
entry = ops['entry']
for operation in [('call','0x4e8160'), ('mov','dword ptr [esi + 0x14c], ecx'), ('mov','dword ptr [esi + 0x150], edx'), ('movsx','eax, byte ptr [esi + 0x11]'), ('call','0x4b9210'), ('mov','byte ptr [esi + 0x11], al'), ('sub','al, 4'), ('call','0x4b92d0'), ('call','0x423a40')]:
    assert operation in entry, operation
assert entry.index(('call','0x4b9210')) < entry.index(('mov','byte ptr [esi + 0x11], al'))
assert [('call',i.op_str) for i in decoded['timer'] if i.mnemonic == 'call'] == [('call','0x4e01a0'),('call','0x423a40')]
advance = ops['advance']
for operation in [('cmp','al, 2'), ('call','0x4238f0'), ('call','0x4235f0'), ('call','0x4d65f0'), ('mov','edx, 0x423cd0'), ('and','ecx, 0xfffffffe'), ('dec','edx'), ('cmp','dword ptr [esi + 0x154], 1')]:
    assert operation in advance, operation
assert ('call','0x4f103c') in ops['deleting-slot0']
out = V/'native-eruption-entry-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n', encoding='utf-8')
record = dict(status='primary_type7_entry_direction_overwrite_and_propagation_branches_bound_only', source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'], primary_PE=binding['primary_PE'], parent_constructor_binding=dict(path=str(parent),sha256=sha(parent)),
    entry='0x423CF0', timer='0x423CD0', advance='0x423A40', factory_fallback_virtual_slot4_is_entry=True,
    observations=[
        'Entry optionally resolves the descriptor at object+68 through 4E8160 and copies returned+2C/+30 to destination+14C/+150.',
        'Entry reads constructor byte11, calls coordinate helper4B9210 with source+2C and destination+14C, then overwrites byte11 with AL; signed AL greater than3 subtracts4 once.',
        'Entry obtains a vector through4B92D0, stores+144/+148 and a max absolute coordinate difference at+154, then calls4E78B0 and423A40.',
        'Timer423CD0 calls4E01A0 then423A40. It contains no direct destination lookup or direction write; transitive callee behavior is not qualified here.',
        'Advance source-byte13 value2 branch allocates and constructs another type7 object, copies fixed coordinates/vector/destination-distance fields, adds a fixed step, rounds with+8000 then logical shift16 and aligns native X even.',
        'That branch compares next coordinates against destination and old distance against1; continuation calls4E78B0/423A40 on the new object. The terminal branch deletes the new type7 object and constructs type6 via4235F0, then invokes its slot4.',
        'Slot0 wrapper423A00 calls423A30 and conditionally deallocates through4F103C; slot4 is distinct.'
    ],
    constructor_fold_alone_is_final_direction=False, direction_helper_semantics_newly_qualified=False, draw_consumer_qualified=False, current_TW_namespace_qualified=False,
    pending=['Trace base4E01A0 and the draw consumer before complete direction/frame claims.','Bind current TW129/123/130/124 definitions and exact type6 child timeline, target association, shadows and Effect3080.','Bind wire fields and scheduling/time units before implementation; no generic finite-class1 substitution.'],
    normal_product_source_modified=False, corresponding_frozen_QA_modified=False, new_sessions=0, new_casts=0, new_renders=0, request_registered=False, renderer_qualified=False, GUI_verified=False, whole_goal_complete=False)
save(out/'binding.json', record)
(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins = {path.name:sha(path) for path in out.iterdir() if path.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as archive:
    for name in pins:
        archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for name,digest in inputs.items():
    assert sha(ROOT/name) == digest
print(json.dumps(dict(binding=str(out/'binding.json'),entry=record['entry'],constructor_fold_alone_is_final_direction=False,request_registered=False,new_casts=0),ensure_ascii=False))
