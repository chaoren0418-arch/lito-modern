from r491_common import *
import shutil,difflib
old=read(L/'r503-final-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_entry_navigation.rs','native_city_runtime.rs','native_city_window.rs'}
diff=L/'r504-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r504-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
save(L/'r504-production-source-snapshot.json',dict(source_pins=pins,changed=delta,added=[],unchanged=615,source_difference=pin(diff),GUI=0))
player=read(L/'r503-final-player-candidate.json');root=P/'target/r504-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R504.rs';main.write_text(Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r503','r504').replace('R503','R504'),encoding='utf-8');entry=read(L/'r503-final-entry-source-contract.json');entry.update(state='R504_RECONNECT_REQUIRED_SHARED_ENTRY_FIXED_CANDIDATE',candidate_main=pin(main),previous_R503_player_record=pin(L/'r503-final-player-candidate.json'));save(L/'r504-entry-source-contract.json',entry)
s=(P/'tools/build-r503-final-player-candidate.ps1').read_text(encoding='utf-8').replace('r503','r504').replace('R503','R504').replace('r504e-library-build','r504a-library-build').replace('target\\r504-final-player-entry-candidate','target\\r504-player-entry-candidate').replace('r504-final-entry-source-contract','r504-entry-source-contract').replace('r504-final-player-candidate.json','r504-player-candidate.json').replace('r504-guard-player-build','r504-player-build');(P/'tools/build-r504-player-candidate.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r503-guard-consumer.ps1').read_text(encoding='utf-8').replace('r503','r504').replace('R503','R504').replace('r504e-library-build','r504a-library-build').replace('r504-guard-consumer','r504-refusal-consumer').replace('r504_guard_consumer','r504_refusal_consumer').replace('r504-fixed-guard-consumer','r504-fixed-refusal-consumer').replace('r504-final-player-entry-candidate','r504-player-entry-candidate').replace('r504-final-player-candidate.json','r504-player-candidate.json').replace("'r504-consumer-candidate.json'","'r503-candidate.json'").replace('Expected216','Expected218').replace('$pins.Count -ne 216','$pins.Count -ne 218')
(P/'tools/build-r504-refusal-consumer.ps1').write_text(s,encoding='utf-8')
old=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8');helpers=old[:old.index('fn main(')].replace('r503','r504').replace('R503','R504');(P/'tools/r504-refusal-consumer.rs').write_text(helpers+(P/'tools/r504-refusal-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
print('R504618frozen/3changed615unchanged/',diff.stat().st_size,'diffbytes; one newfocusedconsumer218pairs')
