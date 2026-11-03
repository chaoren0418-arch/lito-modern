from r491_common import *
s=(P/'tools/r507t-timeout-consumer.rs').read_text(encoding='utf-8')
a='primary.steps.send(Step::Expect(o["target_tx"][0].clone()))?;primary.steps.send(Step::Send([bytes(&o["target_rx"][0]["wire"]),bytes(&o["target_rx"][1]["wire"]),bytes(&o["target_rx"][2]["wire"])].concat()))?;sent(&primary);'
b='primary.steps.send(Step::Send([bytes(&o["target_rx"][0]["wire"]),bytes(&o["target_rx"][1]["wire"]),bytes(&o["target_rx"][2]["wire"])].concat()))?;sent(&primary);primary.steps.send(Step::Expect(o["target_tx"][0].clone()))?;'
assert a in s;s=s.replace(a,b).replace('r507t-event','r507u-event').replace('rx.recv_timeout(Duration::from_secs(20))','rx.recv_timeout(Duration::from_secs(35))').replace('fn sent(e:&Endpoint)','#[track_caller]fn sent(e:&Endpoint)')
(P/'tools/r507u-timeout-consumer.rs').write_text(s,encoding='utf-8')
s=(P/'tools/build-r507t-timeout-consumer.ps1').read_text(encoding='utf-8').replace('target/r507t-fixed-timeout-consumer','target/r507u-fixed-timeout-consumer').replace('r507t-timeout-consumer.rs','r507u-timeout-consumer.rs').replace('r507t-consumer-build','r507u-consumer-build').replace('r507t-consumer-candidate','r507u-consumer-candidate');(P/'tools/build-r507u-timeout-consumer.ps1').write_text(s,encoding='utf-8')
