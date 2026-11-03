package local.lito381.server;
import java.net.*;
import java.nio.file.*;
import java.security.*;
import java.util.*;
import java.util.concurrent.*;

/** Three real peers; initial memory fixture and passive roster observations only. */
public final class NativeObserverHarness {
 public static void main(String[] args)throws Exception {
  Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
  Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  try(var login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));var game=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"))) {
   login.setSoTimeout(60000);game.setSoTimeout(60000);Path path=root.resolve("server.properties");
   NativePeerHarness.write(path,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+"\nserver.name=觀戰實測\nlogin.classic.server.names=測試乙,測試甲\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\nstate.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused.properties\nsource.servers.file=\nlogin.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=false\nmax.frame=65535\n");
   var config=ServerConfig.load(path);if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation");
   for(var component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path p=(Path)component.getAccessor().invoke(config);if(p!=null&&!p.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
   var backing=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",backing);routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
   var state=new GameStateStore(config.stateFile(),false);
   try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);var capture=new CaptureLog(config.captureDirectory());var hub=new GameHub(config,state,backing)) {
    var sessions=new CopyOnWriteArrayList<NativePeerHarness.Session>();var errors=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();
    for(boolean isGame:new boolean[]{false,true}) {
     Thread acceptor=new Thread(()->{try {var listener=isGame?game:login;for(int n=0;n<3;n++){var socket=listener.accept();var owner=new ClientSession(socket,isGame?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);var worker=new Thread(owner,"observer-session");worker.setUncaughtExceptionHandler((t,e)->errors.add(e));sessions.add(new NativePeerHarness.Session(socket,owner,worker,isGame));worker.start();}}catch(Throwable e){errors.add(e);}},"observer-acceptor");acceptors.add(acceptor);acceptor.start();
    }
    try {
     NativePeerHarness.write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"game_port\":"+game.getLocalPort()+",\"jar_sha256\":\""+sha+"\"}");
     long until=System.nanoTime()+200_000_000_000L;var previous=new IdentityHashMap<ClientSession,String>();
     try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"))){while(System.nanoTime()<until){if(!errors.isEmpty())throw new IllegalStateException("worker failure",errors.peek());for(var session:sessions)if(session.game()) {
      String row=NativePeerHarness.observe(session,hub);StringJoiner roster=new StringJoiner(",");
      synchronized(hub){Object room=((Map<?,?>)NativePeerHarness.field(hub,"rooms")).get((long)NativePeerHarness.field(session.owner(),"currentRoomId"));if(room!=null)for(Object member:(Collection<?>)NativePeerHarness.field(room,"battleRoster"))roster.add(Long.toString(((ClientSession)member).playerObjectId()));}
      row=row.substring(0,row.length()-1)+",\"combatant_ids\":["+roster+"]}";
      if(!row.equals(previous.put(session.owner(),row))){trace.write(row+"\n");trace.flush();}
     }if(sessions.size()==6&&acceptors.stream().noneMatch(Thread::isAlive)&&sessions.stream().noneMatch(s->s.thread().isAlive()))break;Thread.sleep(20);}}
     if(sessions.size()!=6||sessions.stream().anyMatch(s->s.thread().isAlive()))throw new IllegalStateException("session cleanup");
     for(int c=0;c<4;c++){if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("channel leak");for(var room:hub.roomSnapshots(c))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room leak");}
     NativePeerHarness.write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":6,\"memberships_empty\":true,\"authority_mutations\":0,\"sql_access\":false,\"state_persistent\":false,\"jar_sha256\":\""+sha+"\"}");
    }finally {login.close();game.close();for(var s:sessions)s.socket().close();for(var t:acceptors)t.join(3000);for(var s:sessions){s.thread().join(3000);if(s.thread().isAlive())throw new IllegalStateException("worker leak");}}
   }
  }
 }
}
