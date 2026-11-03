"""Pin future opt-in and normal GUI checks for the label-only source revision."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
OLD='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D';NEW='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
names=['run-native-keyboard-focus-library-resource-optin.py','checkpoint-native-keyboard-focus-library-resource-optin.py',
    'prepare-native-keyboard-focus-sky-gui.py','start-native-keyboard-focus-sky-gui-case.py','checkpoint-native-keyboard-focus-sky-gui.py',
    'qualify-native-keyboard-focus-GUI-rental-entry.py','qualify-native-keyboard-focus-GUI-hotel-entry.py']
def rename(s):return s.replace('native-keyboard-focus-library','native-accessibility-labels-library').replace('native-keyboard-focus-sky-gui','native-accessibility-labels-sky-gui').replace('native-keyboard-focus-GUI','native-accessibility-labels-GUI')
staged={};origins={}
for name in names:
    original=(T/name).read_bytes();origins[name]=hashlib.sha256(original).hexdigest().upper();body=rename(original.decode('utf-8-sig').replace('\r\n','\n').replace(OLD,NEW))
    if name.startswith('qualify-'):
        anchor="    assert by_label['world-focus']['focused_element'].startswith('0 視窗')"if'rental-entry'in name else"    assert '地圖 500'in by_label['world-focus']['tree']and by_label['world-focus']['focused_element'].startswith('0 視窗')"
        assert body.count(anchor)==1
        body=body.replace(anchor,anchor+"\n    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())\n    assert 'ID: 318'in by_label['world-focus']['tree']\n    assert not any('新密碼'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())")
        if'rental-entry'in name:
            anchor="    assert '關閉背包'in by_label['inventory-ready']['tree']and f'背包（{count}）'in by_label['scene-restored']['tree']"
            assert body.count(anchor)==1;body=body.replace(anchor,anchor+"\n    assert 'ID: 318'not in by_label['inventory-ready']['tree']and 'ID: 318'in by_label['scene-restored']['tree']\n    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['scene-restored']['tree'].splitlines())")
        else:
            anchor="    assert any('席巴:'in row['tree'] and '歡迎來到阿里亞的休憩所'in row['tree'] and 'ID: 220'in row['tree']for row in dialog_observations)"
            assert body.count(anchor)==1;body=body.replace(anchor,anchor+"\n    assert all('ID: 318'not in row['tree']for row in dialog_observations)\n    assert 'ID: 318'in by_label['hotel-room-loaded']['tree']\n    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['hotel-room-loaded']['tree'].splitlines())")
        body=body.replace("record.update(GUI_operator_wait_policy=","record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True)\nrecord.update(GUI_operator_wait_policy=")
    ast.parse(body);target=rename(name);assert target!=name and not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-accessibility-labels-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,payload in staged.items():(T/n).write_bytes(payload);(out/n).write_bytes(payload)
receipt=dict(source_manifest_sha256=NEW,templates=origins,tools={n:hashlib.sha256(payload).hexdigest().upper()for n,payload in staged.items()},runtime_started=False,
    prior_GUI_results_relabelled=False,normal_GUI_qualified=False,actual_skill_name_and_visibility_required=True)
(out/'followup-tools.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),runtime_started=False),ensure_ascii=False))
