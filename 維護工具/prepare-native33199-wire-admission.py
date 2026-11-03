"""Prepare source-independent S4A admission ownership; never alter frozen product."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';T=ROOT/'維護工具'
out=T/'candidates/native33199-wire-admission';out.mkdir()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
originals={};files={}
def get(name):
    p=P/'src/modern_lito'/name;originals[name]=dict(path=str(p),sha256=sha(p));return p.read_text('utf-8-sig')
def put(name,text):
    p=out/name;p.write_text(text,encoding='utf-8',newline='\n');files[name]=dict(path=str(p),sha256=sha(p))
s=get('native_wire_audio.rs');assert s.count('#[derive(Default)]')==1;s=s.replace('#[derive(Default)]','#[derive(Clone, Default)]',1)
needle='impl Cursor {\n';assert s.count(needle)==1
s=s.replace(needle,needle+'''    /// Submission consumes the captured ring even if that scene is retired
    /// before its worker can publish. No sound or authority is changed here.
    pub(crate) fn admit(&mut self,world:&World) {
        if self.generation!=Some(world.generation) {
            self.generation=Some(world.generation);self.sequence=0;
        }
        self.sequence=self.sequence.max(world.wire_sound_sequence);
    }
''',1)
needle='        self.sequence=world.wire_sound_sequence;';assert s.count(needle)==1;s=s.replace(needle,'        self.admit(world);',1)
put('native_wire_audio.rs',s)
s=get('native_world_scene.rs')
needle='water_history,None)';assert s.count(needle)==1;s=s.replace(needle,'water_history,None,None)',1)
needle='        finite_history:Option<FiniteHistory>,\n';assert s.count(needle)==1;s=s.replace(needle,needle+'        wire_audio:Option<super::native_wire_audio::Cursor>,\n',1)
needle='                    let frame = model.draw_at_labels_environment(';assert s.count(needle)==1
s=s.replace(needle,'                    if let Some(cursor)=wire_audio {model.wire_audio=cursor;}\n'+needle,1)
needle='                next.lighting=std::mem::take(&mut self.lighting);';assert s.count(needle)==1
s=s.replace(needle,needle+'\n                next.wire_audio=self.wire_audio.clone();',1)
put('native_world_scene.rs',s)
s=get('native_world_view.rs')
needle='    finite_audio_history:super::native_audio_delivery::FiniteDeliveryHistory,';assert s.count(needle)==1
s=s.replace(needle,needle+'\n    ordinary_wire_cursor:super::native_wire_audio::Cursor,',1)
needle='                self.finite_visual_history=None;';assert s.count(needle)==1
s=s.replace(needle,needle+'\n                self.ordinary_wire_cursor=Default::default();',1)
needle='                    finite_history,\n';assert s.count(needle)==1
s=s.replace(needle,needle+'                    Some(self.ordinary_wire_cursor.clone()),\n',1)
pattern=r'(?m)^(\s*)Ok\(job\) => self\.job = Some\((PendingRender\{[^\n]+\})\),$'
s,n=re.subn(pattern,lambda m:m[1]+'Ok(job) => {self.ordinary_wire_cursor.admit(world);self.job=Some('+m[2]+');},',s);assert n==1
put('native_world_view.rs',s)
record=dict(status='prepared_uncompiled_S4A_submission_cursor_candidate',source_manifest_sha256='33199F4ED6584FD4D19FBF1EC69C3EC69AB6E62B27ABA64CC59FE0AD2C986ED4',original_files=originals,candidate_files=files,normal_product_source_modified=False,global_cache=False,authority_mutated=False,acceptance_8s_changed=False,acceptance_60s_changed=False,scope='S4A cursor survives same-World Model recreation and cancelled submitted jobs; new connection/generation/map resets through the existing Presentation key',remaining='Ordinary actor action playback and other finite visual history across Model/source ABA are not qualified by this S4A-specific change.',compiled=False,executed=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),changed_candidate_modules=3,normal_product_source_modified=False,compiled=False),ensure_ascii=False))
