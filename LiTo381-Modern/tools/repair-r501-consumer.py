from r491_common import *
import shutil
root=P/'target/r501-consumer-build-attempt1';root.mkdir(exist_ok=False);pins=[]
for p in [P/'tools/r501-loopback-login-consumer.rs',P/'tools/build-r501-loopback-login-consumer.ps1',L/'r501-consumer-build.stderr.log',L/'r501-consumer-build.stdout.log']:
 d=root/p.name;shutil.copyfile(p,d);pins.append(pin(d))
save(L/'r501-consumer-build-failure.json',dict(state='R501_CONSUMER_FIRST_COMPILE_FAILED_NO_EXECUTION',exit_code=1,reason='consumer compared public Option<u64> epoch serial with private Epoch capability, E0308/E0277; corrected assertion to previously observed serial; production unchanged',files=pins,consumer_executions=0))
p=P/'tools/r501-loopback-login-consumer.rs';s=p.read_text(encoding='utf-8');s=s.replace('let connection=rt.source_loopback_snapshot().unwrap();','let epoch_serial=rt.source_connection_epoch();let connection=rt.source_loopback_snapshot().unwrap();');s=s.replace('Some(e.clone())','epoch_serial');p.write_text(s,encoding='utf-8')
p=P/'tools/build-r501-loopback-login-consumer.ps1';s=p.read_text(encoding='utf-8').replace('target\\r501-fixed-event-consumer','target\\r501-fixed-event-consumer-attempt2').replace('r501-consumer-build.stdout.log','r501-consumer-build-attempt2.stdout.log').replace('r501-consumer-build.stderr.log','r501-consumer-build-attempt2.stderr.log');p.write_text(s,encoding='utf-8')
print('R501 first consumer compile failure preserved; public epoch serial assertion repaired; no production change')
