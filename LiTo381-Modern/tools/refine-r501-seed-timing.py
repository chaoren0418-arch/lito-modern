from r491_common import *
import shutil,difflib
root=P/'target/r501-pre-coalesced-evidence';root.mkdir(exist_ok=False);pins=[]
for n in ['r501-production-source-snapshot.json','r501-source-scope.json','r501-entry-source-contract.json','r501-player-candidate.json','r501-consumer-candidate.json','r501-consumer-exit.json','r501-event-details.json','r501-event-partial.json','r501-consumer.stdout.log','r501-consumer.stderr.log','r501-source-difference.patch']:
 d=root/n;shutil.copyfile(L/n,d);pins.append(pin(d))
prod=read(L/'r501-production-source-snapshot.json');consumer=read(L/'r501-consumer-candidate.json');player=read(L/'r501-player-candidate.json')
save(L/'r501-pre-coalesced-validation.json',dict(state='R501_PRELIMINARY18_FLOW_PASS_IMMUTABLE_NOT_FINAL_SOURCE',artifacts=pins,production_source_snapshots=[dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256'])for q in prod['source_pins']],player=player['executable'],production_library=player['production_library'],consumer=dict(path=consumer['path'],bytes=consumer['bytes'],sha256=consumer['sha256']),consumer_sources=[dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256'])for q in consumer['source_pins']],flows=18,consumer_executions=1,exit_code=0,reason='Source1/d ->480560 timing requires compatibility reservation immediately at seed dispatch, even if actual58 is in same read batch. New final-source focused test only; prior18 PASS not rerun.',GUI=0))
v=P/'tools/verify-r501-segment.py';d=root/v.name;shutil.copyfile(v,d);save(L/'r501-verifier-attempt1-failure.json',dict(state='R501_CHECKER_SYMBOL_ALIAS_FAILURE_PRESERVED',checker=pin(d),reason='ASM uses offset nullsub_48 alias, not literal641350. PE mov eax641350 is already pinned. Static verifier corrected alias, production issue separate; consumer not rerun.'))
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8');needle='self.source_send.seed_from_source(counter)?;StreamEvent';assert needle in s;s=s.replace(needle,'self.source_send.seed_from_source(counter)?;if self.login_transport.is_some(){self.prepare_source_compatibility_fixture(epoch)?;}StreamEvent');p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_login_transport.rs';s=p.read_text(encoding='utf-8');s=s.replace(',StreamBatch,StreamIngress}',',StreamBatch}');needle='  if batch.terminal.is_none()&&batch.events.iter().any(|e|matches!(e.ingress,StreamIngress::SeedAccepted{..})){self.prepare_source_compatibility_fixture(epoch)?;}\n';assert needle in s;s=s.replace(needle,'');p.write_text(s,encoding='utf-8')
# Final immutable production source and rebuilt player in a new directory.
old=read(L/'r500-production-source-snapshot.json');diffs=[];delta=[]
for q in old['source_pins']:
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
for name in ['native_login_wire.rs','native_login_transport.rs']:
 p=P/'src/modern_lito'/name;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
diff=L/'r501-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');out=P/'target/r501-final-production-source';out.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=out/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
assert len(pins)==615;save(L/'r501-production-source-snapshot.json',dict(state='R501_FINAL615_SOURCE_WITH_SOURCE_SEED_BOUNDARY_COMPATIBILITY_TIMING',source_pins=pins,source_difference=pin(diff),preliminary_validation=pin(L/'r501-pre-coalesced-validation.json'),GUI=0))
scope=read(L/'r501-source-scope.json')
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
save(L/'r501-source-scope.json',scope);save(L/'source-change-scope.json',scope)
entry=read(L/'r501-entry-source-contract.json');out=P/'target/r501-final-player-entry-candidate';out.mkdir(exist_ok=False);p=out/'LiToModern-R501.rs';shutil.copyfile(entry['candidate_main']['path'],p);entry['candidate_main']=pin(p);save(L/'r501-entry-source-contract.json',entry)
p=P/'tools/build-r501-player-candidate.ps1';s=p.read_text(encoding='utf-8').replace('r501d-library-build','r501e-library-build').replace('target\\r501-player-entry-candidate','target\\r501-final-player-entry-candidate').replace('r501-player-build.stdout.log','r501-final-player-build.stdout.log').replace('r501-player-build.stderr.log','r501-final-player-build.stderr.log');p.write_text(s,encoding='utf-8')
v=P/'tools/verify-r501-segment.py';s=v.read_text(encoding='utf-8').replace("['641350','0B104FA87h'","['offset nullsub_48','0B104FA87h'").replace("production['source_difference']['bytes']==38187","production['source_difference']['bytes']=="+str(diff.stat().st_size));s=s.replace("asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');pe=","asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');asm_lines=asm.splitlines();pe=").replace("asm.splitlines()[int(n)-1]","asm_lines[int(n)-1]");v.write_text(s,encoding='utf-8')
p=P/'tools/map-r501-findings.py';s=p.read_text(encoding='utf-8').replace('fixed31byte salt','fixed33byte salt').replace('peer fixture deliberately orders version after compatibility','compatibility is reserved at seed dispatch even if actual58 follows in the same read batch');p.write_text(s,encoding='utf-8')
print('R50118 PASS preserved; final seed dispatch timing corrected;615sources;diff',diff.stat().st_size)
