"""Export real-IMG frame hashes only from the qualified original composition."""
from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-background-real-original/20261016-123005-086741'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
q=read(OUT/'qualified.json');assert q['actual_frames']==24 and q['all_archives_readback_verified']
assert digest((OUT/'run-raw.zip').read_bytes())==q['raw_archive_sha256']
with zipfile.ZipFile(OUT/'run-raw.zip') as z:
    assert z.read('frame-vectors.json')==(OUT/'frame-vectors.json').read_bytes()
cases=[c for c in read(OUT/'frame-vectors.json')['vectors'] if c['input'][0]==303]
assert len(cases)==12
fixture={'format':'LITO_BACKGROUND_REAL_IMG_ORIGINAL1','qualification_sha256':digest((OUT/'qualified.json').read_bytes()),
    'original_raw_archive_sha256':q['raw_archive_sha256'],'source':'original initializer + geometry + RGB555 pixel kernel; real warm IMG, synthetic gate and surface',
    'resource_loader_executed_by_original':False,'modern_viewport_adapter_verified':False,'cases':cases}
p=ROOT/'LiTo381-Modern/tests/fixtures/native-background-real-original.json';assert not p.exists()
p.write_text(json.dumps(fixture,separators=(',',':'))+'\n',encoding='utf-8');assert read(p)==fixture
print(json.dumps({'sha256':digest(p.read_bytes()),'bytes':p.stat().st_size,'real_IMG_frames':len(cases)}))
