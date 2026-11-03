"""Prepare per-Model lazy catalog reuse; no global cache or normal edits."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';T=ROOT/'維護工具'
out=T/'candidates/native8CC5-scoped-catalog';out.mkdir()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
originals={};candidates={}
def source(name):
    p=P/'src/modern_lito'/name;originals[name]=dict(path=str(p),sha256=sha(p));return p.read_text('utf-8-sig')
def put(name,body):
    p=out/name;p.write_text(body,encoding='utf-8',newline='\n');candidates[name]=dict(path=str(p),sha256=sha(p))
body=source('shared_action_catalog.rs')
needle='    pub fn root(&self) -> &Path {';assert body.count(needle)==1
helper='''    /// A lazy slot owned by one Model, dropped with that Model. No global or
    /// cross-root reuse. The original context keeps Text/IDX/PAK handles pinned.
    pub(crate) fn reuse_for_model<'a>(
        root: &Path, cancel: &AtomicBool, slot: &'a mut Option<Self>,
    ) -> Result<&'a Self, SharedCatalogError> {
        active(cancel)?;
        if let Some(context) = slot.as_ref() {
            let metadata=std::fs::symlink_metadata(root).map_err(ResourceError::from)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
            }
            #[cfg(windows)] {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
                }
            }
            let actual=root.canonicalize().map_err(ResourceError::from)?;
            if actual != context.root() {
                return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
            }
        } else {
            *slot=Some(Self::load(root,cancel)?);
        }
        active(cancel)?;
        Ok(slot.as_ref().expect("successful per-Model catalog acquisition"))
    }

'''
put('shared_action_catalog.rs',body.replace(needle,helper+needle,1))
config={
 'native_skill_effects.rs':('world','skill_effects'),
 'native_ground_effects.rs':('world','ground_effects'),
 'native_class1.rs':('w','class1_effects'),
 'native_attack_many.rs':('w','many_effects'),
 'native_spell_projectiles.rs':('world','spell_projectiles'),
}
for name,(world,field) in config.items():
    body=source(name)
    signature=re.search(r'    pub fn prepare\(&mut self,\s*root:\s*&Path,\s*'+world+r':\s*&World,\s*cancel:\s*&AtomicBool\) \{',body)
    assert signature,name
    wrapper=f'''    pub fn prepare(&mut self,root:&Path,{world}:&World,cancel:&AtomicBool) {{
        self.prepare_with_catalog(root,{world},cancel,&mut None);
    }}
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,{world}:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {{'''
    body=body[:signature.start()]+wrapper+body[signature.end():]
    if name=='native_spell_projectiles.rs':
        needle='''                let loaded=if graphics==1797 {LightArrowAssets::load_ice_arrow_effect_only(root,cancel)}
                    else{LightArrowAssets::load_class4_effect_only(root,u32::from(graphics),cancel)};'''
        assert body.count(needle)==1
        body=body.replace(needle,'''                let loaded=(|| {
                    let context=super::shared_action_catalog::SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                    LightArrowAssets::load_effect_profile_with_catalog(context,u32::from(graphics),cancel)
                })();''',1)
    else:
        body,n=re.subn(r'SharedActionCatalog::load\(root,\s*cancel\)\?',r'SharedActionCatalog::reuse_for_model(root,cancel,shared)?',body)
        assert n>=1,name
    put(name,body)
body=source('light_arrow_effect.rs');needle='    /// Worker-only: fixed Text/list.spr, exact167->219';assert body.count(needle)==1
method='''    pub(crate) fn load_effect_profile_with_catalog(context:&SharedActionCatalog,graphics:u32,cancel:&AtomicBool)->Result<Self> {
        active(cancel)?;
        Self::load_selected_profile(context,BTreeSet::new(),graphics,cancel)
    }
'''
put('light_arrow_effect.rs',body.replace(needle,method+needle,1))
body=source('native_world_scene.rs')
needle='    catalog: SpriteActionCatalog,';assert body.count(needle)==1
body=body.replace(needle,needle+'\n    effect_catalog: Option<super::shared_action_catalog::SharedActionCatalog>,',1)
needle='            catalog,\n            sprite_bank,';assert body.count(needle)==1
body=body.replace(needle,'            catalog,\n            effect_catalog: None,\n            sprite_bank,',1)
for name,(world,field) in config.items():
    pattern=r'self\.'+field+r'\.prepare\(&self.root,\s*world,\s*cancel\)'
    body,n=re.subn(pattern,'self.'+field+'.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog)',body)
    assert n==1,field
put('native_world_scene.rs',body)
instrument=T/'candidates/native8CC5-resource-open-profile'
(out/'resources.rs').write_bytes((instrument/'resources.rs').read_bytes())
(out/'profile.rs').write_bytes((instrument/'profile.rs').read_bytes())
prep=json.loads((instrument/'preparation.json').read_text('utf-8-sig'))
prep.update(status='prepared_uncompiled_per_Model_lazy_catalog_candidate',original_files=originals,candidate_files=candidates,candidate_resource=dict(path=str(out/'resources.rs'),sha256=sha(out/'resources.rs')),normal_Model_body_unchanged=False,per_Model_slot=True,global_cache=False,Model_original_catalog_and_32MiB_limit_unchanged=True,effect_catalog_8MiB_limit_unchanged=True,normal_product_source_modified=False)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
runner=(T/'run-native8CC5-resource-open-profile.py').read_text('utf-8-sig').replace('native8CC5-resource-open-profile','native8CC5-scoped-catalog')
names=['resources.rs',*candidates]
runner=runner.replace("path=out/name if name=='resources.rs' else",'path=out/name if name in '+repr(names)+' else')
runner=runner.replace("if name=='resources.rs':overrides+=1",'if name in '+repr(names)+':overrides+=1')
runner=runner.replace('assert overrides==1','assert overrides=='+str(len(names)))
needle="(out/'resources.rs').write_bytes((candidate/'resources.rs').read_bytes());";assert needle in runner
copies='\n'.join("(out/"+repr(n)+").write_bytes((candidate/"+repr(n)+").read_bytes())" for n in candidates)+'\n'
runner=runner.replace(needle,copies+needle,1).replace('normal_Model_body_unchanged=True','normal_Model_body_unchanged=False').replace('resource_open_profile8cc5','scoped_catalog8cc5')
(T/'run-native8CC5-scoped-catalog-candidate.py').write_text(runner,encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),candidate_files=len(candidates),normal_product_source_modified=False),ensure_ascii=False))
