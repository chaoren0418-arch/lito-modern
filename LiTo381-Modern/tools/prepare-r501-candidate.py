from r491_common import *
import shutil,difflib
old=read(L/'r500-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_login_form.rs','native_source_send.rs'},delta
added=['native_login_wire.rs','native_login_transport.rs']
for name in added:
 p=P/'src/modern_lito'/name;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
diff=L/'r501-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r501-production-source';root.mkdir(exist_ok=False);files=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(files)==615;pins=[]
for p in files:
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);assert pin(d)['sha256']==q['sha256'];q['snapshot']=str(d);pins.append(q)
save(L/'r501-production-source-snapshot.json',dict(state='R501_ALL615_PRODUCTION_SOURCE_FROZEN_BEFORE_NEW_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r500-source-scope.json');scope.pop('changed_from_fixed_R499',None);scope.pop('added_from_fixed_R499',None);scope.update(state='R501_TYPED_SYNTHETIC_LOOPBACK_LOGIN_RELOGIN_SEND_SCOPE',source_files=615,changed_from_fixed_R500=delta,added_from_fixed_R500=added,scope='4changed+2new/609previousunchanged; Modern own outputs and existing unique ASCII scratch only; synthetic loopback only; no Java/SQL/server/realcredentials/GUI')
for name in added:scope['new_source_files'].append(dict(path='src/modern_lito/'+name,sha256=pin(P/'src/modern_lito'/name)['sha256']))
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r501-source-scope.json',scope);save(L/'source-change-scope.json',scope)
player=read(L/'r500-player-candidate.json');root=P/'target/r501-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R501.rs';main.write_text(Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r500','r501').replace('R500','R501'),encoding='utf-8');entry=read(L/'r500-entry-source-contract.json');entry.update(state='R501_STATIC_PLAYER_WITH_SYNTHETIC_TYPED_LOOPBACK_RUNTIME_API',candidate_main=pin(main),previous_R500_player_record=pin(L/'r500-player-candidate.json'),UI_assets='sameR498skin; same local Modern form; actual fixture transport only via explicit synthetic API; production/profile network route unchanged; Sending/AwaitingReply cancel disabled; player GUI0')
save(L/'r501-entry-source-contract.json',entry)
for oldname,newname in [('build-r500-player-candidate.ps1','build-r501-player-candidate.ps1'),('build-r500-login-lifecycle-consumer.ps1','build-r501-loopback-login-consumer.ps1')]:
 s=(P/'tools'/oldname).read_text(encoding='utf-8').replace('r500','r501').replace('R500','R501').replace('r501c-library-build','r501d-library-build')
 if 'consumer' in newname:
  s=s.replace('login-lifecycle-consumer','loopback-login-consumer').replace('login_lifecycle_consumer','loopback_login_consumer').replace("'r499-candidate.json'","'r500-candidate.json'").replace('R499 source snapshot','R500 source snapshot').replace('203','207')
  s=s.replace("src\\modern_lito\\native_login_form.rs","src\\modern_lito\\native_login_wire.rs").replace(",'src",",'src")
  needle="(Join-Path $project 'src\\modern_lito\\native_login_wire.rs'))";assert needle in s;s=s.replace(needle,"(Join-Path $project 'src\\modern_lito\\native_login_wire.rs'),(Join-Path $project 'src\\modern_lito\\native_login_transport.rs'))")
 (P/'tools'/newname).write_text(s,encoding='utf-8')
print('R501 frozen615sources/4changed2new/207pairs; actual diff',diff.stat().st_size,'bytes')
