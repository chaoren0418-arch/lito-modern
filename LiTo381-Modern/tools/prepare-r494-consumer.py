from r491_common import *
p=P/'tools/r494-source-stream-consumer.rs';s=p.read_text(encoding='utf-8')
a='record(&p,&rt,"EOF inside LE16 header or seed body retires currentepoch with exact incomplete state",&mut trace);'
b='let e=begin(&mut rt)?;seed(&mut rt,&e,&handshake,counter)?;let mut c=counter;let encrypted_partial=encrypted(&mode,&mut c);rt.feed_source_stream_candidate(&e,&encrypted_partial[..5])?;let end=rt.end_source_stream_candidate(&e)?;assert_eq!(end.terminal,Terminal::BodyEof);assert_eq!((end.before.counter,end.before.body_bytes,end.before.body_expected),(Some(counter),3,Some(mode.len())));record(&p,&rt,"EOF inside LE16 header,seed orencryptedbody retires exact incomplete state without counter advance",&mut trace);'
assert s.count(a)==1;s=s.replace(a,b)
s=s.replace('let delta=[213,2,*777u32.to_le_bytes().first().unwrap(),3,0,0,1];','let delta=[213,2,9,3,0,0,1];')
p.write_text(s,encoding='utf-8')
print('R494 consumer finalprebuild:seed/encrypted EOF,LE32 id777 explicit;no productionchanges')
