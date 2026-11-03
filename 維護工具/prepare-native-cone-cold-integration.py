"""Prepare a reviewable source overlay only; never mutate the frozen product."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,difflib,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';SRC=P/'src/modern_lito'
def read(p):return json.loads(p.read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for n,h in inputs.items():assert sha(ROOT/n)==h
core=ROOT/'驗證紀錄/current/native-cone-cold-core/compile-20261018-044022-260466-debug-f4edbbe9';compiled=read(core/'receipt.json')
assert compiled['status']=='compiled_isolated_cold_core_diagnostic' and compiled['compiler']['exit_code']==0
out=ROOT/'驗證紀錄/current/native-cone-cold-integration'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);sources=out/'sources';sources.mkdir()
old={};new={}
def source(name):
    text=(SRC/name).read_text('utf-8-sig');old[name]=text;return text
def one(text,before,after):
    assert text.count(before)==1,(before,text.count(before));return text.replace(before,after,1)
s=source('native_skills.rs');s=one(s,'18 | 28 | 34 | 77 =>','18 | 28 | 34 | 38 | 77 =>');s=one(s,'17|34=>6','17|34|38=>6');new['native_skills.rs']=s
s=source('native_skill_selection.rs');s=one(s,'                        77 => "#77','                        38 => "#38 冰錐術（選取目標，六格內）".into(),\n                        77 => "#77');new['native_skill_selection.rs']=s
s=source('native_world.rs');s=one(s,'|| super::native_disintegrate::supports_graphics(p.graphics)))','|| super::native_disintegrate::supports_graphics(p.graphics)\n                                || super::native_cone_cold::supports_graphics(p.graphics)))');new['native_world.rs']=s
s=source('mod.rs');assert 'pub mod native_cone_cold;' not in s;s+='\npub mod cone_cold_assets;\npub mod wire_spatial;\npub mod native_cone_cold;\n';new['mod.rs']=s
s=source('native_world_scene.rs')
s=s.replace('lightning_hurt_callbacks','finite_hurt_callbacks')
s=one(s,'target:u32,incarnation:u64,sequence:u64,projectile:u32,accepted:bool,','target:u32,incarnation:u64,sequence:u64,projectile:u32,graphics:u16,accepted:bool,')
s=one(s,'accepted:bool,dispatch_ns:u64)->Self','accepted:bool,dispatch_ns:u64,graphics:u16)->Self')
s=one(s,'Self{target:actor.id,incarnation:actor.incarnation,sequence,projectile,accepted,','Self{target:actor.id,incarnation:actor.incarnation,sequence,projectile,graphics,accepted,')
s=one(s,'    pub(crate) fn order(&self)','    pub(crate) fn graphics(&self)->u16 {self.graphics}\n    pub(crate) fn order(&self)')
s=one(s,'"projectile":self.projectile,"accepted"','"projectile":self.projectile,"graphics":self.graphics,"accepted"')
s=one(s,'call_lightning:super::native_call_lightning::History,restore_other_effects:bool','call_lightning:super::native_call_lightning::History,cone_cold:super::native_cone_cold::History,restore_other_effects:bool')
s=one(s,'self.call_lightning.merge(&other.call_lightning);','self.call_lightning.merge(&other.call_lightning);self.cone_cold.merge(&other.cone_cold);')
s=one(s,'    call_lightning_effects:super::native_call_lightning::Effects,','    call_lightning_effects:super::native_call_lightning::Effects,\n    cone_cold_effects:super::native_cone_cold::Effects,')
s=one(s,'call_lightning:self.call_lightning_effects.history(),restore_other_effects:true','call_lightning:self.call_lightning_effects.history(),cone_cold:self.cone_cold_effects.history(),restore_other_effects:true')
s=one(s,'        self.call_lightning_effects.restore_history(history.call_lightning);','        self.call_lightning_effects.restore_history(history.call_lightning);\n        self.cone_cold_effects.restore_history(history.cone_cold);')
s=one(s,'.chain(self.call_lightning_effects.oldest_active_sequence()).min()', '.chain(self.call_lightning_effects.oldest_active_sequence()).chain(self.cone_cold_effects.oldest_active_sequence()).min()')
s=one(s,'            call_lightning_effects:Default::default(),','            call_lightning_effects:Default::default(),\n            cone_cold_effects:Default::default(),')
s=one(s,'        self.call_lightning_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;','        self.call_lightning_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;\n        self.cone_cold_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;')
s=one(s,'        let native_ambient=self.call_lightning_effects.ambient_events().to_vec();','''        let cold_launches:Vec<_>=poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events)
            .filter_map(|event|super::native_cone_cold::Launch::capture(&self.cone_cold_effects,&self.effect_source_owner,world,event)).collect();
        let cold_impacts=self.cone_cold_effects.update(&self.effect_source_owner,world,cold_launches.iter(),elapsed_ns)?;
        let cold_keys:std::collections::BTreeSet<_>=cold_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(cold_impacts);
        let cold_events=self.cone_cold_effects.entered_events();
        let mut cold_impact_receipts=Vec::new();
        let native_ambient=self.call_lightning_effects.ambient_events().to_vec();''')
s=one(s,'impact.projectile,accepted,elapsed_ns));lightning_impact_receipts.push(row);}', 'impact.projectile,accepted,elapsed_ns,11736));lightning_impact_receipts.push(row);}\n            else if cold_keys.contains(&(impact.target,impact.sequence,impact.projectile)){finite_hurt_callbacks.push(HurtCallback::capture(world,actor,self.effect_source_owner.clone(),impact.sequence,impact.projectile,accepted,elapsed_ns,11738));cold_impact_receipts.push(row);}')
anchor='        let mut lightning_receipt=self.call_lightning_effects.snapshot();'
s=one(s,anchor,'''        let mut cold_receipt=self.cone_cold_effects.snapshot();
        cold_receipt["hurt_callbacks"]=serde_json::json!(cold_impact_receipts);
        let cold_draws=self.cone_cold_effects.draws(&self.effect_source_owner)?;
        for draw in &cold_draws {
            draw.submit_to(&mut effect_target,super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}})?;
        }
        cold_receipt["effect_submissions"]=serde_json::json!(cold_draws.len()*2);
'''+anchor)
s=one(s,'                .chain(lightning_events.iter())','                .chain(lightning_events.iter())\n                .chain(cold_events.iter())')
s=one(s,'        receipt["call_lightning_effects"]=lightning_receipt;','        receipt["call_lightning_effects"]=lightning_receipt;\n        receipt["cone_cold_effects"]=cold_receipt;')
new['native_world_scene.rs']=s
s=source('native_world_view.rs');s=s.replace('lightning_hurt_callbacks','finite_hurt_callbacks').replace('deferred_lightning_hurt','deferred_finite_hurt')
start=s.index('            frame.receipt["call_lightning_effects"]["hurt_callbacks"]')
end=s.index('\n        }',start)
s=s[:start]+'''            for (graphics,key) in [(11736,"call_lightning_effects"),(11738,"cone_cold_effects")] {
                frame.receipt[key]["hurt_callbacks"]=serde_json::json!(frame.finite_hurt_callbacks.iter().filter(|callback|callback.graphics()==graphics).map(|callback|callback.receipt()).collect::<Vec<_>>());
                frame.receipt[key]["deferred_hurt_callbacks"]=serde_json::json!(frame.finite_hurt_callbacks.iter().filter(|callback|callback.graphics()==graphics && callback.deferred).count());
            }'''+s[end:]
new['native_world_view.rs']=s
added={name:ROOT/'維護工具/candidates/native-cone-cold'/name for name in ['cone_cold_assets.rs','wire_spatial.rs','native_cone_cold.rs']}
for name,path in added.items():assert sha(path)==compiled['candidate_inputs'][name]['sha256'];new[name]=path.read_text('utf-8-sig')
for name,text in new.items():(sources/name).write_text(text,encoding='utf-8',newline='\n')
patch=''.join(''.join(difflib.unified_diff(old.get(n,'').splitlines(True),new[n].splitlines(True),fromfile=('a/'+n if n in old else '/dev/null'),tofile='b/'+n))for n in new)
(out/'integration.patch').write_text(patch,encoding='utf-8',newline='\n');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='prepared_unapplied_cold_scene_and_request_overlay',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],base_input_files=1035,changed_original_files={n:sha(SRC/n)for n in old},added_modules=list(added),proposed_sources={n:sha(sources/n)for n in new},files=len(new),normal_product_modified=False,request_support_registered_in_normal=False,candidate_compiled=False,candidate_runtime_executed=False,new_casts=0,new_sessions=0,whole_goal_complete=False,
    required_next=['Finish and read back original current full library before any application.','Execute already compiled frozen core scenarios; preserve first failure.','Typecheck the scene overlay in isolation; qualify normal34 regression and38 request/range/life/audio/held callbacks.','Run isolated C02 positive/refused actual ClientSession and real scene rendering in both profiles before applying normal support.','Retain exact normal-source/EXE/JAR identities and rerun appropriate normal build/related/G06 gates after applying.'])
save(out/'proposal.json',record)
pins={p.relative_to(out).as_posix():sha(p)for p in out.rglob('*') if p.is_file()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'proposal.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(proposal=str(out/'proposal.json'),files=len(new),normal_product_modified=False,new_casts=0),ensure_ascii=False))
