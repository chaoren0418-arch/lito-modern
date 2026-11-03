"""Stage a public-API nondefault-stat probe; no compiler, peer, SQL or GUI here."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,difflib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
SOURCE='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
assert read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256']==SOURCE
prepared=V/'native-installed-resource-profile-nondefault-character-fixture/prepared-20261019-202442-644400'
fixture=read(prepared/'prepared.json');assert fixture['source_manifest_sha256']==SOURCE
for n,h in fixture['fixture_sources'].items():assert sha(prepared/n)==h
origin=T/'candidates/native-rental-service-same-endpoint/probe.rs';old=origin.read_text('utf-8-sig');s=old
helpers='''fn expected_initial_stats(account:&str)->Value {
 match account {
  "PeerFixture"=>json!({"maximum_hp":137,"maximum_mp":83,"armor_class":-23,"strength":19,"constitution":17,"dexterity":13,"charisma":9,"intelligence":15,"wisdom":11}),
  "PeerFixture2"=>json!({"maximum_hp":111,"maximum_mp":231,"armor_class":-7,"strength":8,"constitution":12,"dexterity":14,"charisma":10,"intelligence":21,"wisdom":19}),
  _=>panic!("unknown owned fixture account"),
 }
}
fn directory_stats(s:&Snapshot)->Result<Value> {
 ensure!(s.characters.len()==1,"nondefault fixture needs exactly one source character");
 let f=&s.characters[0].fields;let a=&f.abilities;
 Ok(json!({"maximum_hp":f.max_hp,"maximum_mp":f.max_mp,"armor_class":f.armor_class,"strength":a.strength,"constitution":a.constitution,"dexterity":a.dexterity,"charisma":a.charisma,"intelligence":a.intelligence,"wisdom":a.wisdom}))
}
fn world_stats(s:&Snapshot)->Result<Value> {
 let f=s.world.status.as_ref().context("complete source-owned status receipt required")?;
 Ok(json!({"maximum_hp":f.maximum_hp,"maximum_mp":f.maximum_mp,"armor_class":f.armor_class,"strength":f.strength,"constitution":f.constitution,"dexterity":f.dexterity,"charisma":f.charisma,"intelligence":f.intelligence,"wisdom":f.wisdom}))
}
'''
assert s.count('fn wait(c:')==1;s=s.replace('fn wait(c:',helpers+'fn wait(c:')
needle='  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;'
assert s.count(needle)==1
s=s.replace(needle,'''  let initial=c.snapshot().clone();let expected_initial=expected_initial_stats(account);
  ensure!(directory_stats(&initial)?==expected_initial,"nondefault initial directory was replaced or decoded incorrectly");
  ensure!(initial.selected_source.as_deref()==Some("測試乙"),"nondefault source mismatch");
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected&&s.services.as_ref().is_some_and(|d|!d.rooms.is_empty()))?;c.select_rental_set(1)?;''')
needle='s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987996)';assert s.count(needle)==1
s=s.replace(needle,needle+'&&s.world.status.is_some()&&s.inventory.items.len()==if role=="a"{45}else{43}')
needle='if s.phase==P::CharacterSelected&&s.inventory.batches>0{break}';assert s.count(needle)==1
s=s.replace(needle,'if s.phase==P::CharacterSelected&&s.inventory.batches>0&&s.services.as_ref().is_some_and(|d|!d.rooms.is_empty()){break}')
needle='  let destination=c.snapshot().clone();let new_epoch=c.transport().snapshot().epoch;';assert s.count(needle)==1
s=s.replace(needle,needle+'''
  ensure!(directory_stats(&destination)?==expected_initial,"destination lost a nondefault initial directory field");
  let before_effective_stats=world_stats(&before)?;''')
needle='s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&npc)';assert s.count(needle)==1
s=s.replace(needle,needle+'&&s.world.status.is_some()&&s.inventory.items.len()==if role=="a"{45}else{43}')
needle='  let battle=c.snapshot().clone();';assert s.count(needle)==1
s=s.replace(needle,needle+'''
  ensure!(world_stats(&battle)?==before_effective_stats,"advertised service changed an effective stat after handoff");''')
needle='  let continued=c.snapshot().clone();';assert s.count(needle)==1
s=s.replace(needle,needle+'''
  ensure!(world_stats(&continued)?==before_effective_stats,"continued movement changed effective stat projection");
  let nondefault=json!({"expected_initial":expected_initial,"initial_directory":directory_stats(&initial)?,"destination_directory":directory_stats(&destination)?,"before_effective":before_effective_stats,"battle_effective":world_stats(&battle)?,"continued_effective":world_stats(&continued)?,"wire_numeric_fields_per_projection":9,"initial_server_original_fields":5,"original_five_received_on_wire_verified":false,"GUI_verified":false});''')
needle='"role":role,"action":action,"steps":steps';assert s.count(needle)==1
s=s.replace(needle,'"role":role,"initial":initial,"nondefault":nondefault,"action":action,"steps":steps')
s=s.replace('"claimed_scope":"same_endpoint_advertised_rental_service"','"claimed_scope":"nondefault_initial_directory_and_effective_stat_continuity_same_endpoint"')
s=s.replace('fn actual_two_players_same_endpoint_resume_advertised_rental_service()', 'fn actual_two_players_nondefault_stats_same_endpoint_resume_advertised_rental_service()')
needle=' let peer:SocketAddr=format!';assert s.count(needle)==1
prefix=''' let initial_source:Value=serde_json::from_slice(&std::fs::read(root.join("initial-source-characters.json")).expect("explicit nondefault fixture required")).unwrap();
 for (account,wanted) in [
  ("PeerFixture",json!({"maxHp":137,"maxMp":83,"armorClass":-23,"strength":19,"constitution":17,"dexterity":13,"charisma":9,"intelligence":15,"wisdom":11,"originalStrength":18,"originalConstitution":16,"originalDexterity":12,"originalIntelligence":14,"originalWisdom":10})),
  ("PeerFixture2",json!({"maxHp":111,"maxMp":231,"armorClass":-7,"strength":8,"constitution":12,"dexterity":14,"charisma":10,"intelligence":21,"wisdom":19,"originalStrength":7,"originalConstitution":11,"originalDexterity":13,"originalIntelligence":20,"originalWisdom":18}))] {
  let rows=initial_source[account].as_array().expect("initial account rows");assert_eq!(rows.len(),1);
  for (field,value) in wanted.as_object().unwrap(){assert_eq!(rows[0].get(field.as_str()),Some(value),"initial server field {account}/{field}");}
 }
'''
s=s.replace(needle,prefix+needle)
out=T/'candidates'/('native-nondefault-same-endpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'probe.rs').write_text(s,encoding='utf-8',newline='\n');(out/'change.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),s.splitlines(True),fromfile=str(origin),tofile='probe.rs')),encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='prepared_public_API_nondefault_probe_not_compiled',source_manifest_sha256=SOURCE,
    original_probe=str(origin),original_probe_sha256=sha(origin),probe_sha256=sha(out/'probe.rs'),
    prepared_fixture=str(prepared),prepared_fixture_sha256=sha(prepared/'prepared.json'),fixture_sources=fixture['fixture_sources'],
    initial_server_fields=14,wire_stat_projection_fields=9,original_five_wire_claimed=False,
    retained_assertions='Full initial-to-destination inventory equality, epoch/peer identity, character, retired old scene/frame, replay rejection, fresh same-map generation, authoritative CPU Model movement, own worker cleanup.',
    readiness_refinements='Wait for advertised services, complete status and expected inventory; no gameplay clock changes.',
    original_prefix_byte_identical=False,production_inputs_modified=False,compiled=False,runtime_started=False,
    actual_stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False)
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(candidate=str(out),compiled=False,runtime_started=False,production_inputs_modified=False),ensure_ascii=False))
