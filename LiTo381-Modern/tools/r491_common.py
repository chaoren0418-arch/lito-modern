from pathlib import Path
import json,hashlib,datetime
P=Path(__file__).resolve().parent.parent
L=P/'logs/continuation-20261003'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def pin(p):
 p=Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def progress(phase,task):
 p=P.parent/'監製進度.json';v=read(p);v['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();v['phase']=phase;v['current_task']=task;v['overall_status']='in_progress';v['r490']['new_player_flow_review']='T350_FINITE_ACCEPTED';save(p,v)
