"""Compose existing pre-session fixtures and a real C02 scene-retirement probe."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
TOOLS=ROOT/'維護工具'
dst=TOOLS/'candidates/native-class2-world'
dst.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
base=ROOT/'驗證紀錄/current/native77-live-fixture/20261017-190314-200399/NativeSelectorHarness.java'
s=base.read_text(encoding='utf-8-sig')
patches=[
 ('Set.of("disintegrate","disintegrate-refused","disintegrate-lawful")','Set.of("disintegrate","disintegrate-refused","disintegrate-lawful","class2-world-entry")'),
 ('(profile.equals("world-entry")||profile.equals("world-tree-disabled"))&&method.getName().equals("learnedSkills")','(profile.equals("world-entry")||profile.equals("world-tree-disabled")||profile.equals("class2-world-entry"))&&method.getName().equals("learnedSkills")'),
 ('(profile.equals("world-entry")?"classic.world.compatibility.enabled=true','((profile.equals("world-entry")||profile.equals("class2-world-entry"))?"classic.world.compatibility.enabled=true'),
 ('            boolean knows=accounts.learnedSkills(row.objectId()).contains(77);','            boolean knows=accounts.learnedSkills(row.objectId()).contains(77);\n            if(profile.equals("class2-world-entry")&&!accounts.learnedSkills(row.objectId()).contains(131))throw new IllegalStateException("initial world tree book absent");')]
for old,new in patches:
 assert s.count(old)==1,old
 s=s.replace(old,new)
harness=dst/'NativeSelectorHarness.java';harness.write_text(s,encoding='utf-8',newline='\n')
prep=dict(base_harness=str(base),base_harness_sha256=sha(base),candidate_harness=str(harness),candidate_harness_sha256=sha(harness),
 reference_jar_sha256='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',profiles=['class2-world-entry'],
 exact_patches=[dict(before=x,after=y) for x,y in patches],scope='Existing initial HP/MP/lawful fixture plus initial learned131 and classic WORLD configuration; no runtime authority writes',new_sessions=0)
(dst/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')

s=(TOOLS/'run-native77-fixture-preflight.py').read_text(encoding='utf-8-sig')
s=s.replace("PREP=ROOT/'維護工具/candidates/native77-live'","PREP=ROOT/'維護工具/candidates/native-class2-world'")
s=s.replace('native77-live-fixture','native-class2-world-fixture')
s=s.replace('native77-scene-integration/validation-20261017-183727-278843','native77-lawful/validation-20261017-195458-430865')
old="c=read(normal/'cohort.json');assert c['owned_process'] is not None and c['owned_process']['label'].endswith(('normal-build','lib-build','integration-build')),'do not compile beside timed product cases'"
assert old in s
s=s.replace(old,"c=read(normal/'cohort.json')\nfrom native1030_broad_gate import require_idle_runtime\nrequire_idle_runtime(ROOT)\nassert c['owned_process'] is None")
s=s.replace('fixture_profiles_passed=3','fixture_profiles_passed=1')
(TOOLS/'run-native-class2-world-fixture.py').write_text(s,encoding='utf-8',newline='\n')

source=(TOOLS/'candidates/native-audio-world-tree/probe.rs').read_text(encoding='utf-8-sig')
prefix=source.split('fn player(')[0].replace('use lito_modern::modern_lito as m;','use crate::modern_lito as m;')
prefix=prefix.replace('.filter(|row| row["prepared"] == true)', '.filter(|row| row["prepared"] == true && row["native_group"] == true)')
prefix=prefix.replace('    defer_audio: bool,','    defer_audio: bool,\n    night_generation:Option<u64>,')
prefix=prefix.replace('            defer_audio: false,','            defer_audio: true,\n            night_generation:None,')
start=prefix.index('        let s = self.c.snapshot();')
end=prefix.index('        ensure!(\n            self.view.error.is_none()',start)
prefix=prefix[:start]+'''        let epoch = self.c.transport().snapshot().epoch;
        let s=self.c.snapshot();let active=matches!(s.phase,P::BattlePending|P::WorldReady)&&s.world.ready();
        let (world,environment)=self.c.presentation_inputs();
        if active && world.map_id==Some(564) && self.night_generation!=Some(world.generation) {
            environment.accept(epoch,&[0x21,0xe0,0xc4,0,0])?;
            ensure!(environment.ambient_level==Some(6),"controlled local night level");
            self.night_generation=Some(world.generation);
        }
        self.view.tick_environment(Path::new(r"D:/lineage開發/0-客戶端"),epoch,world,environment,active);
        let s=self.c.snapshot();
'''+prefix[end:]
start=prefix.index('                let mine = ')
end=prefix.index('                if self.defer_audio',start)
prefix=prefix[:start]+'''                let mine=(row["event"]["sound"]==34 && cue.group().selected(cue.selection())==Some(34))
                    || (row["event"]["actor"]==own && row["event"]["sound"]==109 && cue.group().selected(cue.selection())==Some(109));
'''+prefix[end:]
prefix=prefix.replace('"view_quiescent":self.view.quiescent(),','"view_quiescent":self.view.quiescent(),"environment":s.environment,"lightning":self.view.frame.as_ref().map(|f|&f.receipt["call_lightning_effects"]),')
# The authored tail stays independent of the already-qualified diagnostic.
tail_path=TOOLS/'candidates/native-class2-presentation/world_probe_tail.rs'
assert tail_path.is_file(),'write the authored probe tail before preparing'
(TOOLS/'candidates/native-class2-presentation/live_probe_world.rs').write_text(prefix+tail_path.read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
print(dst)
