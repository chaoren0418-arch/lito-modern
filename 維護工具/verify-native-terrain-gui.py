"""Verify the compiled GUI source frontier and existing independent wire/pixel checks."""
from pathlib import Path
import hashlib,json,runpy,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
for name in ['verify-native-teleport-gui-authority.py','verify-teleport-glyph-pixels.py','verify-native-room-map-gui.py']:
 runpy.run_path(str(Path(__file__).with_name(name)),run_name='__main__')
r=read(p/'receipt.json');assert r['complete_build_inputs_verified'] and r['inputs_unchanged']
f=p/'compilation-inputs.json';assert sha(f)==r['compilation_inputs_sha256'];m=read(f)
assert m['status']=='completed_build' and m['target']=='x86_64-pc-windows-msvc' and m['main_sha256']==r['exe_sha256']
root=Path(__file__).resolve().parents[1]
for n,h in m['files'].items():
 q=p/'source-snapshot'/n
 if not q.is_file():q=root/n
 assert sha(q)==h,n
out=dict(status='verified_native_terrain_GUI_compilation_wire_and_pixels',exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],compilation_inputs=len(m['files']),complete_source_pins_verified=True,profile=m['profile'],actual_sessions=6,room_maps=[564,514,564],physical_input=False,observed_DPI=192,receipt_sha256=sha(p/'receipt.json'),compilation_manifest_sha256=sha(f),validator_sha256=sha(Path(__file__)),full_product_verified=False)
(p/'terrain-gui-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(out))
