"""Stage a limited completion reader without transferring historical scene/library claims."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';old=T/'checkpoint-native33A3-audio-cohort-completed.py';new=T/'checkpoint-native-catalog-audio-cohort-completed.py'
original=old.read_text('utf-8-sig');s=original.replace('native33A3-audio-cohort','native-catalog-audio-cohort').replace('native33A3-teleport-audio','native-catalog-teleport-audio').replace('current_native33A3_audio','current_native_catalog_audio')
start=s.index('scene_proofs={}');end=s.index("for n,h in read(run/'compiled-inputs.json')",start)
s=s[:start]+'''normal_proofs={}
for profile in ['Debug','Release']:
    build=read(Path(c['builds'][profile])/'receipt.json');path=Path(build['normal_qualification']);normal=read(path)
    assert sha(path)==build['normal_qualification_sha256'] and normal['source_manifest_sha256']==q['source_manifest_sha256']
    assert normal['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and normal['passed_per_profile']==16 and normal['recorded_owned_processes_all_terminal']
    assert not normal['full_broad_regression_verified'] and not normal['full_library_verified']
    normal_proofs[profile]=dict(path=str(path),sha256=sha(path),normal_related_passed=16,current_scene23_qualification_inferred=False)
''' +s[end:]
s=s.replace('same_World_new_Model_event_admission_regressions_verified_in_separate_scene_suite=True','same_World_new_Model_event_admission_regressions_verified_in_separate_scene_suite=False')
s=s.replace("record['separate_normal_scene23_qualifications']=scene_proofs","record['separate_normal_related16_qualifications']=normal_proofs")
start=s.index("work=f'來源33A3CCF7");end=s.index('\nfor n in before:',start)
s=s[:start]+"work=f'來源{q[\"source_manifest_sha256\"][:8]}正常本體Presentation非空音效兩版六分支原{a.session}已actual exit0及獨立核對：12案／96sessions、44個真實C1F／S20、24個舊待播cue拒絕且零交付、20個恢復後一般音效與4個heal109各一次；69組記錄程序皆終止。只授予當前本體加隔離API、診斷Player及原PNG範圍，不推廣正常EXE GUI或實體音效。正常兩版各16相關測試、控制器96及同端點租裝12sessions維持各自限定資格；本版23場景套件、廣域／library不由舊33A3移記。其他旅館／WORLD服務、正常可見性與完整G06／G01–G12仍待。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立。'"+s[end:]
s=s.replace('source_and_corresponding_QA_frozen=True','source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False')
s=s.replace("remaining='正常模組診斷六分支與非空待播音效已驗；正常GUI／實體輸入音效、同端點、普通WORLD／其他服務、當版旅館、有效不同資源資料集ABA及完整G06.3d仍待。'","remaining='當前本體診斷六分支與非空待播音效已驗；同端點租裝另有當版限定證據。正常GUI／實體輸入音效、普通WORLD／其他服務、旅館、有效不同資源資料集ABA及完整G06.3d仍待。'")
s=s.replace("next_action='Current-source same-endpoint, hotel, WORLD/other services, required broad/full-library and normal visibility; physical input/audio and valid distinct resource-source ABA remain pending.'","next_action='Current-source hotel, WORLD/other services, required scene/broad/full-library and normal visibility; same-endpoint rental has separate limited evidence; physical input/audio and valid distinct resource-source ABA remain pending.'")
ast.parse(s);assert not new.exists();out=V/'native-catalog-teleport-audio'/('completion-tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.before.py').write_text(original,encoding='utf-8',newline='\n');new.write_text(s,encoding='utf-8',newline='\n')
(out/'checkpoint.diff').write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True))),encoding='utf-8',newline='\n')
(out/'preparation.json').write_text(json.dumps(dict(tool=str(new),sha256=hashlib.sha256(s.encode()).hexdigest().upper(),current_normal_related_per_profile=16,historical_scene_and_library_qualification_transferred=False,runtime_executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),AST='passed',runtime_executed=False),ensure_ascii=False))
