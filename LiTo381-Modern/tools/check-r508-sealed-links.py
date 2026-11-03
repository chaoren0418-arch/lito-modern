from r491_common import *
h=L/'r508-login-source-game-handoff.json';v=read(h);checks={}
def walk(x):
 if isinstance(x,dict):
  if all(k in x for k in ['path','sha256','bytes']):
   actual=pin(x['path']);assert actual=={k:x[k] for k in ['path','sha256','bytes']},x
   checks[x['path']]=actual
  for y in x.values():walk(y)
 elif isinstance(x,list):
  for y in x:walk(y)
walk(v)
snapshot=read(L/'r508c-production-source-snapshot.json')
for e in snapshot['source_pins']:
 a=pin(e['path']);b=pin(e['snapshot']);assert a['sha256']==e['sha256']==b['sha256'] and a['bytes']==b['bytes']==e['bytes'];checks[a['path']]=a;checks[b['path']]=b
for e in read(L/'r508c-java-execution-inputs.json')['files']:walk(e)
save(L/'r508-sealed-link-verification.json',dict(handoff=pin(h),unique_files=len(checks),files=list(checks.values()),current_production_and_frozen_snapshot_pairs=len(snapshot['source_pins']),retirement=pin(L/'r508c-integration-exit.json'),historical_tests_rerun=False))
# Newly delivered contracts arrive after immutable R508 closure; preserve the
# handoff and append pending dependencies, without adding an old PASS as new.
pending=[]
for path,sha,use in [
 (r'D:/lineage開發/3-lito開發/lito伺服器/server-character-directory-consumer-contract.json','412748DE715782AEC88D00251DEE791A472EF7C180F905EDF43AA3A36B9464C7','Typed S_CB field interpretation/C_C9 later; current R508 already receives raw owned alias, no objectID/source reverse derivation'),
 (r'D:/lineage開發/3-lito開發/lito脫殼資源/skill-pane-layout-contract.json','45EBDED431F925A0F38066702E67F0AD79E9139939D793E201A96EEBD15E681F','After R508, targeted icon/pane draw consumption alongside prior batch5; no GUI/skill-owner upgrade'),
 (r'D:/lineage開發/3-lito開發/lito脫殼資源/native-icon-draw-contract.json','9BA690374734FECFE86CF95651E864515FFFDC78D71FE58C556974625AD95803','After R508, targeted transparency/offset/RGB conversion/fallback parity; no wire/Java change')]:
 e=pin(path);assert e['sha256']==sha,e;pending.append(dict(contract=e,state='queued_not_adopted',use=use))
save(L/'r508-postclosure-dependency-queue.json',dict(sealed_R508=pin(h),pending=pending,new_tests=0,GUI=0,Native_LOGIN_CHECK_extra_DWORD='Await separate precise PE/producer comparison; existing explicit AD368 adapter and strict Native extension validation preserved'))
v=read(P.parent/'監製進度.json');v['r508']['sealed_link_verification']=pin(L/'r508-sealed-link-verification.json');v['r508']['postclosure_dependency_queue']=pin(L/'r508-postclosure-dependency-queue.json');save(P.parent/'監製進度.json',v)
print(json.dumps({'unique_R508_files_verified':len(checks),'source_pairs':len(snapshot['source_pins']),'pending_contracts':len(pending),'sealed_R508_SHA':pin(h)['sha256']},ensure_ascii=False))
