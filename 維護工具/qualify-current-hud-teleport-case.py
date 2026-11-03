from pathlib import Path
import argparse,hashlib,json,shutil,datetime
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(root/'驗證紀錄/current/native-teleport-gui')
r=read(case/'receipt.json');m=read(case/'compilation-inputs.json');published=root/'驗證紀錄/current/native-status-scroll'/(a.profile.lower()+'-build-inputs-qualified.json');expected=read(published)
assert r['inputs_unchanged'] and r['reference_is_current'] and r['complete_build_inputs_verified']
assert r['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(case/'compilation-inputs.json')==r['compilation_inputs_sha256']
assert m['status']=='completed_build' and m['profile']==a.profile and m['target']=='x86_64-pc-windows-msvc'
assert len(m['files'])==962 and m['files']==expected['files']
assert m['main_sha256']==r['exe_sha256']==expected['main']['sha256']
for n,h in expected['files'].items():assert sha(root/n)==h,n
assert sha(Path(expected['main']['path']))==expected['main']['sha256']
scenario=r['scenario'];name='teleport-gui-authority-comparison.json' if scenario=='cross-endpoint' else 'teleport-recovery-gui-comparison.json'
# Independent verifiers have different fixed output names; identify by actual status/content if needed.
if not (case/name).exists():
 candidates=[]
 for p in case.glob('*comparison.json'):
  d=read(p)
  if d.get('capture_sha256') and d.get('exe_sha256')==r['exe_sha256'] and 'glyph' not in p.name:candidates.append(p)
 assert len(candidates)==1,candidates
 comparison=candidates[0]
else:comparison=case/name
c=read(comparison);g=read(case/'teleport-glyph-comparison.json');assert c['exe_sha256']==g['exe_sha256']==r['exe_sha256'];assert c['receipt_sha256']==g['receipt_sha256']==sha(case/'receipt.json');assert g['matched_glyphs']==4 and g['role_retirements_verified']==2
expected_sessions=6 if scenario=='cross-endpoint' else 10 if scenario=='wrong-service' else 8
assert r['server']['actual_client_sessions']==expected_sessions and r['server']['authority_mutations']==0 and r['server']['memberships_empty'];assert not r['server']['sql_access'] and not r['server']['state_persistent']
verifiers=['verify-native-teleport-gui-authority.py' if scenario=='cross-endpoint' else 'verify-native-teleport-recovery-gui.py','verify-teleport-glyph-pixels.py',Path(__file__).name]
for name in verifiers:
 p=root/'維護工具'/name;dest=case/'validation-sources'/(sha(p)[:16]+'-'+name);dest.parent.mkdir(exist_ok=True)
 if dest.exists():assert sha(dest)==sha(p)
 else:shutil.copy2(p,dest)
q={'status':'qualified_current_HUD_source_teleport_GUI_case','qualified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'profile':a.profile,'scenario':scenario,'case':str(case),'main_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'source_files':962,'all_published_inputs_and_current_sources_match':True,'published_build_manifest_sha256':sha(published),'compilation_inputs_sha256':sha(case/'compilation-inputs.json'),'independent_comparison':comparison.name,'independent_comparison_sha256':sha(comparison),'glyph_comparison_sha256':sha(case/'teleport-glyph-comparison.json'),'receipt_sha256':sha(case/'receipt.json'),'actual_sessions':expected_sessions,'glyph_matches':4,'private_desktop':True,'physical_input_verified':False,'full_game_complete':False}
(case/'HUD-source-qualification.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:q[k] for k in ['status','profile','scenario','actual_sessions','source_files']}))