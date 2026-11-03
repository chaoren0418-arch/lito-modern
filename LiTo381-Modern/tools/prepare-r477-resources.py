"""Own exact additional Native map; catalog40 and complete original source scope."""
from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r477-multi-map';O.mkdir(exist_ok=False);(O/'source').mkdir();(O/'native-map/514').mkdir(parents=True)
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
m=json.loads((L/'r474-mode-map-binding.json').read_text(encoding='utf-8'));assert pin(L/'r474-mode-map-binding.json')['sha256']=='EB3FF940E1944812D0AA30A6016E9EAEAAC37ADF38210703B43BC96E5B44AFD2'
row=next(v for v in m['rows'] if (v['mode'],v['logical_map'])==(1,514));assert len(row['native_sector_files'])==1;original=row['native_sector_files'][0];source=Path(original['path']);assert pin(source)['sha256']==original['sha256'];dst=O/'native-map/514'/source.name;shutil.copyfile(source,dst);assert pin(dst)['sha256']==original['sha256']==pin(source)['sha256']
old=json.loads(Path(m['selected_provider']['path']).read_text(encoding='utf-8'));oldmap=old['native_map']
metadata=dict(state='R477_EXACT_NATIVE514_SINGLE_COMPLETE_SECTOR_PINNED_FOR_CANONICAL_PROBE',map_id=514,mode=1,sector=pin(dst),original_sector=original,sector_x=0x8000,sector_y=0x8000,map_root=str(O/'native-map'),archive_root=str(Path(m['selected_archives'][0]['path']).parent),archives=m['selected_archives'],catalog40=[v['original_table'] for v in m['rows']],catalog_source=pin(L/'r474-mode-map-binding.json'),LTtext_index=m['LTtext_index'],LTtext_pak=m['LTtext_pak'],table_decoded=m['table_decoded'],existing564=dict(mode=1,map_id=564,sector=oldmap,sector_x=0x7fff,sector_y=0x8000,map_root=str(Path(oldmap['path']).parent.parent),canonical_sha256=oldmap['canonical_sha256']),skin_binding=pin(L/'r476-native-menu-skin-binding.json'),remaining38='Catalog names/mode/map known; sector/archive/render recipe not R477-qualified; refuse preview without fallback',GUI=0,LOADED=False)
(L/'r477-resource-probe-input.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2),encoding='utf-8')
for address in ['46B7D0','4645D0','4646C0','474350','474360','4744F0','414A80','414BD0','414C00']:
 f=P/'target/r476-native-menu-skin/source'/f'asm-{address}-through-next-proc.txt';shutil.copyfile(f,O/'source'/f.name)
print(json.dumps(dict(map=514,sector_bytes=pin(dst)['bytes'],catalog_rows=40)))
