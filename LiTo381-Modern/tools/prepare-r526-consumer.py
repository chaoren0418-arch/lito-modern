from r491_common import *
s=(P/'tools/r525-mode-writer-consumer.rs').read_text('utf-8').split('include!')[0].replace('R525','R526')
s=s.replace('java_room_mode_request as mode_request','java_room_mode_request as mode_request,java_room_ready as ready_flow')
s=s.replace('r.configure_java_room_mode_request_adapter(mode_request_adapter()?)?;','r.configure_java_room_ready_adapter(ready_adapter()?)?;')
s=s.replace('let frames=data["setup_frames"].as_array().unwrap();','let mut all_trace=vec![];for round in 0..2 {let frames=data["setup_frames"].as_array().unwrap();')
s=s.replace('if bodies.is_empty(){break;}','if bodies.is_empty(){break;}if bodies==vec![vec![254]]{done.send(())?;break;}')
s=s.replace('command.len()==3&&command.starts_with(&[209,2])&&matches!(command[2],1|2)','command.len()==2&&command[0]==210&&command[1]<=2').replace('relative mode expected command','participant ready expected command').replace('relative mode expectation','participant ready expectation')
s=s.replace('trace.extend(g.trace);Ok(trace)','trace.extend(g.trace);for v in &mut trace{v["round"]=json!(round);}all_trace.extend(trace);}Ok(all_trace)')
(P/'tools/r526-ready-consumer.rs').write_text(s+'include!("r526-ready-case.rs");\n','utf-8')
for kind in ['consumer','player']:
 s=(P/f'tools/build-r525-{kind}.ps1').read_text('utf-8').replace('r525','r526').replace('R525','R526').replace('r526-mode-writer','r526-ready')
 (P/f'tools/build-r526-{kind}.ps1').write_text(s,'utf-8')
print('R526 own2round normal-entry producer and compile scripts saved;no execution')
