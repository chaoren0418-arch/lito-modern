"""Prepare strict completion and subsequent current-source G06 gates."""
from pathlib import Path
import ast
T=Path(__file__).resolve().parent
s=(T/'checkpoint-native-legacy-audio-normal-completed.py').read_text('utf-8-sig')
s=s.replace('native-legacy-audio-normal','native-scoped-catalog-normal').replace('legacy_audio','scoped_catalog').replace('validation-20261018-104256-823814','validation-20261018-132210-585372')
s=s.replace("len(c['process_history'])==122","len(c['process_history'])==134").replace("q['selected_library_passed']==20","q['selected_library_passed']==46")
start=s.index("repair_dir=run/'manifest-serialization-repair'");end=s.index("for n,h in read(run/'compiled-inputs.json')",start)
s=s[:start]+"assert (run/'source-inputs.json').read_bytes()==(Path(c['application'])/'applied-source-inputs.json').read_bytes()\nassert sha(run/'source-inputs.json')==c['source_manifest_sha256']\n"+s[end:]
start=s.index('    preserved_manifest_serialization_reader_failure=');end=s.index("\nsave(out/'evidence.json',record)",start)
s=s[:start].rstrip().rstrip(',')+')'+s[end:]
s=s.replace('two_selected_library_groups','four_selected_library_groups').replace('two selected library groups','four selected library groups').replace('2選定library群組','4選定library群組')
start=s.index("work=f'");end=s.index('\nfor n in before:',start)
s=s[:start]+"work=f'來源33199F4E正常兩版唯一入口建置及每版17整合目標、4選定library群組，沿原53274實際退出0；{counts}過，各0敗0忽略。來源、逐映像、原始輸出與記錄程序終止已獨立核對；同Model動作目錄與同次法術Sprite讀取器共用、七項新增回歸及兩項明確啟用的完整原資源案例已驗。診斷API與資源計時未套入。此資格僅正常建置及相關範圍；下一步當版G06控制器與Presentation／非空音效實際流程，其他廣域／完整library仍待。8CC5各項證據保留原來源，不移記。8／60秒及Release／G11預算不變；相鄰只讀、重建版與登入器產物獨立。正常GUI／實體音效、同端點及完整G01–G12未完成。'"+s[end:]
s=s.replace('current8CC5-G06-controller-and-source-runtime','current33199-G06-controller-and-source-runtime').replace('retained_pre_8CC5_current_evidence','retained_pre_scoped_catalog_current_evidence').replace('8CC5_current_normal_source_baseline_verified_limited','33199_current_normal_source_baseline_verified_limited').replace('8CC5_related_qualified','33199_related_qualified')
s=s.replace('run-native1040-G06-legacy-audio-normal.py','run-native1040-G06-scoped-catalog-normal.py')
s=s.replace("continuation=run.parent/'current-continuation.json';assert not continuation.exists()","continuation=run.parent/'current-continuation.json'\nif continuation.exists():(out/'continuation-before.json').write_bytes(continuation.read_bytes())")
ast.parse(s);(T/'checkpoint-native-scoped-catalog-normal-completed.py').write_text(s,encoding='utf-8',newline='\n')
s=(T/'run-native1040-G06-legacy-audio-normal.py').read_text('utf-8-sig').replace('native-legacy-audio-normal','native-scoped-catalog-normal').replace('legacy_audio','scoped_catalog').replace('legacy-audio-normal-related','scoped-catalog-normal-related')
s=s.replace("len(c['process_history'])==122","len(c['process_history'])==134").replace("related['selected_library_passed']==20","related['selected_library_passed']==46")
ast.parse(s);(T/'run-native1040-G06-scoped-catalog-normal.py').write_text(s,encoding='utf-8',newline='\n')
print('Prepared strict normal completion and G06 driver; neither was executed. Prerequisites remain actual terminal0, both qualified profiles and exact source identity.')
