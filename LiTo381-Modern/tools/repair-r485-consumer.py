from pathlib import Path
P=Path(__file__).resolve().parent.parent
p=P/'tools/r485-consumer-body.rs';t=p.read_text(encoding='utf-8')
t=t.replace('map.sector_bindings()[0]','map.bindings()[0]')
t=t.replace('assert_eq!(n,parse_s32(&b).unwrap())','same(n.clone(),parse_s32(&b).unwrap())')
t=t.replace('assert_eq!(parse_native_s32(&raw).unwrap(),parse_s32(&raw).unwrap(),"old37 Native parser changed parsed records")','same(parse_native_s32(&raw).unwrap(),parse_s32(&raw).unwrap())')
assert 'sector_bindings' not in t
p.write_text(t,encoding='utf-8')
old=(P/'tools/r484-batch-consumer.rs').read_text(encoding='utf-8')
(P/'tools/r485-batch-consumer.rs').write_text(old[:old.index('fn study527(')]+t,encoding='utf-8')
build=(P/'tools/build-r485-batch-consumer.ps1').read_text(encoding='utf-8').replace('r485-fixed-batch-consumer','r485c-fixed-batch-consumer').replace('r485-consumer-','r485c-consumer-').replace('r485-consumer.','r485c-consumer.')
(P/'tools/build-r485c-batch-consumer.ps1').write_text(build,encoding='utf-8')
