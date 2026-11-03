"""Reuse the renderer's checked catalog while preserving its 32 MiB bound."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';P=ROOT/'LiTo381-Modern'
base=T/'candidates/native8CC5-scoped-catalog';out=T/'candidates/native8CC5-scoped-catalog-v2';out.mkdir()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
prep=json.loads((base/'preparation.json').read_text('utf-8-sig'))
for name,pin in prep['candidate_files'].items():
    assert sha(pin['path'])==pin['sha256'];(out/name).write_bytes(Path(pin['path']).read_bytes())
body=(out/'shared_action_catalog.rs').read_text('utf-8-sig')
needle='    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self, SharedCatalogError> {';assert body.count(needle)==1
new='''    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self, SharedCatalogError> {
        Self::load_with_byte_limit(root,cancel,MAX_BYTES)
    }
    /// Renderer acceptance retains the original ActionLimits byte bound.
    pub(crate) fn load_for_model(root:&Path,cancel:&AtomicBool)->Result<Self,SharedCatalogError> {
        Self::load_with_byte_limit(root,cancel,ActionLimits::default().max_bytes)
    }
    fn load_with_byte_limit(root:&Path,cancel:&AtomicBool,max_bytes:usize)->Result<Self,SharedCatalogError> {'''
body=body.replace(needle,new,1)
body=body.replace('expected > MAX_BYTES','expected > max_bytes',1).replace('max_bytes: MAX_BYTES,','max_bytes,',1)
needle='        Ok(slot.as_ref().expect("successful per-Model catalog acquisition"))';assert body.count(needle)==1
body=body.replace(needle,'''        let context=slot.as_ref().expect("successful per-Model catalog acquisition");
        // Effects retain their narrower existing bound, even when the renderer
        // accepted a larger catalog. Never widen an effect's admissible input.
        if context.byte_length()==0 || context.byte_length()>MAX_BYTES {
            return Err(SharedCatalogError::Bound("action catalog bytes"));
        }
        Ok(context)''',1)
(out/'shared_action_catalog.rs').write_text(body,encoding='utf-8',newline='\n')
body=(out/'native_world_scene.rs').read_text('utf-8-sig')
needle='    catalog: SpriteActionCatalog,\n';assert body.count(needle)==1;body=body.replace(needle,'',1)
needle='''        let text =
            res::SharedArchive::open(root, res::SharedArchiveKind::Text)?.read_entry("list.spr")?;
        let catalog = SpriteActionCatalog::parse_for_inspection(&text, ActionLimits::default())?;''';assert body.count(needle)==1
body=body.replace(needle,'''        let effect_catalog=Some(super::shared_action_catalog::SharedActionCatalog::load_for_model(root,cancel)?);''',1)
needle='            catalog,\n            effect_catalog: None,';assert body.count(needle)==1;body=body.replace(needle,'            effect_catalog,',1)
body,n=__import__('re').subn(r'&self\.catalog\b',r'self.effect_catalog.as_ref().expect("Model catalog ownership").catalog()',body);assert n==4
body=body.replace('    sprite_actions::{ActionLimits, SpriteActionCatalog},\n','',1)
(out/'native_world_scene.rs').write_text(body,encoding='utf-8',newline='\n')
for name in ['resources.rs','profile.rs']:(out/name).write_bytes((base/name).read_bytes())
prep.update(status='prepared_uncompiled_per_Model_renderer_catalog_reuse',candidate_files={n:dict(path=str(out/n),sha256=sha(out/n)) for n in prep['candidate_files']},candidate_resource=dict(path=str(out/'resources.rs'),sha256=sha(out/'resources.rs')),renderer_catalog_reused=True,normal_Model_body_unchanged=False,Model_original_catalog_and_32MiB_limit_unchanged=True,effect_catalog_8MiB_limit_unchanged=True)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
runner=(T/'run-native8CC5-scoped-catalog-candidate.py').read_text('utf-8-sig').replace('native8CC5-scoped-catalog','native8CC5-scoped-catalog-v2').replace('scoped_catalog8cc5','scoped_catalog_v2_8cc5')
(T/'run-native8CC5-scoped-catalog-v2.py').write_text(runner,encoding='utf-8',newline='\n')
test=(T/'test-native8CC5-scoped-catalog-component.py').read_text('utf-8-sig').replace('native8CC5-scoped-catalog','native8CC5-scoped-catalog-v2')
(T/'test-native8CC5-scoped-catalog-v2-component.py').write_text(test,encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),normal_product_source_modified=False,renderer_limit_MiB=32,effect_limit_MiB=8),ensure_ascii=False))
