"""Prepare independent readers for the expanded current-source regression scope."""
from pathlib import Path
T=Path(__file__).resolve().parent
s=(T/'qualify-native-legacy-audio-normal.py').read_text('utf-8-sig').replace('native-legacy-audio-normal','native-scoped-catalog-normal').replace('legacy_audio','scoped_catalog')
s=s.replace("len(application['after_files'])==5","len(application['after_files'])==8")
s=s.replace("set(selected['groups'])=={'pending_join','wire_spatial'}","set(selected['groups'])=={'pending_join','wire_spatial','catalog','light_arrow'}")
needle="assert names==set(v['test_names'])and not ignored and len(names)==(2 if group=='pending_join'else 18)";assert s.count(needle)==1
s=s.replace(needle,"assert names==set(v['test_names']) and len(names)=={'pending_join':2,'wire_spatial':18,'catalog':13,'light_arrow':13}[group]\n    expected_ignored={v['prefix']+'actual_shared_light_arrow_full_chain_all_casters_and_rgba_match_csharp',v['prefix']+'actual_shared_four_profiles_preserve_all_assets_and_discard_failed_reader'} if group=='light_arrow' else set()\n    assert ignored==expected_ignored\n    test_job=next(j for j in c['process_history'] if j['label']==p+'/library-'+group+'/tests')\n    assert ('--include-ignored' in test_job['command'])==(group=='light_arrow')",1)
s=s.replace('assert unit_total==20','assert unit_total==46').replace('len(jobs)==61','len(jobs)==67')
(T/'qualify-native-scoped-catalog-normal.py').write_text(s,encoding='utf-8',newline='\n')
s=(T/'observe-native-legacy-audio-normal.py').read_text('utf-8-sig').replace('native-legacy-audio-normal','native-scoped-catalog-normal').replace('legacy_audio','scoped_catalog')
s=s.replace("q['selected_library_passed']==20","q['selected_library_passed']==46").replace('two selected groups per profile','four selected groups including two opted-in original-resource tests per profile')
(T/'observe-native-scoped-catalog-normal.py').write_text(s,encoding='utf-8',newline='\n')
print('Prepared profile and wrapper readers: 17 integration targets, 46 selected library cases and 67 recorded child jobs per profile; no full-library/GUI qualification.')
