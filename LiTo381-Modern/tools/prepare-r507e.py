from r491_common import *
import shutil,uuid
root=L/'r507-third-attempt';root.mkdir();saved=[]
for name in ['r507d-integration-exit.json','r507d-integration-live-status.json','r507d-consumer.stdout.log','r507d-consumer.stderr.log']:
 p=L/name;d=root/name;shutil.copyfile(p,d);saved.append(pin(d))
save(root/'preservation.json',dict(files=saved,failure='second fixture queue dependency: Expect saved alias preceded Send203, and ExpectCA100 preceded SendC8; preserved all failed binaries/source and own process retirement',new_directories_not_rerun=True))
s=(P/'tools/r507d-integration-consumer.rs').read_text(encoding='utf-8')
a='target.steps.send(Step::Expect(o["target_tx"][0].clone()))?;target.steps.send(Step::Send([bytes(&o["target_rx"][0]["wire"]),bytes(&o["target_rx"][1]["wire"]),bytes(&o["target_rx"][2]["wire"])].concat()))?;sent(&target);'
b='target.steps.send(Step::Send([bytes(&o["target_rx"][0]["wire"]),bytes(&o["target_rx"][1]["wire"]),bytes(&o["target_rx"][2]["wire"])].concat()))?;sent(&target);target.steps.send(Step::Expect(o["target_tx"][0].clone()))?;'
assert a in s;s=s.replace(a,b)
a='target.steps.send(Step::Expect(o["target_tx"][1].clone()))?;target.steps.send(Step::Send(bytes(&o["target_rx"][3]["wire"])))?;sent(&target);'
b='target.steps.send(Step::Send(bytes(&o["target_rx"][3]["wire"])))?;sent(&target);target.steps.send(Step::Expect(o["target_tx"][1].clone()))?;'
assert a in s;s=s.replace(a,b)
native=s[s.index(' // Genuine separate Java Main'):s.index(' fs::write(base.join("r507d-event-details.json"')]
host=s[:s.index(' // Genuine separate Java Main')]+s[s.index(' fs::write(base.join("r507d-event-details.json"'):]
host=host.replace('let java=PathBuf::from(std::env::args_os().nth(2).unwrap());','').replace('r507d-event','r507e-event').replace('fn sent(e:&Endpoint)', '#[track_caller]fn sent(e:&Endpoint)')
(P/'tools/r507e-host-consumer.rs').write_text(host,encoding='utf-8')
helpers=s.split('fn unhex(')[0].replace('r507d-event','r507n-event')
native=helpers+'fn main()->anyhow::Result<()>{let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let java=PathBuf::from(std::env::args_os().nth(2).unwrap());let base=p.join("logs/continuation-20261003");let mut trace=Vec::new();'+native+'fs::write(base.join("r507n-event-details.json"),serde_json::to_vec_pretty(&trace)?)?;println!("actual Java boundary recorded");Ok(())}'
(P/'tools/r507n-native-consumer.rs').write_text(native,encoding='utf-8')
for label,file in [('r507e','r507e-host-consumer.rs'),('r507n','r507n-native-consumer.rs')]:
 s=(P/'tools/build-r507d-integration-consumer.ps1').read_text(encoding='utf-8').replace('target/r507d-fixed-integration-consumer',f'target/{label}-fixed-consumer').replace('r507d-integration-consumer.rs',file).replace('r507d-consumer-build',label+'-consumer-build').replace('r507d-consumer-candidate',label+'-consumer-candidate');(P/'tools'/f'build-{label}-consumer.ps1').write_text(s,encoding='utf-8')
v=read(L/'r507d-java-inputs.json');r=Path(r'D:\lito-dev-runtime\20261003-supervision\modern')/('nw-'+uuid.uuid4().hex);r.mkdir();shutil.copytree(Path(v['work'])/'db',r/'db');(r/'generation-1').mkdir();v['runtime']=str(r);save(L/'r507n-java-inputs.json',v)
s=(P/'tools/run-r507d-integration.py').read_text(encoding='utf-8').replace('r507d-java-inputs','r507n-java-inputs').replace('r507d-consumer-candidate','r507n-consumer-candidate').replace("L/'r507d-", "L/'r507n-")
(P/'tools/run-r507n-integration.py').write_text(s,encoding='utf-8')
print(str(r))
