"""Match every classfile method to javap before reading hotel direct callers."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, zipfile

ROOT = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
out = a.run.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-hotel-lifecycle-reference-C02')
jar = ROOT / 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

expected = 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(jar) == expected
with zipfile.ZipFile(jar) as z:
    data = z.read('local/lito381/server/ClientSession.class')
assert data[:4] == b'\xca\xfe\xba\xbe'
at = 8
def take(n):
    global at
    assert n >= 0 and at + n <= len(data)
    result = data[at:at + n]
    at += n
    return result
def u2():
    return int.from_bytes(take(2), 'big')
pool = [None] * u2()
i = 1
while i < len(pool):
    tag = take(1)[0]
    if tag == 1:
        pool[i] = take(u2())
    elif tag in [3, 4, 9, 10, 11, 12, 17, 18]:
        take(4)
    elif tag in [5, 6]:
        take(8)
        i += 1
    elif tag in [7, 8, 16, 19, 20]:
        take(2)
    elif tag == 15:
        take(3)
    else:
        raise ValueError(tag)
    i += 1
take(6)
take(2 * u2())
def attributes():
    for _ in range(u2()):
        take(2)
        take(int.from_bytes(take(4), 'big'))
for _ in range(u2()):
    take(6)
    attributes()
methods = []
for _ in range(u2()):
    flags = u2()
    name, descriptor = [pool[u2()].decode('utf-8') for _ in range(2)]
    attributes()
    methods.append((name, descriptor))
attributes()
assert at == len(data) and len(methods) == len(set(methods)) == 2153
text = (out / 'ClientSession.javap.txt').read_text(encoding='utf-8-sig')
headers = list(re.finditer(r'^  (?=\S).+$', text, re.M))
blocks = {}
for i, m in enumerate(headers):
    block = text[m.start():headers[i + 1].start() if i + 1 < len(headers) else len(text)]
    descriptor = re.search(r'^\s+descriptor: (\(.+)$', block, re.M)
    if descriptor:
        signature = m.group().strip()
        name = '<clinit>' if signature == 'static {};' else signature.split('(')[0].split()[-1]
        if name == 'local.lito381.server.ClientSession':
            name = '<init>'
        key = (name, descriptor[1].strip())
        assert key not in blocks
        blocks[key] = (signature, block)
assert set(blocks) == set(methods)
calls = []
for key, (signature, block) in blocks.items():
    for line in block.splitlines():
        if re.search(r'// Method (?:local/lito381/server/ClientSession\.)?beginRentalTownMapTransition:', line):
            calls.append(dict(caller=key[0], descriptor=key[1], instruction=line.strip()))
assert len(calls) == 2 and {x['caller'] for x in calls} == {'handleRentalTownHotelAction', 'serverRentalHotelLeaseInvalidated'}
portal = blocks[('enterRentalTownPortal', '(Llocal/lito381/server/Protocol$EnterPortalRequest;)V')][1]
assert re.search(r'138: invokevirtual\s+#\d+\s+// Method local/lito381/server/NativeLitoTownPortalCatalog\$Route.targetMapId:\(\)I\s+141: aload_0\s+142: getfield\s+#\d+\s+// Field rentalTownMapId:I\s+145: if_icmpne\s+175', portal)
(out / 'ClientSession.class').write_bytes(data)
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files = {p.name: p for p in out.iterdir() if p.is_file() and p.suffix.lower() != '.zip'}
hashes = {name: sha(p) for name, p in files.items()}
archive = out / 'raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(hashes) and len(z.namelist()) == len(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
report = dict(status='independently_readback_verified_C02_all2153_method_direct_hotel_transition_callers_only',
    jar_sha256=expected, classfile_sha256=sha(out / 'ClientSession.class'), classfile_methods=2153,
    javap_name_descriptor_pairs_exact=True, direct_transition_callers=calls, C73_rental_destination_same_map_required=True,
    first1990_method_parser_incomplete_preserved=True, actual_game_sessions=0, manual_guest_exit_qualified=False,
    inference='No additional direct guest exit contract was found in this class; this does not prove global runtime impossibility.',
    raw_archive_sha256=sha(archive), raw_files=hashes, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'independent-readback.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: report[k] for k in ['status', 'classfile_methods', 'actual_game_sessions', 'manual_guest_exit_qualified']}, ensure_ascii=False))
