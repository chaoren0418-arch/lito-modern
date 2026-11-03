"""Declare an owned-server EOF negative GUI case before any fault injection."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert w['current_full_default_library_verified']and w['current_native_catalog_library']['source_manifest_sha256']==source
assert c['scenario']=='rental-entry'and c['status']=='normal_windows_running_for_Sky_inputs'
assert w['active_workload_sessions'].keys()=={'normal_gui'}and not(case/'Sky-events.json').exists()
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
plan=dict(status='declared_before_owned_EOF_GUI_inputs',source_manifest_sha256=source,case=str(case),profile=c['profile'],scenario='rental-entry-with-intentional-owned-server-EOF',positive_scenario_qualification_requested=False,
    actor='a',account='PeerFixture',source='測試乙',rental_set=2,expected_initial_formal_sessions=2,unused_actor_b_must_not_connect=True,
    steps=['Normal owned GUI LOGIN/GAME, original source confirmation, select character and loadout2, enter map500','Observe actual skills and bag45, open inventory and observe real items, close bag and observe world again','Record exact server image, command, PID, birth and retained process handle; stop only this owned Java child','Pure fresh UIA EOF observation then another pure read after at least 0.25 seconds','Verify connection ended, world/skills/inventory/services/room members and selected character no longer exposed; ordinary action controls disabled','Both GUI processes close normally; retain expected wrapper1 caused by intentional server stop; independently seal negative-only scope'],
    expected_pipeline_exit_code=1,expected_server_exit_code=4294967295,expected_GUI_exit_codes=[0,0],cleanup_kill_of_GUI_forbidden=True,
    required_labels=['eof-world-before','eof-inventory-before','eof-world-restored-before','eof-after','eof-after-stable'],
    no_post_fault_authentication_expected=True,gameplay_or_protocol_clocks_modified=False,formal_JAR_modified=False,authority_injections=False,pixel_qualification_claimed=False,GUI_full_flow_verified=False,whole_goal_complete=False,declared_utc=datetime.now(timezone.utc).isoformat())
target=case/'EOF-negative-plan.json';assert not target.exists();target.write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(case/'EOF-plan-preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(target),sha256=sha(target),negative_scope_only=True),ensure_ascii=False))
