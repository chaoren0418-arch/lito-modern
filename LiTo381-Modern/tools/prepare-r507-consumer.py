from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
import shutil,difflib
o=read(L/'r506c-source-oracle.json');pe=Path(read(L/'r506-API-adoption.json')['originals'][1]['path']).read_bytes()
def vectors(bodies,counter):
 out=[]
 for body in bodies:
  b=bytes(body);enc,steps=source_cipher(b,counter,pe);after=counter_add_adc(counter,len(b));out.append(dict(plain=list(b),wire=list((len(b)+2).to_bytes(2,'little')+enc),before=counter,after=after,source_instruction_steps=steps));counter=after
 return out
tx=[o['select']['plain'],[217],[203],[202,0],[15]]
txl=[o['select']['plain'],[217],[203],[202,0],[202,2],[15]]
rx=[o['same_receive'][0]['plain'],[200,0]]
c9=o['same_receive'][2]['plain'];rxl=rx+[c9]
target=[[3,1],[203],[203],[200,0],c9]
save(L/'r507-source-oracle.json',dict(setup=o['setup'],channel_tx=vectors(tx,o['select']['before']),lobby_tx=vectors(txl,o['select']['before']),channel_rx=vectors(rx,o['setup']['first_roster']['after']),lobby_rx=vectors(rxl,o['setup']['first_roster']['after']),target_rx=vectors(target,o['setup']['receive_after_versions']),target_tx=vectors([[201,*b'Role0',0],[202,100]],o['setup']['send_vectors'][0]['after']),original_PE=read(L/'r506-API-adoption.json')['originals'][1],previous_setup_only=pin(L/'r506c-source-oracle.json'),historical_PASS_reruns=0))
helpers=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8').split('fn main(')[0].replace('r503','r507').replace('R503','R507').replace('r498-native-menu-skin-binding.json','r506-native-menu-skin-binding.json').replace('Duration::from_secs(10)','Duration::from_secs(20)')
# The unique endpoint is configured first; optional startup authority must be
# installed before OpenLogin. This never derives host from listener.local_addr.
helpers=helpers.replace('configure(r,&profile(addr,false,read_cap,write_cap,0))?;r.handle_player_entry_command(Command::OpenLogin)?;', 'configure(r,&profile(addr,false,read_cap,write_cap,0))?;r.configure_source_startup_host_fixture(lito_modern::modern_lito::native_city_runtime::startup_host::FixtureStartupHost::parse(br#"{\"kind\":\"modern_synthetic_startup_host_v1\",\"host\":\"127.0.0.2\"}"#)?)?;r.handle_player_entry_command(Command::OpenLogin)?;')
(P/'tools/r507-integration-consumer.rs').write_text(helpers+(P/'tools/r507-integration-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
root=P/'target/r507-production-source';root.mkdir();pins=[];diff=[];old=read(L/'r506c-production-source-snapshot.json');prev={q['path']:q for q in old['source_pins']};changed=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
 prior=prev.get(str(p))
 if not prior or prior['sha256']!=q['sha256']:
  changed.append(str(p));diff.append(''.join(difflib.unified_diff(Path(prior['snapshot']).read_text(encoding='utf-8').splitlines(True) if prior else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=prior['snapshot'] if prior else '/dev/null',tofile=str(p))))
patch=L/'r507-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8')
save(L/'r507-production-source-snapshot.json',dict(source_pins=pins,changed=changed,source_difference=pin(patch),sealed_R506_untouched=pin(L/'r506-native-transfer-channel-handoff.json'),GUI=0))
