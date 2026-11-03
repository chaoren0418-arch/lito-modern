from r491_common import *
import shutil
archive=L/'r506-first-attempt';archive.mkdir(exist_ok=False)
for name in ['r506-transfer-consumer-candidate.json','r506-transfer-consumer-exit.json','r506-transfer-consumer.stdout.log','r506-transfer-consumer.stderr.log','r506-event-partial.json','r506-transfer-consumer-build.stdout.log','r506-transfer-consumer-build.stderr.log']:
 shutil.copyfile(L/name,archive/name)
save(archive/'preservation.json',dict(files=[pin(p)for p in archive.glob('*')],reason='test harness blocked waiting for target compatibility completion without Runtime polling; same-address/row channel-lobby first-attempt assertions passed; entire attempt remains failed',historical_PASS_reruns=0))
p=P/'tools/r506-transfer-consumer-body.rs';s=p.read_text(encoding='utf-8');s=s.replace('until(&mut r,&mut m,|r|r.source_actor_snapshot().version.is_some());sent(&target);','until(&mut r,&mut m,|r|r.source_actor_snapshot().version.is_some()&&r.source_send_snapshot().pending.is_none());sent(&target);')
assert 'version.is_some()&&r.source_send_snapshot().pending.is_none()' in s
p.write_text(s,encoding='utf-8')
print('Failed first consumer source/exe/logs retained; target handshake wait repaired')
