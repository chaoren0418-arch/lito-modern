"""Prepare a minimal retired-peer ordering fix only after current red replay."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';SRC=ROOT/'LiTo381-Modern/src/modern_lito'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('--red-run',type=Path,required=True);ap.add_argument('--red-session',type=int);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
red=a.red_run.resolve(strict=True);assert red.is_relative_to(V/'native-retired-join-order')
r=read(red/'receipt.json');report=read(red/'replay.json')
assert r['status']=='executed_unchanged_controller_shadow_replay_red_reproduced'
assert all(j['exit_code']==0 for j in r['process_history'])and report['cases']==2 and report['status']=='unchanged_current_controller_retired_D0_intercept_reproduced'
assert report['known_retired_notification_was_authorized']and report['same_room_rejoin_new_early_snapshot_was_incorrectly_removed']
assert a.terminal_chunk not in ['','pending','unknown']and(a.red_session is None or a.red_session>0)
normal=V/'native-cone-cold-normal/validation-20261018-055501-340560'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
assert not read(ROOT/'工作佇列.json').get('active_workloads')
old={n:(SRC/n).read_text('utf-8-sig')for n in ['native_login.rs','native_room.rs']};new=dict(old)
before='''impl RetiredDepartures {
    pub fn clear(&mut self) { self.room = None; }
}'''
after='''impl RetiredDepartures {
    pub fn clear(&mut self) { self.room = None; }
    pub(crate) fn consume(&mut self, room:u32, name:&str)->bool {
        self.room.as_mut().is_some_and(|(id,peers)|*id==room && peers.remove(name))
    }
}'''
assert new['native_room.rs'].count(before)==1;new['native_room.rs']=new['native_room.rs'].replace(before,after)
before='''        if let Some((id, peers)) = &mut retired.room {
            if *id == room_id && peers.remove(name) { return Ok(Departure::RetiredPeer); }
        }'''
after='''        if retired.consume(room_id,name) { return Ok(Departure::RetiredPeer); }'''
assert new['native_room.rs'].count(before)==1;new['native_room.rs']=new['native_room.rs'].replace(before,after)
before='''                    ensure!(name!=own,"self departure before room acceptance");
                    self.early_roster.depart'''
after='''                    ensure!(name!=own,"self departure before room acceptance");
                    // A known old-room peer is consumed once before it can
                    // touch the next join's room binding or early roster.
                    if self.retired_departures.consume(room,&name) { return Ok(true); }
                    self.early_roster.depart'''
assert new['native_login.rs'].count(before)==1;new['native_login.rs']=new['native_login.rs'].replace(before,after)
out=V/'native-retired-join-order'/('candidate-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
for n,s in new.items():(out/n).write_text(s,encoding='utf-8')
prior=V/'native-pending-join-departure/candidate-20261018-054304-344367';pj=read(prior/'proposal.json');assert sha(prior/'probe-addon.rs')==pj['probe_addon_sha256']
old_checks=(prior/'probe-addon.rs').read_text('utf-8-sig').replace('pub fn pending_join_departure_replay()','fn prior_pending_join_contracts()',1)
fixed=ROOT/'維護工具/candidates/native-retired-join/fixed_probe.rs';current=fixed.read_text('utf-8-sig')
needle='''    use super::native_room::{self,Packet,Departure};'''
replacement='''    let prior=prior_pending_join_contracts()?;
    ensure!(prior["cases"].as_array().is_some_and(|rows|rows.len()==14),"original pending join contracts were lost");
'''+needle
assert current.count(needle)==1;current=current.replace(needle,replacement,1)
current=current.replace('"shadow_state_only":true,"new_network_sessions":0','"prior_pending_join_cases_passed":14,"shadow_state_only":true,"new_network_sessions":0',1)
(out/'probe-addon.rs').write_text(old_checks+'\n'+current,encoding='utf-8')
(out/'integration.patch').write_text(''.join(''.join(difflib.unified_diff(old[n].splitlines(True),new[n].splitlines(True),fromfile='a/'+n,tofile='b/'+n))for n in old),encoding='utf-8')
pins={n:sha(out/n)for n in [*old,'probe-addon.rs']}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'proposal.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='prepared_unapplied_known_retired_peer_before_pending_join_fix',base_source_manifest_sha256=r['base_source_manifest_sha256'],sources={n:pins[n]for n in old},probe_addon_sha256=pins['probe-addon.rs'],before_sources={n:sha(SRC/n)for n in old},fixed_probe=dict(path=str(fixed),sha256=sha(fixed)),unchanged_controller_red_replay=str(red),red_original_session=a.red_session,red_observed_actual_exit_code=0,red_terminal_chunk=a.terminal_chunk,proposal_archive_sha256=sha(out/'proposal.zip'),normal_product_modified=False,candidate_runtime_executed=False,whole_goal_complete=False)
(out/'proposal.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(str(out))
