"""Assemble a coherent candidate outside the frozen product source."""
from pathlib import Path
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
BASE=ROOT/'驗證紀錄/current/native-presentation-source-owner/validation-20261017-060440-458259'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
pins=read(BASE/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
OUT=ROOT/'維護工具/candidates/native-selector-nine-integration-reviewed'
OUT.mkdir();SOURCES=OUT/'sources';SOURCES.mkdir()
before={};after={}
def original(name):
    p=PROJECT/'src/modern_lito'/name
    before[str(p.relative_to(ROOT)).replace('\\','/')]=p.read_bytes()
    return p.read_text(encoding='utf-8-sig')
def write(name,text):
    p=SOURCES/name;p.write_text(text,encoding='utf-8',newline='\n')
    after['LiTo381-Modern/src/modern_lito/'+name]=sha(p)
def replace(s,a,b,count=1):
    assert s.count(a)==count,(a[:100],s.count(a),count)
    return s.replace(a,b)
proven=[
    ('native-selector-nine-intents','native_skills.rs'),
    ('native-selector-nine-intents','native_skill_selection.rs'),
    ('native3934-empty-sentinel','sprite.rs'),
    ('native-selector-nine-effects','native_selector_effects.rs'),
    ('native-typed-audio-delivery','native_audio_delivery.rs'),
]
derivations=[]
for folder,name in proven:
    p=ROOT/'維護工具/candidates'/folder/name
    if (PROJECT/'src/modern_lito'/name).exists():original(name)
    s=p.read_text(encoding='utf-8-sig')
    if name=='native_audio_delivery.rs':
        s=replace(s,'    fn same(&self,other:&Self)->bool {','    pub fn same_context(&self,other:&Self)->bool {')
        s=s.replace('.same(','.same_context(')
        s=replace(s,'    pub fn selection(&self)->u32 {self.0.selection}',
            '    pub fn selection(&self)->u32 {self.0.selection}\n    pub fn prepared_sound(&self)->Option<&super::combat_audio::PreparedSound> {\n        self.0.group.selected(self.0.selection).and_then(|id|self.0.bank.get(id))\n    }')
    write(name,s);derivations.append(dict(source=str(p),source_sha256=sha(p),candidate_sha256=after['LiTo381-Modern/src/modern_lito/'+name]))

# A dedicated bank policy admits only this pinned original3934's empty slots.
s=original('tower_assets.rs')
s=replace(s,'enum ReferencedFramePolicy { NonEmpty, TwHealingFamily }','enum ReferencedFramePolicy { NonEmpty, TwHealingFamily, ReviewedSelectors }')
s=replace(s,'Self::load_profile(context, cancel, qualify, bindings, false)','Self::load_profile(context, cancel, qualify, bindings, false, ReferencedFramePolicy::NonEmpty)')
s=replace(s,'Self::load_profile(context, cancel, qualify, &[], true)','Self::load_profile(context, cancel, qualify, &[], true, ReferencedFramePolicy::NonEmpty)')
anchor='    fn load_profile(\n'
s=replace(s,anchor,'''    pub(crate) fn load_reviewed_selectors(context:&SharedActionCatalog,cancel:&AtomicBool)->Result<Self> {
        Self::load_profile(context,cancel,super::native_selector_effects::qualify,
            super::native_selector_effects::GRAPHICS_BINDINGS,false,ReferencedFramePolicy::ReviewedSelectors)
    }
'''+anchor)
s=replace(s,'        directional: bool,\n    ) -> Result<Self> {','        directional: bool,\n        frame_policy: ReferencedFramePolicy,\n    ) -> Result<Self> {')
s=replace(s,'bindings, directional, ReferencedFramePolicy::NonEmpty, cancel)','bindings, directional, frame_policy, cancel)')
digest=bytes.fromhex('97183EBE2307D1A7F5D244459AE7B8434072544DC800C82D94B4B1C441F03753')
anchor='            let decoded_bytes = decoded\n'
s=replace(s,anchor,f'''            if matches!(frame_policy,ReferencedFramePolicy::ReviewedSelectors) && name=="3934-0.spr" {{
                let expected:[u8;32]={list(digest)};
                if size!=1_168_552 || Sha256::digest(&bytes).as_slice()!=expected
                    || decoded.frames.len()!=13 || [0usize,12].iter().any(|i| {{
                        let f=&decoded.frames[*i];
                        (f.x_offset,f.y_offset,f.width,f.height,f.unknown1,f.unknown2)!=(352,120,0,0,u16::MAX,0)
                            || !f.rgba.is_empty()
                    }}) {{return Err(HonorAssetError::Unsupported("selector3934 original blank slots changed"));}}
            }}
'''+anchor)
s=replace(s,'&& is_tw_healing_blank(&asset.file_name(),asset.subframe as usize,frame);','''&& is_tw_healing_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy,ReferencedFramePolicy::ReviewedSelectors)
                            && asset.file_name()=="3934-0.spr" && matches!(asset.subframe,0|12)
                            && (frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2)==(352,120,0,0,u16::MAX,0)
                            && frame.rgba.is_empty();''')
s=replace(s,'    pub(crate) fn sound_bank(&self) -> Arc<SoundBank> {','    pub(crate) fn retained_rgba_bytes(&self)->usize {self.rgba}\n    pub(crate) fn encoded_bytes(&self)->usize {self.encoded}\n    pub(crate) fn sound_bank(&self) -> Arc<SoundBank> {')
write('tower_assets.rs',s)

# Every old event producer stays explicit; finite effects attach typed payloads.
names=['native_combat_animation.rs','native_ground_effects.rs','native_spell_projectiles.rs',
    'native_world_animation.rs','native_attack_many.rs','native_class1.rs','native_wire_audio.rs']
for name in names:
    s=original(name)
    if name=='native_combat_animation.rs':
        s=replace(s,'    pub sound: Option<u32>,','    pub sound: Option<u32>,\n    #[serde(skip)]\n    pub prepared_audio: Option<super::native_audio_delivery::PreparedCue>,')
    pattern=r'(EnteredEvent\s*\{)(?!\s*\n\s*pub actor)'
    s,n=re.subn(pattern,r'\1 prepared_audio:None,',s)
    assert n==1,(name,n)
    write(name,s)
s=original('native_skill_effects.rs')
s=replace(s,'    tw_heal_bank: Option<SharedEffectBank>,','    tw_heal_bank: Option<SharedEffectBank>,\n    selector_bank: Option<SharedEffectBank>,\n    selector_error: Option<String>,')
anchor='        // S37\'s explicit11722..11725 selects the reviewed TW data namespace.'
s=replace(s,anchor,'''        // Preserve the old startup path. The nine-selector bank is demand-loaded
        // only when a real S37 names one of its exact qualified graphics IDs.
        if self.profile==Profile::Skills && self.selector_bank.is_none() && self.selector_error.is_none()
            && world.effects.iter().any(|e|super::native_selector_effects::GRAPHICS.contains(&e.graphics)) {
            let loaded=(||->anyhow::Result<_>{
                let context=SharedActionCatalog::load(root,cancel)?;
                let bank=SharedEffectBank::load_reviewed_selectors(&context,cancel)?;
                let mut clips=BTreeMap::new();
                for gfx in super::native_selector_effects::GRAPHICS {
                    clips.insert(gfx,Clip::new(Arc::clone(bank.animation(u32::from(gfx),0)?))?);
                }
                Ok((bank,clips))
            })();
            match loaded {Ok((bank,clips))=>{self.clips.extend(clips);self.selector_bank=Some(bank);},
                Err(e)=>self.selector_error=Some(format!("{e:#}")),}
        }
'''+anchor)
s=replace(s,'let bank = if matches!(cue.graphics,11722..=11725) { self.tw_heal_bank.as_ref() } else { self.bank.as_ref() }',
    'let bank = if matches!(cue.graphics,11722..=11725) { self.tw_heal_bank.as_ref() } else if self.profile==Profile::Skills && super::native_selector_effects::GRAPHICS.contains(&cue.graphics) {self.selector_bank.as_ref()} else { self.bank.as_ref() }')
start=s.index('                    for crossed in sample.crossed_cues {')
end=s.index('                    if sample.finished {',start)
s=s[:start]+'''                    for crossed in sample.crossed_cues {
                        if crossed.event_index!=0 {continue;}
                        let animation=bank.animation(u32::from(cue.graphics),self.profile.action())?;
                        let frame=&animation.frames[crossed.source_frame];
                        let classified=super::combat_animation::classify_native_cues(&frame.events)?;
                        anyhow::ensure!(classified.non_audio.is_empty(),"finite non-audio cue unsupported");
                        if let Some(group)=classified.audio {
                            // All source-reviewed finite banks have a single audio candidate.
                            // Weighted producers must supply their reviewed random-word source.
                            anyhow::ensure!(group.candidates().len()==1,"finite weighted RNG provider unavailable");
                            let sound=group.selected(0);
                            let label=if group.mode==super::combat_animation::NativeAudioMode::SuppressWhileSameIdPlaying {
                                format!("EffectSound({})",sound.unwrap_or(0))
                            } else {format!("EffectAudioOverlap({})",sound.unwrap_or(0))};
                            let prepared=super::native_audio_delivery::PreparedCue::new(bank.sound_bank(),group,0)?;
                            events.push(EnteredEvent {actor:cue.target.map(|t|t.0).unwrap_or(0),sequence:cue.sequence,
                                action:self.profile.action(),frame:crossed.source_frame,event:label,sound,
                                prepared_audio:Some(prepared)});
                        }
                    }
'''+s[end:]
anchor='        if let Some(bank) = &self.tw_heal_bank {'
s=replace(s,anchor,'''        if let Some(bank)=&self.selector_bank {
            receipt["selector_bank"]=serde_json::json!({"lazy":true,"loaded":true,
                "encoded_bytes":bank.encoded_bytes(),"retained_RGBA_bytes":bank.retained_rgba_bytes(),
                "sound_file_bytes":bank.sound_bank().file_bytes(),"sound_PCM_bytes":bank.sound_bank().pcm_bytes(),
                "load_error":self.selector_error});
        } else if self.selector_error.is_some() {
            receipt["selector_bank"]=serde_json::json!({"lazy":true,"loaded":false,"load_error":self.selector_error});
        }
'''+anchor)
write('native_skill_effects.rs',s)

# Prepared payloads avoid a second WAV read and remain separate from legacy cues.
s=original('native_world_scene.rs')
s=replace(s,'    pub sounds: Vec<super::combat_audio::PreparedSound>,','    pub sounds: Vec<super::combat_audio::PreparedSound>,\n    pub native_audio: Vec<super::native_audio_delivery::PreparedCue>,')
s=replace(s,'        let mut sounds = Vec::new();','        let mut sounds = Vec::new();\n        let mut native_audio = Vec::new();')
s=replace(s,'e.sound.is_some() && (e.actor == 0 || audible_actors.contains(&e.actor))','(e.sound.is_some() || e.prepared_audio.is_some()) && (e.actor == 0 || audible_actors.contains(&e.actor))')
s=replace(s,'        for event in audible().take(32) {','''        for event in audible().take(32) {
            if let Some(cue)=&event.prepared_audio {
                let mut row=serde_json::json!({"event":event,"prepared":true,"native_group":true,
                    "mode":format!("{:?}",cue.group().mode),"candidates":cue.group().candidates()});
                if let Some(sound)=cue.prepared_sound() {
                    row["bytes"]=serde_json::json!(sound.file_bytes());
                    row["sha256"]=serde_json::json!(sound.sha256().iter().map(|b|format!("{b:02x}")).collect::<String>());
                } else {row["silent"]=serde_json::json!(true);}
                audio_receipts.push(row);native_audio.push(cue.clone());continue;
            }''')
s=replace(s,'            sounds,\n            navigation:','            sounds,\n            native_audio,\n            navigation:')
write('native_world_scene.rs',s)

s=original('native_world_view.rs')
s=replace(s,'    deferred_sounds: Vec<super::combat_audio::PreparedSound>,','    deferred_sounds: Vec<super::combat_audio::PreparedSound>,\n    deferred_native_audio: Vec<super::native_audio_delivery::PreparedCue>,')
s=replace(s,'        self.deferred_sounds.clear();','        self.deferred_sounds.clear();\n        for cue in &self.deferred_native_audio {cue.discard();}\n        self.deferred_native_audio.clear();',2)
s=replace(s,'    pub fn quiescent(&self) -> bool {','    pub fn audio_scope(&self,connection:u64,generation:u64)->super::native_audio_delivery::Scope {\n        super::native_audio_delivery::Scope::new(connection,generation,Arc::clone(&self.publication))\n    }\n    pub fn quiescent(&self) -> bool {')
s=replace(s,'                if !current_context {','                if !current_context {\n                    if let Ok(done)=&result {for cue in &done.frame.native_audio {cue.discard();}}')
s=replace(s,'                            self.model = Some(done.model);\n                            if Some(*captured_environment) == expected_environment {','''                            self.model = Some(done.model);
                            let scope=super::native_audio_delivery::Scope::new(key.connection,key.generation,Arc::clone(publication));
                            let bound=done.frame.native_audio.iter().try_for_each(|cue|cue.bind(scope.clone()));
                            if let Err(e)=bound {self.error=Some(format!("native audio publication: {e:#}"));}
                            else if Some(*captured_environment) == expected_environment {''')
s=replace(s,'                                    let count = self.deferred_sounds.len();','                                    let count = self.deferred_sounds.len()+self.deferred_native_audio.len();')
s=replace(s,'                                    done.frame.sounds = sounds;','                                    done.frame.sounds = sounds;\n                                    let mut native=std::mem::take(&mut self.deferred_native_audio);\n                                    native.append(&mut done.frame.native_audio);done.frame.native_audio=native;')
s=replace(s,'self.deferred_sounds.len().saturating_add(done.frame.sounds.len()) > 4096','self.deferred_sounds.len().saturating_add(self.deferred_native_audio.len()).saturating_add(done.frame.sounds.len()).saturating_add(done.frame.native_audio.len()) > 4096')
s=replace(s,'                                    self.deferred_sounds.append(&mut done.frame.sounds);','                                    self.deferred_sounds.append(&mut done.frame.sounds);\n                                    self.deferred_native_audio.append(&mut done.frame.native_audio);')
write('native_world_view.rs',s)

s=original('native_login_window.rs')
s=replace(s,'    voices: Vec<super::audio_output::PcmVoice>,','    voices: Vec<super::audio_output::PcmVoice>,\n    native_voices: super::combat_audio::CombatVoicePool,\n    audio_scope: Option<super::native_audio_delivery::Scope>,')
s=replace(s,'                voices: (0..4).map(|_| Default::default()).collect(),','                voices: (0..4).map(|_| Default::default()).collect(),\n                native_voices: super::combat_audio::CombatVoicePool::new(4)?,\n                audio_scope: None,')
pattern=r'(for voice in &mut (self|ui)\.voices \{\s*let _ = voice\.stop\(\);\s*\})'
s,n=re.subn(pattern,lambda m:m[1]+f'\n                    if let Err(e)={m[2]}.native_voices.stop_all() {{{m[2]}.audio_error=Some(e.to_string());}}',s)
assert n==3,n
anchor='        if let Some(frame) = &self.world_presentation.frame {'
insert='''        if let Err(e)=self.native_voices.poll() {self.audio_error=Some(e.to_string());}
        let native_scope=self.world_presentation.audio_scope(self.controller.transport().snapshot().epoch,
            self.controller.snapshot().world.generation);
        if self.audio_scope.as_ref().is_none_or(|old|!old.same_context(&native_scope)) {
            for voice in &mut self.voices {let _=voice.stop();}
            if let Err(e)=self.native_voices.stop_all(){self.audio_error=Some(e.to_string());}
            self.audio_frame=None;self.audio_scope=Some(native_scope.clone());
        }
'''
# The window has other paint uses of frame; target only the tick's audio block.
where=s.index('        for voice in &mut self.voices {\n            if let Err(e) = voice.poll()')
pos=s.index(anchor,where);s=s[:pos]+insert+s[pos:]
end=s.index('    unsafe fn cast_skill(',pos)
prefix=s[:end];tail=s[end:]
needle='''                }
            }
        }
    }
'''
assert prefix.endswith(needle)
prefix=prefix[:-len(needle)]+'''                }
                for cue in &frame.native_audio {
                    if self.audio_error.is_none() && GetForegroundWindow()==parent {
                        match cue.submit(&native_scope,&mut self.native_voices) {
                            Err(e)=>self.audio_error=Some(e.to_string()),
                            Ok(super::native_audio_delivery::DeliveryOutcome::Unbound)=>self.audio_error=Some("音效尚未綁定場景".into()),
                            _=>{},
                        }
                    } else {cue.discard();}
                }
            }
        }
    }
'''
write('native_login_window.rs',prefix+tail)

s=original('mod.rs');s+='\npub mod native_selector_effects;\npub mod native_audio_delivery;\n';write('mod.rs',s)
with zipfile.ZipFile(OUT/'before.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,b in before.items():z.writestr(n,b)
with zipfile.ZipFile(OUT/'before.zip') as z:
    for n,b in before.items():assert z.read(n)==b
assert all(sha(ROOT/n)==h for n,h in pins.items())
metadata=dict(status='assembled_selector_nine_full_integration_pending_typecheck',base_related_run=str(BASE),
    base_source_manifest_sha256=read(BASE/'completion-observation.json')['source_manifest_sha256'],
    before={n:hashlib.sha256(b).hexdigest().upper() for n,b in before.items()},after=after,derivations=derivations,
    normal_audio_voice_capacity=8,MAX_VOICES=16,selector_bank_lazy_only_on_S37=True,
    old_twelve_effect_qualification_unchanged=True,product_source_changed=False,applied=False,
    typechecked=False,actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(OUT/'candidate.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(output=str(OUT),modified_existing_files=len(before),candidate_files=len(after),product_source_changed=False),ensure_ascii=False))
