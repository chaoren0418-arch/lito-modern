"""Independent diagnostic client: establish whether C02 can complete same-socket reselect."""
from pathlib import Path
import argparse,json,socket,struct,time
ap=argparse.ArgumentParser();ap.add_argument('root',type=Path);a=ap.parse_args();root=a.root.resolve(strict=True)
base=Path(__file__).resolve().parents[1];assert root.is_relative_to(base/'LiTo381-Modern/target/current')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
ready=read(root/'ready.json');version=read(base/'LiTo381-Modern/tests/fixtures/native-version-current.json');login=read(base/'LiTo381-Modern/tests/fixtures/native-login-current.json')
rows=[];clients=[];result={'status':'running','Rust_client_test':False,'GUI_verified':False,'whole_goal_complete':False}
class Peer:
 def __init__(self,host,port,role):
  assert host in ['127.0.0.1','127.0.0.2'];self.socket=socket.create_connection((host,port),timeout=5);self.socket.settimeout(5);self.role=role;clients.append(self)
  key=self.frame();assert len(key)==5 and key[0]==1
  self.rx=self.tx=(0x87546ca1<<32)|int.from_bytes(key[1:],'little')
  self.send(bytes.fromhex(version['request']));self.until(lambda b:b[0]==0x3a)
 def exact(self,n):
  out=bytearray()
  while len(out)<n:
   b=self.socket.recv(n-len(out));assert b,'EOF before expected diagnostic response';out.extend(b)
  return bytes(out)
 def frame(self):
  n=int.from_bytes(self.exact(2),'little');assert 2<n<=65535;return self.exact(n-2)
 def send(self,plain):
  b=bytearray(plain);key=self.tx.to_bytes(8,'little');previous=0
  for i in range(len(b)):b[i]^=key[i&7]^previous;previous=b[i]
  self.tx=(self.tx+len(b))&((1<<64)-1);self.socket.sendall(struct.pack('<H',len(b)+2)+b)
  rows.append({'role':self.role,'direction':'client_to_server','plain':'MASKED_C_LOGIN' if plain[0]==1 else plain.hex(),'monotonic':time.monotonic()})
 def recv(self):
  b=bytearray(self.frame());key=self.rx.to_bytes(8,'little');previous=0
  for i in range(len(b)):cipher=b[i];b[i]^=key[i&7]^previous;previous=cipher
  self.rx=(self.rx+len(b))&((1<<64)-1);plain=bytes(b);rows.append({'role':self.role,'direction':'server_to_client','plain':plain.hex(),'monotonic':time.monotonic()});return plain
 def until(self,predicate):
  for _ in range(256):
   b=self.recv()
   if predicate(b):return b
  raise AssertionError('Diagnostic response bound exceeded')
 def character(self):
  assert self.until(lambda b:b[0]==3)==bytes([3,1]);row=self.until(lambda b:b[0]==0xcb);return row[1:row.index(0,1)]
 def channel(self,n):
  self.send(bytes([0xca,n]));result=self.until(lambda b:b[0]==0xc9);assert result[:3]==bytes([0xc9,0,n]);info=self.until(lambda b:b[0]==0xca);assert info[1]==n
  self.send(bytes([0xce]));self.send(bytes([0xca,n]));info=self.until(lambda b:b[0]==0xca);assert info[1]==n
try:
 first=Peer('127.0.0.1',ready['port'],'LOGIN');first.send(bytes.fromhex(login['login']));first.until(lambda b:b[0]==0x3b);first.send(bytes.fromhex(login['source_request']));redirect=first.until(lambda b:b[0]==0xf4)
 host=socket.inet_ntoa(redirect[2:6]);port=int.from_bytes(redirect[6:8],'little');assert port==ready['game_port']
 game=Peer(host,port,'GAME');game.send(bytes.fromhex(login['login']));game.until(lambda b:b[0]==0xe6);game.send(bytes([0xe2]));name=game.character();game.send(bytes([0xc9])+name+b'\0');assert game.until(lambda b:b[0]==0xc8)==bytes([0xc8,0]);game.channel(0)
 game.send(bytes([0xcb]));game.send(bytes([0xca,100]));assert game.until(lambda b:b[0]==0xf5)==bytes([0xf5]);game.send(bytes.fromhex(login['source_request']));again=game.character();assert again==name
 game.send(bytes([0xc9])+again+b'\0');assert game.until(lambda b:b[0]==0xc8)==bytes([0xc8,0]);result['second_selection_ack_received']=True
 game.channel(3);result.update(status='independent_current_reference_reselect_channel3_completed',final_channel=3)
except BaseException as error:
 result.update(status='failed',error=f'{type(error).__name__}: {error}');raise
finally:
 for peer in clients:peer.socket.close()
 (root/'diagnostic-wire.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 (root/'diagnostic-result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,ensure_ascii=False))
