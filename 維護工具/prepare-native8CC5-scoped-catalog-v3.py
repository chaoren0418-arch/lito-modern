"""Share a temporary Sprite reader across one spell preparation call."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具'
base=T/'candidates/native8CC5-scoped-catalog-v2';out=T/'candidates/native8CC5-scoped-catalog-v3';out.mkdir()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
prep=json.loads((base/'preparation.json').read_text('utf-8-sig'))
for name,pin in prep['candidate_files'].items():
    assert sha(pin['path'])==pin['sha256'];(out/name).write_bytes(Path(pin['path']).read_bytes())
body=(out/'light_arrow_effect.rs').read_text('utf-8-sig')
assert body.count('struct SpriteReader {')==1;body=body.replace('struct SpriteReader {','pub(crate) struct SpriteReader {',1)
needle='    fn load_selected_profile(\n';assert body.count(needle)==1
method='''    /// Shared only during a single prepare call; no handles reach the UI.
    pub(crate) fn load_effect_profile_with_reader(context:&SharedActionCatalog,graphics:u32,cancel:&AtomicBool,slot:&mut Option<SpriteReader>)->Result<Self> {
        let result=Self::load_selected_profile_using_reader(context,BTreeSet::new(),graphics,cancel,slot);
        // Do not reuse partial state after any failed qualification/read/decode.
        if result.is_err() {*slot=None;}
        result
    }
'''
body=body.replace(needle,method+needle,1)
needle='''    fn load_selected_profile(
        context:&SharedActionCatalog, wanted:BTreeSet<BaseAppearance>, graphics:u32, cancel:&AtomicBool,
    )->Result<Self> {''';assert body.count(needle)==1
replacement=needle+'''
        Self::load_selected_profile_using_reader(context,wanted,graphics,cancel,&mut None)
    }
    fn load_selected_profile_using_reader(
        context:&SharedActionCatalog,wanted:BTreeSet<BaseAppearance>,graphics:u32,cancel:&AtomicBool,
        reader_slot:&mut Option<SpriteReader>,
    )->Result<Self> {'''
body=body.replace(needle,replacement,1)
needle='        let mut reader = SpriteReader::open(root, cancel)?;';assert body.count(needle)==1
body=body.replace(needle,'''        if reader_slot.is_none() {*reader_slot=Some(SpriteReader::open(root,cancel)?);}
        let reader=reader_slot.as_mut().expect("successful scoped Sprite reader");
        if reader.root != root {
            return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
        }''',1)
assert body.count('        drop(reader);')==1;body=body.replace('        drop(reader);','',1)
(out/'light_arrow_effect.rs').write_text(body,encoding='utf-8',newline='\n')
body=(out/'native_spell_projectiles.rs').read_text('utf-8-sig')
needle='        for (skill,graphics) in [(4,167),(6,1797),(7,1799),(15,1583)] {';assert body.count(needle)==1
body=body.replace(needle,'        let mut sprite_reader=None;\n'+needle,1)
needle='LightArrowAssets::load_effect_profile_with_catalog(context,u32::from(graphics),cancel)';assert body.count(needle)==1
body=body.replace(needle,'LightArrowAssets::load_effect_profile_with_reader(context,u32::from(graphics),cancel,&mut sprite_reader)',1)
(out/'native_spell_projectiles.rs').write_text(body,encoding='utf-8',newline='\n')
for name in ['resources.rs','profile.rs']:(out/name).write_bytes((base/name).read_bytes())
prep.update(status='prepared_uncompiled_scoped_catalog_and_spell_Sprite_reader_reuse',candidate_files={n:dict(path=str(out/n),sha256=sha(out/n)) for n in prep['candidate_files']},candidate_resource=dict(path=str(out/'resources.rs'),sha256=sha(out/'resources.rs')),spell_reader_scope='one prepare call; dropped on method return; discarded on failed profile load',global_cache=False,normal_product_source_modified=False)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
runner=(T/'run-native8CC5-scoped-catalog-v2.py').read_text('utf-8-sig').replace('native8CC5-scoped-catalog-v2','native8CC5-scoped-catalog-v3').replace('scoped_catalog_v2_8cc5','scoped_catalog_v3_8cc5')
(T/'run-native8CC5-scoped-catalog-v3.py').write_text(runner,encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),normal_product_source_modified=False,reader_lifetime='single_prepare_call'),ensure_ascii=False))
