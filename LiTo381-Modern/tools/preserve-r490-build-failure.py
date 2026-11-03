from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r490-fixed-event-consumer/failed-build';O.mkdir(exist_ok=False)
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
pins=[]
for p in [P/'tools/r490-state-flow-consumer.rs',P/'tools/build-r490-state-consumer.ps1',L/'r490-consumer-build.stderr.log',L/'r490-consumer-build.stdout.log']:
 dest=O/p.name;shutil.copyfile(p,dest);pins.append(pin(dest))
out=dict(state='R490_FIRST_CONSUMER_COMPILE_FAILURE_SOURCE_AND_LOGS_PRESERVED_NO_EXECUTION',attempt=1,failure='Consumer helper omitted native_resources::BytePin import;productionlibrary/player alreadybuilt successfully unchanged. Correcthelper import andnewfixedconsumer directory beforefirstexecution.',files=pins,executions=0,GUI=0,LOADED=False)
(L/'r490-consumer-build-failure.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('R490 firstfailed consumer source/script/logs preserved;0executions')
