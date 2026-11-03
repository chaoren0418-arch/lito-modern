"""Retain full original wire/audio/item/PNG checks for an explicit candidate scope."""
from pathlib import Path
T=Path(__file__).resolve().parent
s=(T/'qualify-native8CC5-teleport-audio.py').read_text('utf-8-sig')
s=s.replace("current/native8CC5-teleport-audio'","current/native8CC5-catalog-audio'")
s=s.replace('current_normal_body','candidate_catalog_body')
needle="assert not b['source_overlays_used'] and not r['source_overlays_used'] and b['normal_source_bodies_unchanged'] and r['normal_source_bodies_unchanged']";assert s.count(needle)==1
s=s.replace(needle,"assert b['source_overlays_used'] and r['source_overlays_used'] and not b['normal_source_bodies_unchanged'] and not r['normal_source_bodies_unchanged']\nassert r['scenario']=='refused', 'This reader is qualified only for the refused candidate preflight'\noverlays=read(build/'diagnostic-inputs.json')['candidate_source_overlays'];assert len(overlays)==8\nfor n,pin in overlays.items():assert sha(build/n)==sha(pin['path'])==pin['sha256']",1)
s=s.replace("resolved=build/name if name=='native_login.rs' else", "resolved=build/name if name=='native_login.rs' or name in overlays else")
s=s.replace("if name=='native_login.rs':overrides+=1", "if name=='native_login.rs' or name in overlays:overrides+=1")
s=s.replace('assert overrides==1 and','assert overrides==9 and')
s=s.replace('source_overlays_used=False,normal_source_bodies_unchanged=True','source_overlays_used=True,normal_source_bodies_unchanged=False')
s=s.replace('independently_qualified_current_normal_bodies_nonempty_audio_branch','independently_qualified_catalog_candidate_nonempty_audio_refused_preflight')
(T/'qualify-native8CC5-catalog-audio.py').write_text(s,encoding='utf-8',newline='\n')
print('Prepared candidate-only reader; all original refused wire, audio, item, PNG and process checks retained.')
