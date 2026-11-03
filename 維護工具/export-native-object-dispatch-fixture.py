"""Export numeric route/pulse-or-weight/gate bytes, excluding native pointers."""
from pathlib import Path
import csv,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-background-dispatch-original/20261016-124150-153377'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
q=read(OUT/'qualified.json');assert q['actual_cases']==263423 and q['all_archives_readback_verified']
assert digest((OUT/'run-raw.zip').read_bytes())==q['raw_archive_sha256']
with zipfile.ZipFile(OUT/'run-raw.zip') as z:
    for n in ['records-native.bin','cases.csv']:assert z.read(n)==(OUT/n).read_bytes()
raw=(OUT/'records-native.bin').read_bytes();assert len(raw)==263423*16
packed=bytearray()
for route,a,b,c in struct.iter_unpack('<IIII',raw):
    value=b if route==5 else a if route==8 else 0
    gate=a&255 if route==7 else 0
    assert route<9 and value<=31 and gate in [0,1]
    packed.extend([route,value,gate])
with (OUT/'cases.csv').open(encoding='ascii',newline='') as f:
    rows=list(csv.reader(f));columns=rows.pop(0)
assert len(rows)==263423
fixture={'format':'LITO_OBJECT_DISPATCH_ORIGINAL1','qualification_sha256':digest((OUT/'qualified.json').read_bytes()),
    'original_raw_archive_sha256':q['raw_archive_sha256'],'expected_outcome_sha256':digest(packed),
    'outcome_columns':['route','pulse_or_roof_weight','map303_gate_low_byte'],
    'exhaustive_cases':262144,'total_cases':263423,'targeted_input_columns':columns[1:],
    'targeted_inputs':[[int(v) for v in row[1:]] for row in rows[262144:]],
    'no_original_instructions_or_native_pointer_operands':True}
folder=ROOT/'LiTo381-Modern/tests/fixtures';jp=folder/'native-object-dispatch-original.json';bp=folder/'native-object-dispatch-outcomes.bin'
assert not jp.exists() and not bp.exists()
bp.write_bytes(packed);jp.write_text(json.dumps(fixture,separators=(',',':'))+'\n',encoding='utf-8')
print(json.dumps({'JSON_bytes':jp.stat().st_size,'JSON_sha256':digest(jp.read_bytes()),'outcome_bytes':len(packed),'outcome_sha256':digest(packed)}))
