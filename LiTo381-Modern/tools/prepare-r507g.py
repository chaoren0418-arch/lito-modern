from r491_common import *
s=(P/'tools/r507f-host-consumer.rs').read_text(encoding='utf-8').replace('r507f-event','r507g-event')
a='target.steps.send(Step::Expect(o["target_tx"][1].clone()))?;write_wait(&mut r,&mut m,&target,1);'
b='target.steps.send(Step::Expect(o["target_tx"][1].clone()))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().scene==Scene::AwaitAutomaticLobbyReply);write_wait(&mut r,&mut m,&target,1);'
assert a in s;s=s.replace(a,b);(P/'tools/r507g-host-consumer.rs').write_text(s,encoding='utf-8')
s=(P/'tools/build-r507f-consumer.ps1').read_text(encoding='utf-8').replace('r507f','r507g');(P/'tools/build-r507g-consumer.ps1').write_text(s,encoding='utf-8')
save(L/'r507-fifth-attempt-preserved.json',dict(candidate=read(L/'r507f-consumer-candidate.json'),stdout=pin(L/'r507f-consumer.stdout.log'),stderr=pin(L/'r507f-consumer.stderr.log'),exit=Path(L/'r507f-consumer.exit.txt').read_text(),reason='diagnostic proves actual old c15/new seed/alias replay succeeded; stale write_complete caused harness to block before polling new C8; add causal scene gate; fixed production unchanged'))
s=(P/'tools/r507u-timeout-consumer.rs').read_text(encoding='utf-8').replace('r507u-event','r507v-event')
s=s.replace('until(&mut r,&mut m,|r|r.source_transfer_snapshot().roster_count==Some(1));','until(&mut r,&mut m,|r|r.source_stream_snapshot().counter==Some(o["setup"]["receive_after_versions"].as_u64().unwrap()+3));')
s=s.replace('[bytes(&o["timeout_rx"][0]["wire"]),bytes(&o["timeout_rx"][1]["wire"]),bytes(&o["timeout_rx"][2]["wire"])]','[bytes(&o["timeout_rx"][0]["wire"]),bytes(&o["timeout_rx"][1]["wire"])]')
a='until(&mut r,&mut m,|r|r.source_channel_back().is_some());assert!(!r.source_transfer_snapshot().replay_sent);assert_eq!(r.source_navigation_snapshot().return_state,2);'
b='until(&mut r,&mut m,|r|r.source_stream_snapshot().counter==Some(o["setup"]["receive_after_versions"].as_u64().unwrap()+3));assert!(r.source_transfer_snapshot().replay_sent);assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitTransferRoster);assert!(r.source_send_snapshot().pending.is_none());'
assert a in s;s=s.replace(a,b).replace('retains BE1 until actual C8 success clears it, then normal Close joins all4 IO owners','retains BE1; received new3/203 does not replay a second alias, then normal Close joins all4 IO owners; no C8 confirmation claimed')
(P/'tools/r507v-timeout-consumer.rs').write_text(s,encoding='utf-8')
s=(P/'tools/build-r507u-timeout-consumer.ps1').read_text(encoding='utf-8').replace('r507u','r507v');(P/'tools/build-r507v-consumer.ps1').write_text(s,encoding='utf-8')
