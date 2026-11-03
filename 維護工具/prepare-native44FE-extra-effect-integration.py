"""Stage actual lazy effect-bank integration; do not touch the frozen normal product."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
normal=ROOT/'驗證紀錄/current/native-model-generation-normal/validation-20261018-160708-195207'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
base=ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects';proof=read(base/'preparation.json')
assert proof['component_metadata_guards_verified']and not proof['applied_to_normal_product']
assert sha(base/'native_additional_finite_effects.rs')==proof['candidate_sha256']
out=ROOT/'維護工具/candidates/native44FE-extra-skill-effects-integration';out.mkdir()
origin=P/'src/modern_lito/native_skill_effects.rs';original=origin.read_bytes();s=original.decode('utf-8-sig')
def replace(old,new):
    global s
    assert s.count(old)==1,old[:120]
    s=s.replace(old,new)
replace('    selector_error: Option<String>,','    selector_error: Option<String>,\n    additional_bank: Option<SharedEffectBank>,\n    additional_error: Option<String>,')
needle='        // S37\'s explicit11722..11725 selects the reviewed TW data namespace.'
block=r'''        // Only exact additional finite definitions are admitted. Load after a
        // real cue names one; original startup work and all bounds stay intact.
        if self.profile==Profile::Skills && self.additional_bank.is_none() && self.additional_error.is_none()
            && world.effects.iter().chain(self.active.iter().map(|a|&a.cue))
                .any(|e|super::native_additional_finite_effects::GRAPHICS.contains(&e.graphics)) {
            let loaded=(||->anyhow::Result<_>{
                let context=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_with_graphics(context,cancel,
                    super::native_additional_finite_effects::qualify,
                    super::native_additional_finite_effects::GRAPHICS_BINDINGS)?;
                let mut clips=BTreeMap::new();
                for &gfx in super::native_additional_finite_effects::GRAPHICS {
                    clips.insert(gfx,Clip::new(Arc::clone(bank.animation(u32::from(gfx),0)?))?);
                }
                Ok((bank,clips))
            })();
            match loaded {
                Ok((bank,clips))=>{self.clips.extend(clips);self.additional_bank=Some(bank);},
                Err(e)=>self.additional_error=Some(format!("{e:#}")),
            }
        }
'''
replace(needle,block+needle)
needle='else if self.profile==Profile::Skills && super::native_selector_effects::GRAPHICS.contains(&cue.graphics)'
replace(needle,'else if self.profile==Profile::Skills && super::native_additional_finite_effects::GRAPHICS.contains(&cue.graphics) {self.additional_bank.as_ref()} '+needle)
replace('"load_error":self.error.as_ref().or(self.tw_heal_error.as_ref())','"load_error":self.error.as_ref().or(self.tw_heal_error.as_ref()).or(self.additional_error.as_ref())')
needle='        if let Some(bank) = &self.tw_heal_bank {'
block=r'''        if let Some(bank)=&self.additional_bank {
            receipt["additional_bank"]=serde_json::json!({"lazy":true,"loaded":true,
                "graphics":super::native_additional_finite_effects::GRAPHICS,
                "encoded_bytes":bank.encoded_bytes(),"retained_RGBA_bytes":bank.retained_rgba_bytes(),
                "sound_file_bytes":bank.sound_bank().file_bytes(),"sound_PCM_bytes":bank.sound_bank().pcm_bytes(),
                "sources":bank.sources().map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,
                    "sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>(),
                "load_error":self.additional_error});
        } else if self.additional_error.is_some() {
            receipt["additional_bank"]=serde_json::json!({"lazy":true,"loaded":false,"load_error":self.additional_error});
        }
'''
replace(needle,block+needle)
(out/'native_skill_effects.original.rs').write_bytes(original)
(out/'native_skill_effects.rs').write_text(s,encoding='utf-8',newline='\n')
(out/'native_additional_finite_effects.rs').write_bytes((base/'native_additional_finite_effects.rs').read_bytes())
module=P/'src/modern_lito/mod.rs';raw=module.read_bytes();assert b'mod native_additional_finite_effects;'not in raw
(out/'mod.original.rs').write_bytes(raw)
(out/'mod.rs').write_bytes(raw+b'\npub(crate) mod native_additional_finite_effects;\n')
record=dict(status='staged_lazy_finite_effect_bank_integration_not_compiled_not_applied',
    base_source_manifest_sha256='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0',
    base_runtime_sha256=sha(origin),base_module_sha256=sha(module),prior_component_proof=str(base/'preparation.json'),prior_component_proof_sha256=sha(base/'preparation.json'),
    files={n:sha(out/n)for n in ['native_skill_effects.rs','native_additional_finite_effects.rs','mod.rs']},
    normal_source_modified=False,applied_to_normal_product=False,typecheck_verified=False,SPR_decode_verified=False,PCM_decode_verified=False,
    new_behavior='Demand-load11 exact finite effects using the existing bounded bank/clip/audio pipeline. Original fields/history, startup bank and clocks stay intact; include new source hashes and allocation counts in receipts.',
    required_next='Compile isolated complete library after original broad cohort is terminal; verify original SPR/PCM and visible frame/audio event progression, lazy startup, malformed-source rejection and lifecycle before normal application.',
    gameplay_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
print(json.dumps(dict(candidate=str(out),finite_graphics=11,normal_source_modified=False,compiled=False,applied=False),ensure_ascii=False))
