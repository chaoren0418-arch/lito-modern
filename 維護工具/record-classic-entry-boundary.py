import json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
run=root/'驗證紀錄/current/native-real-peers/20261013-103448-269'
receipt=json.loads((run/'receipt.json').read_text(encoding='utf-8-sig'))
frames=[]
for path in (Path(receipt['runtime'])/'captures').rglob('frames.jsonl'):
    for line in path.read_text(encoding='utf-8-sig').splitlines():
        r=json.loads(line)
        if (r['direction'],r['opcode']) in [('client_to_server','0x05'),('server_to_client','0x07')]:
            body=bytes.fromhex(r['plain']);frames.append({'capture':str(path),'direction':r['direction'],'hex':body.hex(),'name':body[1:-1].decode('cp950') if body[0]==5 else None})
assert len([f for f in frames if f['hex']=='070000000000'])==2
for c in receipt['clients'].values():
    assert any(f['name']==c['directory'][0]['name'] for f in frames)
out={'scope':'C05 from the native Tournament character directory only','exe_sha256':receipt['exe_sha256'],'jar_sha256':receipt['jar_sha256'],
 'rust_tests':14,'rust_suites':4,'rust_log':'驗證紀錄/current/native-world-integration/rust-classic-entry.log','frames':frames,
 'observation':'Both exact received display identities receive NO_TRANSITION; no world scene is supplied.',
 'not_inferred':'This does not show that every classic entry path or rental-to-world portal is unsupported.'}
evidence='驗證紀錄/current/native-world-integration/classic-entry-boundary.json'
(root/evidence).write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
p=root/'工作佇列.json';q=json.loads(p.read_text(encoding='utf-8-sig'));g=next(x for x in q['items'] if x['id'].startswith('G06'))
check=next(x for x in g['checks'] if x['id']=='G06.4');check.update(state='native_directory_boundary_verified_other_routes_pending',evidence=evidence,
 qualified_scope='兩端實送原生清單的顯示身份 C05 均回 070000000000，無世界資料。正式原始角色為 TestKnight／TestMage，與原生別名不同；不可剝除後綴猜身份。Classic 原始清單與租裝／戰鬥傳送門入口仍待獨立驗證。')
g['evidence'].append(evidence);g['checks'][2]['state']='in_progress';q['last_verified_baseline']['current_working_tree_verified']=False
p.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'frames':len(frames),'exe_sha256':receipt['exe_sha256']}))
