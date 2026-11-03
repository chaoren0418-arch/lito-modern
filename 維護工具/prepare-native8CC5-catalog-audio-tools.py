"""Use the existing real C02 probes with eight explicitly pinned candidate bodies."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具'
text=(T/'compile-native8CC5-teleport-audio.py').read_text('utf-8-sig')
# Keep the unchanged audio probes and API suffix in their original directory.
text=text.replace("out=V/'native8CC5-teleport-audio'","out=V/'native8CC5-catalog-audio'")
text=text.replace("runtime=P/'target/current/native8CC5-teleport-audio'","runtime=P/'target/current/native8CC5-catalog-audio'")
needle='for n,path in source_paths.items():(out/n).write_bytes(path.read_bytes())';assert text.count(needle)==1
text=text.replace(needle,'''overlay_candidate=T/'candidates/native8CC5-scoped-catalog-v3'
overlay_preparation=read(overlay_candidate/'preparation.json')
overlay_files=overlay_preparation['candidate_files'];assert len(overlay_files)==8
for n,pin in overlay_files.items():
    assert sha(pin['path'])==pin['sha256'];source_paths[n]=Path(pin['path'])
pins={n:dict(path=str(path),sha256=sha(path)) for n,path in source_paths.items()}
'''+needle,1)
text=text.replace("resolved=out/name if name=='native_login.rs' else", "resolved=out/name if name=='native_login.rs' or name in overlay_files else")
text=text.replace("if name=='native_login.rs':overrides+=1", "if name=='native_login.rs' or name in overlay_files:overrides+=1")
text=text.replace('assert overrides==1','assert overrides==9')
text=text.replace('source_overlays_used=False','source_overlays_used=True').replace('normal_source_bodies_unchanged=True','normal_source_bodies_unchanged=False')
text=text.replace("save(out/'diagnostic-inputs.json',identity)","identity['candidate_source_overlays']=overlay_files\nsave(out/'diagnostic-inputs.json',identity)")
text=text.replace('normal8cc5_wave_diagnostic','catalog_v3_wave_diagnostic').replace('current_normal_body_audio_diagnostics','candidate_catalog_body_audio_diagnostics')
# Every compiler boundary verifies copied candidate pins in addition to normal inputs.
text=text.replace('        guard()\n        with',"        guard()\n        for pin in pins.values():assert sha(pin['path'])==pin['sha256']\n        with",1)
(T/'compile-native8CC5-catalog-audio.py').write_text(text,encoding='utf-8',newline='\n')
text=(T/'run-native8CC5-teleport-audio.py').read_text('utf-8-sig').replace("V/'native8CC5-teleport-audio'","V/'native8CC5-catalog-audio'")
text=text.replace('current_normal_body','candidate_catalog_body').replace('source_overlays_used=False','source_overlays_used=True').replace('normal_source_bodies_unchanged=True','normal_source_bodies_unchanged=False')
needle="b=read(build/'receipt.json');assert b['status']=='compiled_candidate_catalog_body_audio_diagnostics'";assert text.count(needle)==1
text=text.replace(needle,needle+"\nassert b['source_overlays_used'] and not b['normal_source_bodies_unchanged']\nfor pin in b['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']",1)
text=text.replace("def stable():\n", "def stable():\n    for pin in b['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']\n",1)
(T/'run-native8CC5-catalog-audio.py').write_text(text,encoding='utf-8',newline='\n')
print('Prepared isolated candidate compiler and real C02 runner; normal bodies/products unchanged.')
