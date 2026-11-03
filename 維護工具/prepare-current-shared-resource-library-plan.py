"""Inventory installed-resource opt-ins from the actual sealed library audit.

Extracts exact test bodies for review. Runs no test, compiler, JVM or UI.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
parser=argparse.ArgumentParser();parser.add_argument('library',type=Path);a=parser.parse_args()
base=a.library.resolve(strict=True);assert base.is_relative_to(V/'native-rental-service-entry-return-library-default')
c=read(base/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
qp=base/'Debug/full-default-qualified/qualified.json';q=read(qp)
assert q['source_manifest_sha256']==source and q['failed']==q['Rust_exit_code']==0
audit_path=base/'Debug/inventory-audit/ignored-source-reasons.json';audit=read(audit_path)
inputs=read(base/'compiled-inputs.json');assert len(inputs)==1049

def function_body(text,name,ignore_line):
    lines=text.splitlines(keepends=True)
    offset=sum(map(len,lines[:ignore_line-1]))
    match=re.search(r'\bfn\s+'+re.escape(name)+r'\s*\(',text[offset:])
    assert match is not None
    start=offset+match.start();open_at=text.index('{',offset+match.end())
    assert text[offset:start].count('\n')<15
    depth=0;i=open_at
    while i<len(text):
        if text.startswith('//',i):
            end=text.find('\n',i);i=len(text)if end<0 else end+1;continue
        if text.startswith('/*',i):
            level=1;i+=2
            while level:
                assert i<len(text)
                if text.startswith('/*',i):level+=1;i+=2
                elif text.startswith('*/',i):level-=1;i+=2
                else:i+=1
            continue
        raw=re.match(r'r(#{0,32})"',text[i:])if text[i]=='r'else None
        if raw:
            end=text.find('"'+raw[1],i+raw.end());assert end>=0
            i=end+1+len(raw[1]);continue
        if text[i]=='"':
            i+=1
            while i<len(text):
                if text[i]=='\\':i+=2
                elif text[i]=='"':i+=1;break
                else:i+=1
            continue
        if text[i]=="'":
            char=re.match(r"'(?:\\(?:u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2}|.)|[^'\\\n])'",text[i:])
            if char:i+=char.end();continue
        if text[i]=='{':depth+=1
        elif text[i]=='}':
            depth-=1
            if depth==0:return text[start:i+1],text[:start].count('\n')+1,text[:i+1].count('\n')+1
        i+=1
    raise AssertionError('unterminated test body '+name)

selected=[];bodies={};concerns=[]
for row in audit['tests']:
    candidates=row.get('candidates',[])
    if len(candidates)!=1 or 'LITO_SHARED_RESOURCE_ROOT'not in candidates[0]['reason']:continue
    item=candidates[0];path=R/item['source'];assert sha(path)==item['source_sha256']==inputs[item['source']]
    body,start,end=function_body(path.read_text('utf-8-sig'),row['test'].split('::')[-1],item['ignore_line'])
    operations=[line.strip()for line in body.splitlines()if re.search(r'File::create|fs::write|create_dir|write_png|save_png|write_image|Command::|TcpListener|TcpStream|UdpSocket|CreateWindow|CreateDesktop|SendInput|SendMessage|waveOut|WaveOut',line)]
    if operations:concerns.append(dict(test=row['test'],operations=operations))
    selected.append(dict(test=row['test'],source=item['source'],source_sha256=item['source_sha256'],
        ignore_reason=item['reason'],start_line=start,end_line=end,body_sha256=hashlib.sha256(body.encode('utf-8')).hexdigest().upper(),
        operations_requiring_review=operations))
    bodies[row['test']]=body
assert len(selected)==72 and len({item['test']for item in selected})==72
out=V/'native-library-shared-resource-optin'/('plan-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'test-bodies.json').write_text(json.dumps(bodies,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'planner.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='installed_shared_resource_optin_plan_pending_body_review_no_execution',
    source_manifest_sha256=source,library=str(base),Debug_qualification=str(qp),Debug_qualification_sha256=sha(qp),
    audit=str(audit_path),audit_sha256=sha(audit_path),selected_tests=selected,expected_tests=72,
    selected_bodies=str(out/'test-bodies.json'),selected_bodies_sha256=sha(out/'test-bodies.json'),
    environment_required=dict(LITO_SHARED_RESOURCE_ROOT=r'D:\lineage開發\0-客戶端'),
    explicit_operation_review=concerns,review_completed=False,compiler_or_runtime_started=False,
    additional_filters_require_help_and_list_validation=True,legacy_output_preservation_required=True,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'plan.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(plan=str(out/'plan.json'),expected_tests=72,operations_to_review=concerns,executed=False),ensure_ascii=False))
