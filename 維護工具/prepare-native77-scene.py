"""Prepare the real native scene integration without touching production sources."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];SRC=ROOT/'LiTo381-Modern/src/modern_lito'
OUT=ROOT/'維護工具/candidates/native77-scene-integration'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def put(name,s):(OUT/name).write_text(s,encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']
baseline=ROOT/'驗證紀錄/current/native544-integration/validation-20261017-172422-276459'
inputs=read(baseline/'compiled-inputs.json');assert len(inputs)==1029
for name,h in inputs.items():assert sha(ROOT/name)==h
old=ROOT/'維護工具/candidates/native77-current-assets'
for name,h in queue['current_native77_lifecycle_candidate']['core_pins'].items():assert sha(name)==h
OUT.mkdir()
names=['mod.rs','native_world.rs','native_world_effects.rs','native_world_layers.rs','native_world_scene.rs','native_world_view.rs','native_audio_delivery.rs','disintegrate_assets.rs']
for name in names:put(name,(SRC/name).read_text(encoding='utf-8-sig'))
put('disintegrate_assets.rs',(old/'disintegrate_assets.rs').read_text(encoding='utf-8-sig'))
put('native_disintegrate.rs',(old/'native_disintegrate.rs').read_text(encoding='utf-8-sig'))
record=dict(prepared_utc=datetime.now(timezone.utc).isoformat(),baseline_source_manifest_sha256=queue['current_product_identity']['source_manifest_sha256'],
    baseline_inputs=str(baseline/'compiled-inputs.json'),baseline={name:sha(SRC/name) for name in names},new_sources=['native_disintegrate.rs','native_disintegrate_tests.rs'],production_source_modified=False)
with zipfile.ZipFile(OUT/'baseline.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in names:z.write(SRC/name,name)
with zipfile.ZipFile(OUT/'baseline.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in record['baseline'].items())
put('preparation.json',json.dumps(record,ensure_ascii=False,indent=2)+'\n')
print(OUT)
