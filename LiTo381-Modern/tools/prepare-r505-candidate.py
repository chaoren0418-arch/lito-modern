from r491_common import *
import shutil,difflib
old=read(L/'r504-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_source_connection.rs'}
new=P/'src/modern_lito/native_entry_transfer.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r505-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r505-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
save(L/'r505-production-source-snapshot.json',dict(source_pins=pins,changed=delta,added=[new.name],unchanged=616,source_difference=pin(diff),GUI=0))
player=read(L/'r504-player-candidate.json');root=P/'target/r505-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R505.rs';main.write_text(Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r504','r505').replace('R504','R505'),encoding='utf-8');entry=read(L/'r504-entry-source-contract.json');entry.update(state='R505_TRANSFER_PREFIX_SHARED_ENTRY_FIXED_CANDIDATE_CHANNEL_PATH_OPEN',candidate_main=pin(main),previous_R504_player_record=pin(L/'r504-player-candidate.json'));save(L/'r505-entry-source-contract.json',entry)
s=(P/'tools/build-r504-player-candidate.ps1').read_text(encoding='utf-8').replace('r504','r505').replace('R504','R505');(P/'tools/build-r505-player-candidate.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r504-refusal-consumer.ps1').read_text(encoding='utf-8').replace('r504','r505').replace('R504','R505').replace('refusal-consumer','transfer-consumer').replace('refusal_consumer','transfer_consumer').replace("'r503-candidate.json'","'r504-refusal-consumer-candidate.json'").replace('Expected218','Expected221').replace('$pins.Count -ne 218','$pins.Count -ne 221')
s=s.replace("(Join-Path $project 'src\\modern_lito\\native_player_entry_commands.rs'))", "(Join-Path $project 'src\\modern_lito\\native_player_entry_commands.rs'),(Join-Path $project 'src\\modern_lito\\native_entry_transfer.rs'))")
(P/'tools/build-r505-transfer-consumer.ps1').write_text(s,encoding='utf-8')
helpers=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8').split('fn main(')[0].replace('r503','r505').replace('R503','R505');(P/'tools/r505-transfer-consumer.rs').write_text(helpers+(P/'tools/r505-transfer-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
print(f'R505{len(pins)} sources/2 changed1 new616 unchanged/{diff.stat().st_size} diffbytes;221 fixed pairs expected')
