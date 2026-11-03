"""Export only original numeric inputs/call records from qualified geometry."""
from pathlib import Path
import csv, hashlib, json, zipfile

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'驗證紀錄/current/native-background-geometry-original/20261016-110731-202085'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
q=read(SOURCE/'qualified.json')
assert q['status']=='independently_verified_original_background_geometry_and_rectangle_intersections'
assert q['actual_cases']==2688 and q['actual_recorded_pixel_calls']==17472
assert digest((SOURCE/'run-raw.zip').read_bytes())==q['raw_archive_sha256']
with zipfile.ZipFile(SOURCE/'run-raw.zip') as z:
    for n in ['cases.csv','calls-native.csv','counts-native.csv']:
        assert z.read(n)==(SOURCE/n).read_bytes(), n
def rows(name):
    with (SOURCE/name).open(encoding='ascii',newline='') as f:
        r=csv.reader(f);columns=next(r);return columns,[[int(v) for v in row] for row in r]
ci,cases=rows('cases.csv');co,calls=rows('calls-native.csv');_,counts=rows('counts-native.csv')
assert len(cases)==len(counts)==2688 and len(calls)==17472
grouped=[[] for _ in cases]
for c in calls:
    assert c[1]==len(grouped[c[0]])
    grouped[c[0]].append(c[2:])
for i,(params,count) in enumerate(zip(cases,counts)):
    assert params[0]==count[0]==i and count[1]==len(grouped[i])
fixture={'format':'LITO_BACKGROUND_GEOMETRY_ORIGINAL1',
    'qualification_sha256':digest((SOURCE/'qualified.json').read_bytes()),
    'original_raw_archive_sha256':q['raw_archive_sha256'],
    'input_columns':ci[1:],'call_columns':co[2:],
    'cell_x_is_native_half_cell':True,'modern_viewport_adapter_verified':False,
    'cases':[{'input':p[1:],'calls':c} for p,c in zip(cases,grouped)]}
dest=ROOT/'LiTo381-Modern/tests/fixtures/native-background-geometry-original.json'
assert not dest.exists()
dest.write_text(json.dumps(fixture,separators=(',',':'))+'\n',encoding='utf-8')
assert read(dest)==fixture
print(json.dumps({'path':str(dest),'bytes':dest.stat().st_size,'sha256':digest(dest.read_bytes()),'cases':2688,'calls':17472}))
