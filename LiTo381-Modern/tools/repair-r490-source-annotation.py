from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';F=P/'target/r490-state-flow-study/verifier-attempt1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
e=read(L/'r490-source-evidence.json');assert 'mapcontrol+20'in e['map_alias'];e['map_alias']=e['map_alias'].replace('mapcontrol+20','mapcontrol+8');save(L/'r490-source-evidence.json',e)
findings=read(L/'r490-source-findings.json');findings['proven_map_alias']=e['map_alias'];save(L/'r490-source-findings.json',findings)
save(L/'r490-verification-failure.json',dict(state='R490_SOURCE_ANNOTATION_AND_VERIFIER_CONTROL_SLOT_CORRECTED_BEFORE_SEAL',failure='Initial summary-derived control+20 annotation/assertion contradicted474280 actualstore[eax+8]=list. Correctedto+8 fromcompletepinnedconstructor. Global64A968maplist and206450230clear relation unchanged.',frozen_attempt=[pin(p)for p in sorted(F.iterdir())if p.is_file()],production_changed_after_consumer=False,consumer_rerun=False))
print('R490 source control slot corrected+8;firstfailedsource/verifier frozen;production/consumer unchanged')
