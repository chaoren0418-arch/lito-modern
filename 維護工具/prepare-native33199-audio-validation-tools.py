"""Prepare exact-current-source audio diagnostics without running any sessions."""
from pathlib import Path
import ast,hashlib,json
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
normal=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';c=read(normal/'cohort.json');source=c['source_manifest_sha256']
assert source=='33199F4ED6584FD4D19FBF1EC69C3EC69AB6E62B27ABA64CC59FE0AD2C986ED4'
base=T/'candidates/native8CC5-teleport-audio';out=T/'candidates/native33199-teleport-audio';out.mkdir()
prep=read(base/'preparation.json')
for n,h in prep['files'].items():assert sha(base/n)==h;(out/n).write_bytes((base/n).read_bytes())
prep.update(status='prepared_uncompiled_exact_current_normal_nonempty_audio_probes',source_manifest_sha256=source,normal_cohort=str(normal),origin_preparation=dict(path=str(base/'preparation.json'),sha256=sha(base/'preparation.json')),compiled=False,executed=False,new_sessions=0)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def write(name,text):ast.parse(text);(T/name).write_text(text,encoding='utf-8',newline='\n')
s=(T/'compile-native8CC5-teleport-audio.py').read_text('utf-8-sig').replace('native8CC5-teleport-audio','native33199-teleport-audio').replace('native-legacy-audio-normal','native-scoped-catalog-normal')
s=s.replace('8CC5A560EFC60C6830658A06B3C4430593493377D433684090B4F46F3427D24E',source).replace("==(130,0,0)","==(156,0,0)").replace('normal8cc5_wave_diagnostic','normal33199_wave_diagnostic')
write('compile-native33199-teleport-audio.py',s)
s=(T/'run-native8CC5-teleport-audio.py').read_text('utf-8-sig').replace('native8CC5-teleport-audio','native33199-teleport-audio');write('run-native33199-teleport-audio.py',s)
s=(T/'observe-native8CC5-catalog-audio-compile.py').read_text('utf-8-sig').replace('native8CC5-catalog-audio','native33199-teleport-audio').replace('compiled_candidate_catalog_body_audio_diagnostics','compiled_current_normal_body_audio_diagnostics')
s=s.replace("assert r['source_overlays_used'] and not r['normal_source_bodies_unchanged']", "assert not r['source_overlays_used'] and r['normal_source_bodies_unchanged']")
s=s.replace(" and len(identity['candidate_source_overlays'])==8", " and not identity.get('candidate_source_overlays')")
needle="for n,pin in identity['candidate_source_overlays'].items():assert sha(build/n)==pin['sha256']\n";assert s.count(needle)==1;s=s.replace(needle,'',1)
s=s.replace('observed_candidate_catalog_audio_compile_completed_only','observed_current_normal_audio_compile_completed_only').replace('source_overlays_used=True,normal_source_bodies_unchanged=False,candidate_module_overlays=8','source_overlays_used=False,normal_source_bodies_unchanged=True,candidate_module_overlays=0').replace('candidate_overlays=8','candidate_overlays=0')
write('observe-native33199-teleport-audio-compile.py',s)
s=(T/'qualify-native8CC5-teleport-audio.py').read_text('utf-8-sig').replace('native8CC5-teleport-audio','native33199-teleport-audio').replace('8CC5A560EFC60C6830658A06B3C4430593493377D433684090B4F46F3427D24E',source).replace("==(130,0,0)","==(156,0,0)")
needle="max(transfers)<n<source['upper_line']";assert s.count(needle)==1;s=s.replace(needle,'max(transfers)<n',1)
needle="min(heal)<n<source['upper_line']";assert s.count(needle)==1;s=s.replace(needle,'min(heal)<n',1)
# Each destination heal follows that role's F4 and has its own actor ID.
# The peer's earlier source-wave request is not an upper bound for that heal.
write('qualify-native33199-teleport-audio.py',s)
s=(T/'run-native8CC5-audio-cohort.py').read_text('utf-8-sig').replace('native8CC5-teleport-audio','native33199-teleport-audio').replace('native8CC5-audio-cohort','native33199-audio-cohort')
needle="assert(rq['profile'],rq['scenario'])==('Release','refused') and Path(rr['compile'])==builds['Release']";assert s.count(needle)==1
s=s.replace(needle,"assert rq['profile'] in ['Debug','Release'] and rq['scenario']=='refused' and Path(rr['compile'])==builds[rq['profile']]",1)
s=s.replace("for profile in ['Release','Debug']:","for profile in ['Debug','Release']:")
write('run-native33199-audio-cohort.py',s)
print(json.dumps(dict(prepared_candidate=str(out),prepared_tools=6,source_manifest_sha256=source,compiled=False,executed=False,new_sessions=0),ensure_ascii=False))
