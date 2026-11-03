"""Extract exact static table statements required by E149. Never execute SQL."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==ref['formal_sha256']=='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
inspection=V/'native-E149-production-item-bootstrap/20261023-195115-607039'
query_source=inspection/'NativeDeItemDefinitionsSql.stdout.txt';main_source=inspection/'Main.stdout.txt'
queries=[s for s in re.findall(r'// String (.*)',query_source.read_text('utf-8-sig')) if s.startswith('SELECT ')]
tables=sorted({n for q in queries for n in re.findall(r'(?:FROM|JOIN) `([^`]+)`',q)})
assert len(queries)==18 and len(tables)==19
main=main_source.read_text('utf-8-sig');assert main.index('NativeServerLifetime.install:')<main.index('NativeDeItemApplication.bootstrap:')<main.index('class java/net/ServerSocket')
source=R.parent/'lito伺服器/LiTo381-Server/db/lito_static.sql'
raw=source.read_bytes();source_hash=digest(raw)
assert source_hash=='D4111D8AEFF50EA5D9D36630C7381132812F7F727E8432FFAF035B89C9CBB075'
raw.decode('utf-8-sig')

def statements(blob):
    # Byte offsets preserve original UTF-8 exactly. Quotes, doubled quotes,
    # backslash escapes, --/# comments and block comments hide semicolons.
    start=0;i=0;quote=None;line=False;block=False
    while i<len(blob):
        c=blob[i];n=blob[i+1] if i+1<len(blob) else None
        if line:
            if c in (10,13):line=False
        elif block:
            if c==42 and n==47:block=False;i+=1
        elif quote is not None:
            if c==92:i+=1
            elif c==quote:
                if n==quote:i+=1
                else:quote=None
        elif c in (39,34,96):quote=c
        elif c==35:line=True
        elif c==45 and n==45 and (i+2==len(blob) or blob[i+2]<=32):line=True;i+=1
        elif c==47 and n==42:block=True;i+=1
        elif c==59:
            yield start,i+1,blob[start:i+1];start=i+1
        i+=1
    assert quote is None and not block
    if blob[start:].strip():
        assert code(blob[start:]).strip()==b'', 'unterminated noncomment SQL tail'

def code(statement):
    pos=0
    while True:
        while pos<len(statement) and statement[pos]<=32:pos+=1
        if statement[pos:pos+2]==b'--' or statement[pos:pos+1]==b'#':
            end=statement.find(b'\n',pos);pos=len(statement) if end<0 else end+1
        elif statement[pos:pos+2]==b'/*':
            end=statement.find(b'*/',pos+2);assert end>=0;pos=end+2
        else:return statement[pos:]

def value_rows(body):
    match=re.search(rb'\bVALUES\s*',body,re.I);assert match
    values=body[match.end():];quote=None;depth=0;rows=0;i=0
    while i<len(values):
        c=values[i];n=values[i+1] if i+1<len(values) else None
        if quote is not None:
            if c==92:i+=1
            elif c==quote:
                if n==quote:i+=1
                else:quote=None
        elif c in (39,34):quote=c
        elif c==40:
            if depth==0:rows+=1
            depth+=1
        elif c==41:depth-=1;assert depth>=0
        i+=1
    assert quote is None and depth==0 and rows>0
    return rows

selected=[];info={n:dict(create_statements=0,insert_statements=0,rows=0,columns=[],source_statements=[]) for n in tables}
for start,end,statement in statements(raw):
    body=code(statement)
    match=re.match(rb'(CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?|INSERT\s+INTO|REPLACE\s+INTO|UPDATE|ALTER\s+TABLE|DROP\s+TABLE(?:\s+IF\s+EXISTS)?)\s+`([^`]+)`',body,re.I)
    if not match:continue
    name=match.group(2).decode('utf-8')
    if name not in info:continue
    action=re.sub(rb'\s+',b' ',match.group(1).upper())
    if action.startswith(b'DROP TABLE'):continue
    assert action in [b'CREATE TABLE',b'INSERT INTO'],(name,action.decode())
    entry=info[name]
    if action==b'CREATE TABLE':
        entry['create_statements']+=1
        entry['columns']=[v.decode('utf-8') for v in re.findall(rb'^\s*`([^`]+)`\s+\w+',body,re.M)]
        assert entry['columns']
        refs=[v.decode('utf-8') for v in re.findall(rb'\bREFERENCES\s+`([^`]+)`',body,re.I)]
        assert set(refs)<=set(tables),(name,refs)
        row_count=0
    else:
        assert entry['create_statements']==1
        row_count=value_rows(body);entry['insert_statements']+=1;entry['rows']+=row_count
    pin=dict(offset=start,end=end,bytes=len(statement),sha256=digest(statement),kind=action.decode(),rows=row_count)
    entry['source_statements'].append(pin);selected.append((name,pin,statement))
assert all(x['create_statements']==1 for x in info.values())
assert all(info[n]['rows']>0 for n in ['etcitem','weapon','armor'])
# No USE, DATABASE, DROP, UPDATE or non-whitelisted statements are emitted.
# Original CREATE/INSERT bytes and source ordering remain intact.
header=b'-- E149 static items: only import through an owned isolated database runner.\nSET NAMES utf8mb4;\n'
payload=header+b'\n'.join(s for _,_,s in selected)+b'\n'
out=R/'維護工具/resources'/('E149-static-item-tables-'+source_hash[:12]);out.mkdir(parents=True,exist_ok=True)
sql=out/'static-tables.sql'
if sql.exists():assert sql.read_bytes()==payload
else:sql.write_bytes(payload)
record=dict(status='E149_original_static_item_statements_prepared_not_imported',formal_target_sha256=ref['formal_sha256'],
    source_dump=str(source),source_dump_sha256=source_hash,source_dump_bytes=len(raw),source_encoding='utf-8',
    query_disassembly=str(query_source),query_disassembly_sha256=sha(query_source),original_queries=queries,
    original_Main=str(main_source),original_Main_sha256=sha(main_source),startup_order=['NativeServerLifetime.install','NativeDeItemApplication.bootstrap','create/bind listeners and accept'],
    selected_tables=info,table_count=len(tables),query_count=len(queries),rows=sum(v['rows'] for v in info.values()),
    statement_count=len(selected),SQL=str(sql),SQL_sha256=sha(sql),SQL_bytes=len(payload),
    original_selected_statements_byte_preserved=True,player_tables_selected=False,source_files_modified=False,
    import_executed=False,database_started=False,SQL_executed=False,tests_executed=0,
    limitations=['Presence and extraction only; actual original E149 loader/column/SQL-mode compatibility has not run',
        'DDL/data import requires a fresh owned isolated database, never any formal database',
        'Production loader must complete its original read-only transaction rollback/close/validation before publication; do not replace it with constructed empty snapshots'],
    whole_goal_complete=False)
manifest=out/'manifest.json'
if manifest.exists():assert read(manifest)==record
else:manifest.write_bytes(data(record))
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
assert sha(source)==source_hash and sha(ref['formal'])==ref['formal_sha256']
print(json.dumps(dict(manifest=str(manifest),SQL=str(sql),tables=len(tables),rows=record['rows'],statement_count=len(selected),bytes=len(payload),SQL_executed=False),ensure_ascii=False))
