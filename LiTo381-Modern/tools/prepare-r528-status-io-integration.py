import pathlib, json, hashlib, shutil, difflib, datetime
P=pathlib.Path(__file__).resolve().parent.parent
L=P/'logs/continuation-20261003'
B=pathlib.Path('D:/lito-dev-runtime/20261003-supervision/server/modern-485-provider-frozen-aa561cbbe3d74e1b80f3d8858c1e576f')
ROOT=P.parent.parent/'lito伺服器/server-modern-485F-status-io-cold-repair.json'
OUT=L/'r528-status-io-accepted'
def read(p):return json.loads(pathlib.Path(p).read_text('utf-8-sig'))
def pin(p):
 p=pathlib.Path(p);raw=p.read_bytes();return dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n') as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
def text(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n') as f:f.write(v)
assert pin(ROOT)['sha256']=='6C2C82073BB3DB18FC2121A7C74FAEF4E7A5DF28376874C171F4888BE99ED4EE'
assert pin(ROOT)['bytes']==5522
root=read(ROOT);OUT.mkdir()
checked={str(ROOT):pin(ROOT)}
for key in ['handoff','manifest','launch_integrity','launcher','publisher','reader','delta','tests','Modern_compatibility_ack']:
 expected=root[key];assert pin(expected['path'])==expected;checked[expected['path']]=expected
manifest=read(B/'manifest.json');launch=read(B/'launch-integrity.json')
assert len(manifest['files'])==122 and len(launch['files'])==26
for collection in [manifest['files'],launch['files']]:
 for row in collection:
  rel=pathlib.PurePosixPath(row['path']);assert not rel.is_absolute() and '..' not in rel.parts
  p=B/row['path'];actual=pin(p);assert actual['bytes']==row['bytes'] and actual['sha256']==row['sha256'];checked[str(p)]=actual
for source,dest in [(ROOT,'accepted-status-io-root.json'),(B/'evidence/Modern-status-io-cold-handoff.json','accepted-status-io-handoff.json'),(B/'manifest.json','accepted-status-io-manifest.json'),(B/'launch-integrity.json','accepted-status-io-launch.json')]:
 shutil.copyfile(source,OUT/dest)
save(L/'r528-status-io-cold-adoption.json',dict(status='PIN_VERIFIED_READ_ONLY_COLD_ADOPTION',root=pin(ROOT),bundle=str(B),payload_count=122,launch_count=26,unique_checked=len(checked),checked=list(checked.values()),copies=[pin(p) for p in OUT.iterdir()],server_writes=0,provider_starts=0,credentials_reads=0,PIDqueries=0,retired_READY_reads=0))
old=P/'tools/r528-provider-status-consistent-proposal.rs'
s=old.read_text('utf-8')
assert pin(old)['sha256'].startswith('E7083268')
for a,b in [
 ('modern-485-provider-frozen-7f8b15934fc34983bca934785a4c5370',B.name),
 ('4646D28B95674D2415D5CBAFBA1A365D67C44C00BD8F5ED1B537D42530E67D73',pin(ROOT)['sha256']),
 ('758D0768FE117CDBA02E81D11C516BA93AAEF6BBA67BC2A1D2733DCF4C191327',root['handoff']['sha256']),
 ('AC320ACBD656DB763B32D842857B22464A09D77D621612AAF9A7C61B95B64C11',root['launch_integrity']['sha256']),
 ('"nw-97cd2af6b99842238b90bcef2ba99803"|"nw-6fdb7420a2bc4b8492dd7cb90ff270ed"','"nw-97cd2af6b99842238b90bcef2ba99803"|"nw-6fdb7420a2bc4b8492dd7cb90ff270ed"|"nw-975e97ed0f74480c9c92edd7d6b7d51e"')]:
 assert a in s;s=s.replace(a,b)
insertion='''const MANIFEST_SHA:&str="511B44E9FD136A6EBDBD7245D8FEAAA4773E09C6C695E590BD551AB7FD2E8634";
const STATUS_POLL_INTERVAL:Duration=Duration::from_millis(200);
fn claim_status_poll(last:&std::cell::Cell<Option<Instant>>,now:Instant)->bool{
 if last.get().is_some_and(|previous|now.saturating_duration_since(previous)<STATUS_POLL_INTERVAL){return false;}
 last.set(Some(now));true
}
fn publication_diagnostic(s:&Value)->Result<Value>{
 let d=&s["status_publication"];
 ensure!(d["maximum_move_attempts"]==6&&d["retry_delay_ms"]==20&&["denied_moves","recovered_publications","exhausted_publications"].iter().all(|k|d[*k].as_u64().is_some()),"accepted publication diagnostic schema required");
 let first=d.get("first_error").context("first_error diagnostic absent")?;
 ensure!(first.is_null()||(first["utc"].as_str().is_some()&&first["type"]=="AccessDeniedException"&&first["message"].as_str().is_some()&&first["cause_confirmed"]==false),"publication first_error diagnostic schema invalid");
 // Nonzero counters and a first error neither admit nor reject Host readiness.
 Ok(d.clone())
}
fn accepted_cold_bytes(handoff:&[u8],integrity:&[u8],payload:&[u8])->Result<Value>{
 ensure!(handoff.len()==6067&&digest(handoff)==HANDOFF_SHA,"accepted status IO handoff changed");
 ensure!(integrity.len()==5120&&digest(integrity)==LAUNCH_SHA,"accepted status IO launch integrity changed");
 ensure!(payload.len()==24434&&digest(payload)==MANIFEST_SHA,"accepted status IO payload manifest changed");
 let h:Value=serde_json::from_slice(handoff)?;
 ensure!(h["bundle_root"].as_str().map(Path::new)==Some(Path::new(BUNDLE))&&h["candidate"]["sha256"]==ready_flow::PRODUCER_SHA,"cold status IO candidate differs");
 let m:Value=serde_json::from_slice(integrity)?;let all:Value=serde_json::from_slice(payload)?;
 let files=m["files"].as_array().context("launch pins absent")?;
 let payload_files=all["files"].as_array().context("payload pins absent")?;
 ensure!(files.len()==26&&payload_files.len()==122,"exact26 launch/122 payload pins required");
 ensure!(files.iter().all(|f|payload_files.iter().filter(|p|*p==f).count()==1),"launch pins must match payload manifest exactly");
 Ok(m)
}
'''
s=s.replace('const INITIAL:',insertion+'const INITIAL:',1)
start=s.index(' let handoff=read_bound(')
end=s.index(' for e in files',start)
s=s[:start]+''' let handoff=read_bound(&Path::new(BUNDLE).join("evidence/Modern-status-io-cold-handoff.json"),6067)?;
 let integrity=read_bound(&Path::new(BUNDLE).join("launch-integrity.json"),5120)?;
 let payload=read_bound(&Path::new(BUNDLE).join("manifest.json"),24434)?;
 let manifest=accepted_cold_bytes(&handoff,&integrity,&payload)?;
 let files=manifest["files"].as_array().context("launch pins absent")?;
'''+s[end:]
s=s.replace(' let opened=super::status_reader::read_json', ' p.status_poll_started.set(Some(Instant::now()));\n let opened=super::status_reader::read_json',1)
s=s.replace(' let s=opened.json;', ''' let mut s=opened.json;
 let reader=json!({"method":"same_opened_handle_metadata_and_bytes","modified_epoch_ms":u64::try_from(opened.modified.duration_since(UNIX_EPOCH)?.as_millis())?,"bytes":opened.length,"max_age_ms":5000});
 checked_status_header(&s,p)?;
 s["_modern_status_reader"]=reader;Ok(s)
}
fn checked_status_header(s:&Value,p:&Provider)->Result<()>{''',1)
s=s.replace('"at least60seconds remaining before last GuestReady");Ok(s)', '"at least60seconds remaining before last GuestReady");\n publication_diagnostic(s)?;Ok(())',1)
s=s.replace('lease_end_epoch_ms:c.lease_end_epoch_ms};','lease_end_epoch_ms:c.lease_end_epoch_ms,status_poll_started:std::cell::Cell::new(None)};',1)
s=s.replace(' let s=fresh_status(p)?;let sync=', ' if !claim_status_poll(&p.status_poll_started,Instant::now()){return Ok(None);}\n let s=fresh_status(p)?;let sync=',1)
s=s.replace('"status_utc":s["utc"],"host_sync":sync', '"status_utc":s["utc"],"status_publication_diagnostic":s["status_publication"],"modern_status_reader":s["_modern_status_reader"],"status_poll_minimum_interval_ms":200,"host_sync":sync',1)
s=s.replace('#[cfg(test)]mod r528_final_provider_boundaries', '#[cfg(test)]mod r528_status_io_integration {include!("r528-status-io-interface-tests.rs");}\n\n#[cfg(test)]mod r528_final_provider_boundaries',1)
text(P/'tools/r528-status-io-provider.rs',s)
assert pin(P/'tools/r528-status-reader.rs')['sha256']=='EB80740516AF815C2EAAA35E1024A2E02E281140B11F5A45CB289D898CB4207E'
shutil.copyfile(P/'tools/r528-status-reader.rs',P/'tools/r528-status-io-reader.rs')
shutil.copyfile(P/'tools/r528-final-countdown-case.rs',P/'tools/r528-status-io-countdown-case.rs')
c=(P/'tools/r528-backend-consumer.rs').read_text('utf-8')
c=c.replace('include!("r528-final-countdown-case.rs")','include!("r528-status-io-countdown-case.rs")').replace('include!("r528-provider.rs")','include!("r528-status-io-provider.rs")')
c=c.replace('const PREPARATION_SCHEMA:', 'mod status_reader {include!("r528-status-io-reader.rs");}\nconst PREPARATION_SCHEMA:',1)
c=c.replace('port:u16,lease_end_epoch_ms:u64}', 'port:u16,lease_end_epoch_ms:u64,status_poll_started:std::cell::Cell<Option<Instant>>}',1)
text(P/'tools/r528-status-io-consumer.rs',c)
text(L/'r528-status-io-source-difference.patch',''.join(difflib.unified_diff((P/'tools/r528-backend-consumer.rs').read_text('utf-8').splitlines(True),c.splitlines(True),fromfile='preserved/r528-backend-consumer.rs',tofile='new/r528-status-io-consumer.rs'))+''.join(difflib.unified_diff((P/'tools/r528-provider.rs').read_text('utf-8').splitlines(True),s.splitlines(True),fromfile='preserved/r528-provider.rs',tofile='new/r528-status-io-provider.rs')))
req=read(L/'r528-final-provider-supply-request.json')
req['accepted_cold']=pin(OUT/'accepted-status-io-root.json')
req['accepted_bundle']=str(B)
req['accepted_status_io_handoff']=pin(OUT/'accepted-status-io-handoff.json')
req['accepted_launch_integrity']=pin(OUT/'accepted-status-io-launch.json')
req['accepted_payload_manifest']=pin(OUT/'accepted-status-io-manifest.json')
req['fresh_conditions'][0]='New nw lowercase32hex UUID under exact server namespace; accepted aa561cbbe3d74e1b80f3d8858c1e576f bundle/485F candidate only; retired nw97/nw6f/nw975 refused.'
req['fresh_conditions'][3]='Verify accepted status IO handoff 884F2FE6, launch 214AF07D/26 exact launch pins, payload manifest 511B44E9/122 entries, Jar485F 8906219bytes. New status_publication schema is diagnostic only; counters/first_error never replace readiness guards.'
req['status_reader']=dict(source=pin(P/'tools/r528-status-io-reader.rs'),same_handle=True,max_bytes=65536,max_age_seconds=5,poll_start_minimum_interval_ms=200,scope='Host gate only before GuestReady; initial validation also records read start; no error retry/cache/extra sleep',E103_to_Cancel0_extra_IO_or_wait=False)
req['publisher_retry_boundary']=read(B/'evidence/Modern-status-io-cold-handoff.json')['precise_retry_boundary']
req['limits']+=read(B/'evidence/Modern-status-io-cold-handoff.json')['unknowns']
exe=P/'target/r528-status-io-backend-consumer/r528-status-io-consumer.exe'
newreq=L/'r528-status-io-provider-supply-request.json'
for key,args in req['commands'].items():
 args[0]=str(exe)
 if key=='check_prepared':args[2]=str(newreq)
save(newreq,req)
print(json.dumps(dict(status='NEW_OWN_SOURCES_PREPARED',accepted_unique_pins=len(checked),server_writes=0,production_changes=0),ensure_ascii=False))
