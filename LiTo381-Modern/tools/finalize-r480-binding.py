from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
old=read(L/'r479b-map-binding.json');m=read(L/'r480-resource-input.json');probe=read(P/'target/r480-multisector/reference-b/reference.json');assert probe['map_id']==552 and len(probe['sectors'])==4
b=m['batch']|dict(canonical_sha256=probe['canonical_sha256']);b.pop('name')
binding=old|dict(state='R480_CATALOG40_SEVEN_NATIVE_PREVIEWS_COMPLETE552_FOUR_SECTORS',bindings=old['bindings']+[b],unqualified_count=33,remaining_unqualified_maps=33,new_complete_single_sector_ids=[],new_complete_multisector_ids=[552],multisector_probe=pin(P/'target/r480-multisector/reference-b/reference.json'),camera_adaptation='Immutable lexicographic whole-map anchor; full projected sector extent signed bounds. Multi552 initial2672,468,800x600; legacy single1136,468. Modern inspection camera, no native spawn.',multi_sector_scope='Complete original map552 2x2 four sectors; X/Y and junction source geometry/depth sort. Other33 maps unqualified;573/563 Layer7 retained.')
(L/'r480-map-binding.json').write_text(json.dumps(binding,ensure_ascii=False,indent=2),encoding='utf-8')
print('Seven preview bindings: complete map552 four sectors; remaining33')
