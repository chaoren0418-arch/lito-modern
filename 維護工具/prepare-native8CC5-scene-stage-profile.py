"""Add timing only to isolated Model preparations; keep normal sources unchanged."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';P=ROOT/'LiTo381-Modern'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
base=T/'candidates/native8CC5-resource-open-profile';out=T/'candidates/native8CC5-scene-stage-profile';out.mkdir(exist_ok=True);assert not any(out.iterdir())
original=P/'src/modern_lito/native_world_scene.rs'
body=original.read_text('utf-8-sig')
first='        self.projectiles.prepare(&self.root, cancel, world);'
last='        self.many_effects.prepare(&self.root,world,cancel);'
start=body.index(first);end=body.index(last,start)+len(last)
block=body[start:end];lines=block.splitlines();assert len(lines)==12 and all('.prepare(' in line for line in lines)
replacement=[]
for line in lines:
    name=re.search(r'self\.(\w+)\.prepare',line).group(1)
    replacement.append('        let diagnostic_stage_start=std::time::Instant::now();')
    replacement.append(line)
    replacement.append('        res::diagnostic_archive_event(serde_json::json!({"stage":"model_prepare","kind":"'+name+'","elapsed_us":diagnostic_stage_start.elapsed().as_micros(),"thread_name":std::thread::current().name()}));')
body=body[:start]+'\n'.join(replacement)+body[end:]
(out/'native_world_scene.rs').write_text(body,encoding='utf-8',newline='\n')
resource=(base/'resources.rs').read_text('utf-8-sig');assert resource.count('fn diagnostic_archive_event(')==1
resource=resource.replace('fn diagnostic_archive_event(','pub(crate) fn diagnostic_archive_event(',1)
(out/'resources.rs').write_text(resource,encoding='utf-8',newline='\n')
(out/'profile.rs').write_bytes((base/'profile.rs').read_bytes())
prep=json.loads((base/'preparation.json').read_text('utf-8-sig'))
prep.update(status='prepared_uncompiled_scene_stage_instrumentation',original_Model=dict(path=str(original),sha256=sha(original)),candidate_Model=dict(path=str(out/'native_world_scene.rs'),sha256=sha(out/'native_world_scene.rs')),candidate_resource=dict(path=str(out/'resources.rs'),sha256=sha(out/'resources.rs')),normal_Model_body_unchanged=False,instrumentation='Only monotonic timings around the original twelve prepare calls; calls, ordering, return handling and budgets unchanged.')
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
runner=(T/'run-native8CC5-resource-open-profile.py').read_text('utf-8-sig').replace('native8CC5-resource-open-profile','native8CC5-scene-stage-profile')
runner=runner.replace("path=out/name if name=='resources.rs' else", "path=out/name if name in ['resources.rs','native_world_scene.rs'] else")
runner=runner.replace("if name=='resources.rs':overrides+=1", "if name in ['resources.rs','native_world_scene.rs']:overrides+=1")
runner=runner.replace('assert overrides==1','assert overrides==2')
needle="(out/'resources.rs').write_bytes((candidate/'resources.rs').read_bytes());"
assert needle in runner
runner=runner.replace(needle,"(out/'native_world_scene.rs').write_bytes((candidate/'native_world_scene.rs').read_bytes());"+needle,1)
runner=runner.replace('normal_Model_body_unchanged=True','normal_Model_body_unchanged=False')
runner=runner.replace("modes=['a','parallel']","modes=['a']")
runner=runner.replace('resource_open_profile8cc5','scene_stage_profile8cc5')
(T/'run-native8CC5-scene-stage-profile.py').write_text(runner,encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),prepare_stages=len(lines),normal_product_modified=False),ensure_ascii=False))
