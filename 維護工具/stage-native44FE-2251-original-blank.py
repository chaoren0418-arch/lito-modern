"""Admit only the byte-pinned original no-block2251 slot; ordinary profiles stay nonempty."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
out=ROOT/'維護工具/candidates/native44FE-extra-skill-effects-integration';prep=read(out/'preparation.json')
source=V/'native44FE-skill-book-audit/20261018-175440-043382/original-effect-SPR-metadata-20261018-194533-464520';proof=read(source/'readback.json');assert sha(source/'inspect.stdout')==proof['data_sha256']
row=next(r for r in read(source/'inspect.stdout')['rows']if r['graphics']==2251)
assert row['sha256']=='B90C450146296E77043090948B11075C0731FC7B15093734DC37935A114D975B'and row['bytes']==157993 and row['frame_count']==21
assert [f['index']for f in row['frames']if f['empty']]==[0]and row['frames'][0]['x']==192 and row['frames'][0]['y']==120
failed=V/'native44FE-extra-finite-effects/run-20261018-194001-834268-debug';f=read(failed/'receipt.json');assert f['status']=='failed_preserved'and [(p['label'],p['exit_code'])for p in f['process_history']]==[('scene23',0),('additional11',101)]
base=ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects/native_additional_finite_effects.rs';assert (out/base.name).read_bytes()==base.read_bytes()
(out/'preparation-before-2251-blank.json').write_bytes((out/'preparation.json').read_bytes())
extension='''
// Original2251 has one no-block slot. It retains the normal clip/audio clock.
const ORIGINAL2251_SHA:[u8;32]=[185,12,69,1,70,41,110,119,4,48,144,148,139,17,7,92,7,49,252,123,21,9,55,52,220,55,147,90,17,77,151,91];
pub fn requires_reviewed_blank(name:&str)->bool{name=="2251-0.spr"}
pub fn is_reviewed_blank(name:&str,index:usize,f:&super::sprite::SpriteFrame)->bool{
    requires_reviewed_blank(name)&&index==0
        &&(f.x_offset,f.y_offset,f.width,f.height,f.frame_type,f.unknown1,f.unknown2)==(192,120,0,0,0,u16::MAX,0)
        &&f.rgba.is_empty()&&f.fragments.as_ref().is_some_and(|p|p.references.is_empty())
}
pub fn reviewed_blank_source_matches(name:&str,raw:&[u8],decoded:&super::sprite::DecodedSprite)->bool{
    use sha2::Digest;
    requires_reviewed_blank(name)&&raw.len()==157993&&sha2::Sha256::digest(raw).as_slice()==ORIGINAL2251_SHA
        &&decoded.frames.len()==21&&decoded.frames.iter().enumerate().all(|(index,f)|{
            if index==0{is_reviewed_blank(name,index,f)}else{f.width>0&&f.height>0
                &&(f.width as usize).checked_mul(f.height as usize).and_then(|n|n.checked_mul(4))==Some(f.rgba.len())}
        })
}
'''
(out/base.name).write_bytes(base.read_bytes()+extension.encode())
normal=P/'src/modern_lito/tower_assets.rs';s=normal.read_text('utf-8-sig')
old='enum ReferencedFramePolicy { NonEmpty, TwHealingFamily, ReviewedSelectors }';assert s.count(old)==1;s=s.replace(old,old.replace('ReviewedSelectors }','ReviewedSelectors, ReviewedAdditionalFinite }'))
old='    fn load_profile(';assert s.count(old)==1;s=s.replace(old,'''    pub(crate) fn load_reviewed_additional_finite(context:&SharedActionCatalog,cancel:&AtomicBool)->Result<Self>{
        Self::load_profile(context,cancel,super::native_additional_finite_effects::qualify,
            super::native_additional_finite_effects::GRAPHICS_BINDINGS,false,ReferencedFramePolicy::ReviewedAdditionalFinite)
    }
'''+old)
old='            let decoded_bytes = decoded\n';assert s.count(old)==1;s=s.replace(old,'''            if matches!(frame_policy,ReferencedFramePolicy::ReviewedAdditionalFinite)
                && super::native_additional_finite_effects::requires_reviewed_blank(&name)
                && !super::native_additional_finite_effects::reviewed_blank_source_matches(&name,&bytes,&decoded) {
                return Err(HonorAssetError::Unsupported("original2251 no-block source changed"));
            }
'''+old)
old='                            && is_reviewed_selector_blank(&asset.file_name(),asset.subframe as usize,frame);';assert s.count(old)==1
s=s.replace(old,old[:-1]+'''
                        || matches!(frame_policy,ReferencedFramePolicy::ReviewedAdditionalFinite)
                            && super::native_additional_finite_effects::is_reviewed_blank(&asset.file_name(),asset.subframe as usize,frame);''')
(out/'tower_assets.rs').write_text(s,encoding='utf-8',newline='\n')
p=out/'native_skill_effects.rs';s=p.read_text('utf-8-sig');old='''SharedEffectBank::load_with_graphics(context,cancel,
                    super::native_additional_finite_effects::qualify,
                    super::native_additional_finite_effects::GRAPHICS_BINDINGS)?''';assert s.count(old)==1;s=s.replace(old,'SharedEffectBank::load_reviewed_additional_finite(context,cancel)?');p.write_text(s,encoding='utf-8',newline='\n')
prep.update(status='staged_exact_original2251_blank_policy_requires_rebuild',files={n:sha(out/n)for n in ['native_skill_effects.rs','native_additional_finite_effects.rs','tower_assets.rs','mod.rs']},
    base_tower_sha256=sha(normal),definition_guard_prefix_sha256=sha(base),original2251_SPR_metadata=str(source),original2251_SPR_metadata_readback_sha256=sha(source/'readback.json'),
    prior_failed_resource_run=str(failed),prior_resource_actual_exit_code=1,prior_additional11_Rust_exit_code=101,prior_unchanged_scene23_passed=23,
    reviewed_blank_only=dict(name=row['name'],bytes=row['bytes'],sha256=row['sha256'],frame_count=21,index=0,x=192,y=120,unknown1=65535,unknown2=0),
    typecheck_verified=False,SPR_decode_verified=False,PCM_decode_verified=False,resource_probe_executed=False,normal_source_modified=False,applied_to_normal_product=False)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),reviewed_blank='2251-0.spr slot0 only',normal_modified=False,rebuild_required=True),ensure_ascii=False))
