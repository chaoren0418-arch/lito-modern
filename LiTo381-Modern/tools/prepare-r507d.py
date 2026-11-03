from r491_common import *
import shutil,uuid
f=L/'r507-second-attempt';f.mkdir();saved=[]
for name in ['r507c-integration-exit.json','r507c-integration-live-status.json','r507c-consumer.stdout.log','r507c-consumer.stderr.log','r507-event-partial.json']:
 p=L/name;d=f/name;shutil.copyfile(p,d);saved.append(pin(d))
save(f/'preservation.json',dict(files=saved,partial_passes=4,failure='fixture queued Expect(D9) before sending F4, preventing the trigger packet from ever being sent; product not implicated',owned_retirement=read(L/'r507c-integration-exit.json'),remaining_only_next=True))
v=read(L/'r507c-java-inputs.json');r=Path(r'D:\lito-dev-runtime\20261003-supervision\modern')/('nw-'+uuid.uuid4().hex);r.mkdir();shutil.copytree(Path(v['work'])/'db',r/'db');(r/'generation-1').mkdir();v['runtime']=str(r);save(L/'r507d-java-inputs.json',v)
s=(P/'tools/r507-integration-consumer.rs').read_text(encoding='utf-8')
start=' // Actual production Java Protocol output, not a translated LTM1 namespace.';end=' // Two new host-qualified Back branches. Only initialization is reused.'
a=s.index(start);b=s.index(end);s=s[:a]+s[b:]
old='  for i in 1..4{e.steps.send(Step::Expect(tx[i].clone()))?;}\n  e.steps.send(Step::Send(bytes(&rx[0]["wire"])))?;sent(&e);'
assert old in s
s=s.replace(old,'  e.steps.send(Step::Send(bytes(&rx[0]["wire"])))?;sent(&e);for i in 1..4{e.steps.send(Step::Expect(tx[i].clone()))?;}')
s=s.replace('r507-event-', 'r507d-event-');(P/'tools/r507d-integration-consumer.rs').write_text(s,encoding='utf-8')
s=(P/'tools/build-r507c-integration-consumer.ps1').read_text(encoding='utf-8').replace('target/r507c-fixed-integration-consumer','target/r507d-fixed-integration-consumer').replace('r507-integration-consumer.rs','r507d-integration-consumer.rs').replace('r507c-consumer-build','r507d-consumer-build').replace('r507c-consumer-candidate','r507d-consumer-candidate');(P/'tools/build-r507d-integration-consumer.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/run-r507c-integration.py').read_text(encoding='utf-8').replace('r507c-java-inputs','r507d-java-inputs').replace('r507c-consumer-candidate','r507d-consumer-candidate').replace("L/'r507c-", "L/'r507d-")
(P/'tools/run-r507d-integration.py').write_text(s,encoding='utf-8')
print(str(r))
