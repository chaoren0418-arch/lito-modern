"""Bind selected current TW definitions by exact pack/text hashes; no playback."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, struct, zipfile, zlib
from Crypto.Cipher import AES

ROOT = Path(__file__).resolve().parents[1]
V = ROOT/'驗證紀錄/current'
def read(path):
    return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path, value):
    Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')

normal = V/'native-retired-join-normal/validation-20261018-063154-317211'
inputs = read(normal/'compiled-inputs.json')
assert len(inputs) == 1041
for name,digest in inputs.items():
    assert sha(ROOT/name) == digest
decoder_path = ROOT/'LiTo381-Modern/src/modern_lito/tw_action_catalog.rs'
mac_path = ROOT/'LiTo381-Modern/src/shared_v1/morph_auth.rs'
decoder = decoder_path.read_text('utf-8-sig')
mac_source = mac_path.read_text('utf-8-sig')
def table(source,name):
    match = re.search(r'const '+name+r':\s*\[u8;\s*\d+\]\s*=\s*\[(.*?)\];',source,re.S)
    assert match
    return bytes(int(value,16) for value in re.findall(r'0x([0-9A-Fa-f]{2})',match[1]))
xor = table(decoder,'XOR_TABLE')
auth_key = table(mac_source,'MORPH_AUTH_KEY')
assert len(xor) == 256 and len(auth_key) == 16
pack = Path(r'D:/lineage開發/0-客戶端/launcher/TW13081901.pak')
before = (pack.stat().st_size,pack.stat().st_mtime_ns)
assert 36 <= before[0] <= 8*1024*1024
encoded = pack.read_bytes()
pack_hash = hashlib.sha256(encoded).hexdigest().upper()
assert pack_hash == 'B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218'
expected = struct.unpack_from('<I',encoded)[0]
assert expected == 5_974_929
bound = encoded[:-16]
prefix = struct.pack('<Q',len(bound))+b'\x80'+bytes(7)
pad = (b'\x80'+bytes(15-len(bound)%16)) if len(bound)%16 else b'\x80'+bytes(15)
tag = AES.new(auth_key,AES.MODE_CBC,iv=bytes(16)).encrypt(prefix+bound+pad)[-16:]
assert tag == encoded[-16:]
key = encoded[4:20]
payload = bytearray(encoded[20:-16])
complete = len(payload)//16*16
payload[:complete] = AES.new(key,AES.MODE_ECB).decrypt(payload[:complete])
for index in range(len(payload)):
    payload[index] ^= xor[index%256]^key[index%16]
if payload[:1] == b'S':
    output = bytes(payload)
else:
    inflater = zlib.decompressobj()
    output = inflater.decompress(payload,expected+1)
    assert inflater.eof and not inflater.unused_data and not inflater.unconsumed_tail
assert len(output) == expected and output[:1] == b'S' and b'\0' not in output
text = output[1:]
text_hash = hashlib.sha256(text).hexdigest().upper()
assert text_hash == '7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008'
headers = list(re.finditer(rb'(?m)^[\t ]*#([0-9]+)(?=[\s=])[^\r\n]*',text))
selected = {123,124,129,130,1811}
definitions = {}
for index,header in enumerate(headers):
    graphics = int(header[1])
    if graphics not in selected:
        continue
    assert graphics not in definitions, graphics
    end = headers[index+1].start() if index+1 < len(headers) else len(text)
    data = text[header.start():end]
    assert len(data) <= 256*1024
    definitions[graphics] = dict(graphics_id=graphics,byte_offset=header.start(),byte_length=len(data),line=text[:header.start()].count(b'\n')+1,sha256=hashlib.sha256(data).hexdigest().upper(),raw=data)
assert set(definitions) == selected
out = V/'native-eruption-current-tw-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
for graphics,row in definitions.items():
    (out/(str(graphics)+'.definition.txt')).write_bytes(row.pop('raw'))
record = dict(status='selected_current_TW_raw_definitions_content_bound_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],pack=dict(path=str(pack),bytes=len(encoded),sha256=pack_hash,content_binding_verified=True,binding_is_not_a_trusted_signature=True),decoded_bytes=len(text),decoded_sha256=text_hash,decoder_sources={str(decoder_path):sha(decoder_path),str(mac_path):sha(mac_path)},definitions=list(definitions.values()),base_namespace_merged=False,strict_Rust_catalog_parser_executed=False,SPR_members_read=0,current_TW_raw_spans_bound=True,current_TW_typed_actions_qualified=False,draw_consumer_qualified=False,normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record)
(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins = {path.name:sha(path) for path in out.iterdir() if path.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as archive:
    for name in pins:
        archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
assert before == (pack.stat().st_size,pack.stat().st_mtime_ns) and sha(pack) == pack_hash
for name,digest in inputs.items():
    assert sha(ROOT/name) == digest
print(json.dumps(dict(binding=str(out/'binding.json'),selected_ids=sorted(selected),selected_bytes=sum(row['byte_length'] for row in definitions.values()),new_casts=0,typed_actions_qualified=False),ensure_ascii=False))
