from r491_common import *
import shutil,difflib
root=P/'target/r508z-production-source';root.mkdir();pins=[];old=read(L/'r508c-production-source-snapshot.json');changes=[];patch=[]
for e in old['source_pins']:
 p=Path(e['path']);d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);a=pin(p);a['snapshot']=str(d);pins.append(a)
 if a['sha256']!=e['sha256']:changes.append(str(p));patch.append(''.join(difflib.unified_diff(Path(e['snapshot']).read_text(encoding='utf-8').splitlines(True),p.read_text(encoding='utf-8').splitlines(True),fromfile=e['snapshot'],tofile=str(p))))
assert changes==[str(P/'src/modern_lito/native_login_bootstrap.rs')],changes
f=L/'r508z-source-difference.patch';f.write_text('\n'.join(patch),encoding='utf-8');save(L/'r508z-production-source-snapshot.json',dict(source_pins=pins,changes=changes,patch=pin(f),previous_finite_R508=pin(L/'r508-login-source-game-handoff.json')))
root=P/'target/r508z-player-entry-candidate';root.mkdir();main=root/'LiToModern-R508Z.rs';main.write_text((P/'target/r508q-player-entry-candidate/LiToModern-R508Q.rs').read_text(encoding='utf-8').replace('r508q','r508z').replace('R508Q','R508Z'),encoding='utf-8');v=read(L/'r508q-entry-source-contract.json');v['candidate_main']=pin(main);save(L/'r508z-entry-source-contract.json',v)
s=(P/'tools/build-r508q-player.ps1').read_text(encoding='utf-8').replace('r508q','r508z').replace('R508Q','R508Z').replace('r508c-library-build','r508z-library-build');(P/'tools/build-r508z-player.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r508c-consumer.ps1').read_text(encoding='utf-8').replace('r508c','r508z').replace('r508-bootstrap-consumer','r508z-zero-consumer');(P/'tools/build-r508z-consumer.ps1').write_text(s,encoding='utf-8')
prefix=(P/'tools/r508-bootstrap-consumer.rs').read_text(encoding='utf-8').split('use lito_modern::modern_lito::native_city_runtime::login_bootstrap')[0]
(P/'tools/r508z-zero-consumer.rs').write_text(prefix+(P/'tools/r508z-zero-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
