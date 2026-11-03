from r491_common import *
import shutil,difflib
old=read(L/'r505-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
diff=L/'r506-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r506-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
assert len(pins)==619
save(L/'r506-production-source-snapshot.json',dict(source_pins=pins,changed=delta,added=[],unchanged=len(pins)-len(delta),source_difference=pin(diff),GUI=0))
player=read(L/'r505-player-candidate.json');root=P/'target/r506-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R506.rs';main.write_text(Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r505','r506').replace('R505','R506'),encoding='utf-8');entry=read(L/'r505-entry-source-contract.json');entry.update(state='R506_SOURCE_CHANNEL_ROW_REDIRECT_SHARED_ENTRY_CANDIDATE',candidate_main=pin(main),previous_R505_player_record=pin(L/'r505-player-candidate.json'),skin_binding=pin(L/'r506-native-menu-skin-binding.json'),UI_assets='33 Modern controls; four source page40 channel normal/pressed IMG pairs and background; source S_FB second-vector rows; shared opaque EntryCommand callbacks; GUI0');save(L/'r506-entry-source-contract.json',entry)
s=(P/'tools/build-r505-player-candidate.ps1').read_text(encoding='utf-8').replace('r505','r506').replace('R505','R506').replace('r506a-library','r506b-library');(P/'tools/build-r506-player-candidate.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r505-transfer-consumer.ps1').read_text(encoding='utf-8').replace('r505','r506').replace('R505','R506').replace('r506a-library','r506b-library').replace("'r504-refusal-consumer-candidate.json'","'r505-transfer-consumer-candidate.json'").replace('Expected221','Expected223').replace('$pins.Count -ne 221','$pins.Count -ne 223');(P/'tools/build-r506-transfer-consumer.ps1').write_text(s,encoding='utf-8')
helpers=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8').split('fn main(')[0].replace('r503','r506').replace('R503','R506').replace('r498-native-menu-skin-binding.json','r506-native-menu-skin-binding.json').replace('let end = Instant::now() + Duration::from_secs(10);','let end = Instant::now() + Duration::from_secs(15);');(P/'tools/r506-transfer-consumer.rs').write_text(helpers+(P/'tools/r506-transfer-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
print(f'R506 {len(pins)} sources/{len(delta)}changed/{diff.stat().st_size}diff bytes;223 fixed consumer pairs expected')
