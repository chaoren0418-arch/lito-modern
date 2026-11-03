"""Stage a separate advertised-rental destination case; keep the failed channel case intact."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
normal=V/'native-ability-finite-normal/validation-20261018-202532-027508';inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
q=read(ROOT/'工作佇列.json');assert q['active_workload_sessions']=={'broad_default':20372}and q['source_and_corresponding_QA_frozen']
source=P/'tests/native_teleport_handoff.rs';raw=source.read_bytes();s=raw.decode('utf-8-sig')
start=s.index('  c.enter_channel(0)?;');end=s.index('  std::fs::write(root.join(format!("{role}-handoff.json"))',start)
prefix=s[:start].encode('utf-8');assert raw.startswith(prefix)
old_branch=s[start:end]
replacement=r'''  ensure!(same_endpoint,"this separate probe requires the same-endpoint scenario");
  ensure!(destination.services.as_ref().is_some_and(|d|!d.rooms.is_empty()),"destination did not advertise rental service");
  ensure!(serde_json::to_value(&before.inventory.items)?==serde_json::to_value(&destination.inventory.items)?,"handoff changed an initial destination inventory field");
  // Follow the advertised service via the same public actions as initial entry.
  // The preserved failed CA00/common-room case is a different contract scope.
  let restored_set=before.rental_set.context("source rental selection absent")?;
  c.select_rental_set(restored_set)?;wait(&mut c,15,|s|s.phase==P::RentalReady)?;
  c.join_rental_room(0)?;wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&npc))?;
  let battle=c.snapshot().clone();
  ensure!(battle.world.generation>before.world.generation&&battle.world.map_id==Some(500),"fresh same-map rental world absent");
  ensure!(battle.selected_character==before.selected_character&&battle.rental_set==before.rental_set,"advertised service changed character or requested set");
  ensure!(!old_frame.input_ready(&battle.world),"old frame authorized resumed service input");
  let mut battle_model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&battle.world,&cancel)?;
  let deadline=Instant::now()+Duration::from_secs(8);
  let frame=loop {
   c.poll();ensure!(c.snapshot().phase==P::BattlePending,"resumed rental world left");
   let frame=battle_model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;
   if frame.input_ready(&c.snapshot().world)&&frame.movement_ready(&c.snapshot().world){break frame;}
   ensure!(Instant::now()<deadline,"resumed frame readiness deadline");thread::sleep(Duration::from_millis(10));
  };
  let image=root.join(format!("{role}-same-endpoint-rental.png"));
  let mut encoder=png::Encoder::new(std::fs::File::create(&image)?,800,600);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
  encoder.write_header()?.write_image_data(frame.canvas.rgba())?;
  let frame_receipt=frame.receipt.clone();drop(frame);
  let resume_steps=walk(&mut c,&mut battle_model,npc,&cancel,start)?;
  ensure!(!resume_steps.is_empty(),"resumed service movement was not exercised");
  let continued=c.snapshot().clone();
  let service_resume=json!({"kind":"advertised_rental_service","requested_set":restored_set,"image":image,"frame":frame_receipt,"steps":resume_steps,"continued":continued,
   "normal_channel_continuation_verified":false,"GUI_verified":false,"nondefault_stat_continuity_verified":false});
  std::fs::write(root.join(format!("{role}-battle-ready")),b"done")?;
  wait(&mut c,20,|_|root.join(format!("{other}-battle-ready")).exists())?;
'''
s=s[:start]+replacement+s[end:]
needle='"battle":battle,"old_frame_rejected":true';assert s.count(needle)==1;s=s.replace(needle,'"battle":battle,"service_resume":service_resume,"claimed_scope":"same_endpoint_advertised_rental_service","old_frame_rejected":true')
old_name='fn actual_two_players_teleport_and_continue_on_new_game_endpoints()';assert s.count(old_name)==1;s=s.replace(old_name,'fn actual_two_players_same_endpoint_resume_advertised_rental_service()')
old_reference='let expected=std::env::var("LITO_TELEPORT_REFERENCE_SHA").unwrap_or_else(|_|"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43".into());'
new_reference='let expected=std::env::var("LITO_TELEPORT_REFERENCE_SHA").expect("explicit current75 reference required");assert_eq!(expected.to_ascii_uppercase(),"75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371");assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("same-endpoint"));'
assert s.count(old_reference)==1;s=s.replace(old_reference,new_reference)
assert s.encode('utf-8').startswith(prefix)
out=ROOT/'維護工具/candidates/native33A3-same-endpoint-rental';out.mkdir()
(out/'probe.rs').write_text(s,encoding='utf-8',newline='\n');(out/'original-common-channel-branch.rs.txt').write_text(old_branch,encoding='utf-8',newline='\n')
contract=V/'native33A3-contract-followup/ClientSession-selected-methods.json';data=read(contract)
record=dict(status='staged_uncompiled_separate_advertised_rental_resume_case',base_source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],base_probe=str(source),base_probe_sha256=sha(source),probe_sha256=sha(out/'probe.rs'),original_prefix_preserved_bytes=len(prefix),original_prefix_sha256=hashlib.sha256(prefix).hexdigest().upper(),contract_inspection=str(contract),contract_inspection_sha256=sha(contract),reference_jar_sha256=data['reference_jar_sha256'],old_failed_case=str(V/'native44FE-same-endpoint/20261018-171204-385221-debug'),
    intended_behavior='After real same-endpoint S72/relogin and exact initial item continuity, use received rental service, select the previous set and join rental room through normal public APIs. Require fresh same-map world, old-frame rejection, native PNG and real movement.',
    old_common_channel_expectations_removed_from_original_test=False,original_failed_case_not_relabelled=True,normal_source_modified=False,normal_request_API_extension=False,runtime_overrides=False,compiled=False,tests_executed=False,actual_sessions=0,gameplay_qualified=False,GUI_verified=False,whole_goal_complete=False,
    required_next='After current broad/full-library processes finish, compile this separate test against the exact normal Cargo library, run unmodified75 same-endpoint fixture, independently decode all sessions/CA-C9-D9-ARGO room entry/authority/items/PNG/movement/owned cleanup. Keep CA00 common-channel failure separate; do not infer success from this preparation.')
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
print(json.dumps(dict(candidate=str(out),compiled=False,actual_sessions=0,normal_source_modified=False),ensure_ascii=False))
