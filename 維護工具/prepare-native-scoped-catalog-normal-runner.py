"""Build normal products and run unchanged integration tests plus catalog/spell regressions."""
from pathlib import Path
T=Path(__file__).resolve().parent
s=(T/'run-native-legacy-audio-normal.py').read_text('utf-8-sig').replace('native-legacy-audio-normal','native-scoped-catalog-normal').replace('legacy_audio','scoped_catalog')
s=s.replace("unit_groups={'pending_join':'modern_lito::native_login::pending_join_departure_tests::','wire_spatial':'modern_lito::wire_spatial::tests::'}","unit_groups={'pending_join':'modern_lito::native_login::pending_join_departure_tests::','wire_spatial':'modern_lito::wire_spatial::tests::','catalog':'modern_lito::shared_action_catalog::tests::','light_arrow':'modern_lito::light_arrow_effect::tests::'}")
s=s.replace("assert len(names)==(2 if group=='pending_join'else 18)","assert len(names)=={'pending_join':2,'wire_spatial':18,'catalog':13,'light_arrow':13}[group]")
needle="execute([str(image),prefix,'--list','--ignored','--format','terse'],label+'/ignored-inventory');assert not inventory(label+'/ignored-inventory')";assert s.count(needle)==1
s=s.replace(needle,"execute([str(image),prefix,'--list','--ignored','--format','terse'],label+'/ignored-inventory');ignored=inventory(label+'/ignored-inventory')\n            expected_ignored={prefix+'actual_shared_light_arrow_full_chain_all_casters_and_rgba_match_csharp',prefix+'actual_shared_four_profiles_preserve_all_assets_and_discard_failed_reader'} if group=='light_arrow' else set()\n            assert ignored==expected_ignored",1)
needle="result=counts(execute([str(image),prefix,'--test-threads=1'],label+'/tests'))";assert s.count(needle)==1
s=s.replace(needle,"result=counts(execute([str(image),prefix,'--test-threads=1']+(['--include-ignored'] if group=='light_arrow' else []),label+'/tests'))",1)
s=s.replace("env['PATH']=str(RUST)+os.pathsep+env['PATH']", "env['PATH']=str(RUST)+os.pathsep+env['PATH'];env['LITO_SHARED_RESOURCE_ROOT']=str(Path('D:/lineage開發/0-客戶端'))")
s=s.replace('two selected library groups','four selected library groups (including two opted-in original-resource tests)')
(T/'run-native-scoped-catalog-normal.py').write_text(s,encoding='utf-8',newline='\n')
print('Prepared unique-entry normal Debug/Release runner: 17 integration targets and 4 library groups; no ignored cases left unexecuted within selected groups.')
