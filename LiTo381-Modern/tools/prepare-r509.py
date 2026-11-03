from r491_common import *
import shutil,uuid,difflib
root=P/'target/r509-production-source';root.mkdir();pins=[];diff=[];changes=[];prior={q['path']:q for q in read(L/'r508z-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);a=pin(p);a['snapshot']=str(d);pins.append(a);old=prior.get(str(p))
 if not old or old['sha256']!=a['sha256']:
  changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text(encoding='utf-8').splitlines(True) if old else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r509-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r509-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous=pin(L/'r508-zero-directory-guard-addendum.json'),GUI=0))
root=P/'target/r509-player-entry-candidate';root.mkdir();main=root/'LiToModern-R509.rs';main.write_text((P/'target/r508z-player-entry-candidate/LiToModern-R508Z.rs').read_text(encoding='utf-8').replace('r508z','r509').replace('R508Z','R509'),encoding='utf-8');v=read(L/'r508z-entry-source-contract.json');v['candidate_main']=pin(main);save(L/'r509-entry-source-contract.json',v)
s=(P/'tools/build-r508z-player.ps1').read_text(encoding='utf-8').replace('r508z','r509').replace('R508Z','R509').replace('r509-library-build','r509c-library-build');(P/'tools/build-r509-player.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r508z-consumer.ps1').read_text(encoding='utf-8').replace('r508z','r509').replace('r509-zero-consumer','r509-character-consumer').replace('r509-library-build','r509c-library-build');(P/'tools/build-r509-consumer.ps1').write_text(s,encoding='utf-8')
prefix=(P/'tools/r508-bootstrap-consumer.rs').read_text(encoding='utf-8').split('use lito_modern::modern_lito::native_city_runtime::login_bootstrap')[0];(P/'tools/r509-character-consumer.rs').write_text(prefix+(P/'tools/r509-character-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
v=read(L/'r508c-java-inputs.json');old=Path(v['work']);work=P/'target'/('r509-java-'+uuid.uuid4().hex);work.mkdir()
for name in ['source','candidate','ddl','db']:shutil.copytree(old/name,work/name)
(work/'test-classes').mkdir();r=Path(r'D:/lito-dev-runtime/20261003-supervision/modern')/('nw-'+uuid.uuid4().hex);r.mkdir();shutil.copytree(work/'db',r/'db');(r/'generation-1').mkdir();v.update(work=str(work),runtime=str(r),previous_work_read_only=str(old));save(L/'r509-java-inputs.json',v)
s=(P/'tools/run-r508c-integration.py').read_text(encoding='utf-8').replace('r508c','r509').replace('r508-','r509-').replace('R508NativeFixtureMain','R509NativeFixtureMain');(P/'tools/run-r509-integration.py').write_text(s,encoding='utf-8')
# Update only evidence queues. These dependencies do not start a second line.
pending=[]
for p,sha in [(r'D:/lineage開發/3-lito開發/lito脫殼資源/skill-pane-render-initialization-contract.json','00262DDFF19C9392CFAEDDC53F38DE46725D99D0011A48F3FEF046C02754C1DE'),(r'D:/lineage開發/3-lito開發/lito伺服器/native-login-check-tail-delta.json','E1B8F323C74A51C8AC22D3CCB5802B89A9C84C455D21AA0236CB5F43B6DCAD1D')]:
 e=pin(p);assert e['sha256']==sha;pending.append(e)
save(L/'r509-additional-dependencies.json',dict(contracts=pending,render='queued after R509 with previous batch5/6; no GUI/render adoption yet',S0='Exact long opcode0 ordinary branch remains strict safety validation; private conditional dispatch/alternate suppression not generally emulated. Native unbounded reader does not imply crash/desync; Java missing4-byte extension stays explicit adapter.',new_tests=0))
print('R509 fresh namespace',r,'source files',len(pins),'changes',len(changes))
