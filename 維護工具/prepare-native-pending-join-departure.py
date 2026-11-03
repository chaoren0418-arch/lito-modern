"""Prepare only a bounded peer-departure ordering overlay, leaving normals intact."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];SRC=ROOT/'LiTo381-Modern/src/modern_lito'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
red=ROOT/'驗證紀錄/current/native-pending-join-departure/20261018-054029-810877'
assert read(red/'receipt.json')['status']=='executed_unchanged_controller_shadow_replay_red_reproduced'
assert read(red/'replay.json')['original_rejection_atomic']
out=red.parent/('candidate-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
old={n:(SRC/n).read_text('utf-8-sig')for n in ['native_login.rs','native_room.rs']};new=dict(old)
needle='    pub fn take(&mut self,epoch:u64,room:u32)->Result<Vec<Member>> {'
replacement='''    /// A peer broadcast may precede our CE. It can only retire a snapshot
    /// within this requested join; it cannot create or accept room membership.
    pub fn depart(&mut self,epoch:u64,requested:Option<u32>,room:u32,name:&str)->Result<()> {
        ensure!(room!=0 && requested.is_none_or(|id|id==room),"early departure for another requested room");
        ensure!(self.binding.is_none_or(|b|b==(epoch,room)),"early departure crossed room or connection");
        ensure!(requested.is_some() || self.binding==Some((epoch,room)),"early departure without a bound pending room");
        encode_text(name,27)?;
        self.members.retain(|m|m.name!=name);
        self.binding=Some((epoch,room));Ok(())
    }
'''+needle
assert new['native_room.rs'].count(needle)==1;new['native_room.rs']=new['native_room.rs'].replace(needle,replacement)
needle='''                let own = self.state.selected_character.as_deref().context("departure without character")?;
                if name == own {'''
replacement='''                let own = self.state.selected_character.as_deref().context("departure without character")?;
                if self.state.joined_room.is_none() && matches!(self.state.phase,LoginPhase::RoomJoining|LoginPhase::RoomCreating) {
                    ensure!(name!=own,"self departure before room acceptance");
                    self.early_roster.depart(self.transport.snapshot().epoch,self.requested_room,room,&name)?;
                    return Ok(true);
                }
                if name == own {'''
assert new['native_login.rs'].count(needle)==1;new['native_login.rs']=new['native_login.rs'].replace(needle,replacement)
for n,s in new.items():(out/n).write_text(s,encoding='utf-8')
(out/'probe-addon.rs').write_bytes((ROOT/'維護工具/candidates/native-pending-join/probe-addon.rs').read_bytes())
(out/'integration.patch').write_text(''.join(''.join(difflib.unified_diff(old[n].splitlines(True),new[n].splitlines(True),fromfile='a/'+n,tofile='b/'+n))for n in old),encoding='utf-8')
pins={n:sha(out/n)for n in [*old,'probe-addon.rs']}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'proposal.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='prepared_unapplied_pending_join_peer_departure_ordering_fix',base_source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],sources={n:pins[n]for n in old},probe_addon_sha256=pins['probe-addon.rs'],before_sources={n:sha(SRC/n)for n in old},unchanged_controller_red_replay=str(red),red_original_session=4295,red_observed_actual_exit_code=0,red_terminal_chunk='68f1e0',proposal_archive_sha256=sha(out/'proposal.zip'),normal_product_modified=False,candidate_runtime_executed=False,whole_goal_complete=False)
(out/'proposal.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(str(out))
