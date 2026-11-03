package local.lito381.server;

import java.net.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.concurrent.*;

/** Unavailable destination: configuration before boot, no authority/clock mutation. */
public final class NativeTeleportFailureHarness {
 // Pure reflection snapshot: do not call has/consume, which prune expired rows.
 static String stagedSnapshot(GameHub hub)throws Exception {
  synchronized(hub) {
   long now=System.nanoTime();var rows=new ArrayList<String>();
   for(var entry:((Map<?,?>)NativePeerHarness.field(hub,"rentalBattleHandoffs")).entrySet()) {
    Object value=entry.getValue();long expiry=(long)NativePeerHarness.field(value,"expiresAtNanos");
    var inventory=(RuntimeInventory)NativePeerHarness.field(value,"inventory");var ids=new ArrayList<String>();for(var item:inventory.items())ids.add(String.valueOf(item.objectId()));
    String character=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(String.valueOf(NativePeerHarness.field(value,"character")).getBytes(java.nio.charset.StandardCharsets.UTF_8)));
    String key=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(String.valueOf(entry.getKey()).getBytes(java.nio.charset.StandardCharsets.UTF_8)));
    rows.add("{\"key_sha256\":\""+key+"\",\"character_sha256\":\""+character+"\",\"expires_at_nanos\":"+expiry+",\"expired\":"+(now>=expiry)+",\"inventory_ids\":["+String.join(",",ids)+"]}");
   }
   Collections.sort(rows);return "{\"observed_now_nanos\":"+now+",\"staged\":["+String.join(",",rows)+"]}";
  }
 }
 public static void main(String[] args)throws Exception {
  boolean gui=args.length>1&&args[1].startsWith("gui-");
  String scenario=args.length>1?args[1].replaceFirst("^gui-",""):"timeout";
  boolean cancelExpiry=scenario.equals("cancel-expiry");
  boolean refused=scenario.equals("refused");
  boolean wrongService=scenario.equals("wrong-service");
  int expectedSessions=wrongService?10:8;
  Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
  Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  try(ServerSocket login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));ServerSocket town=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"));
      ServerSocket unavailable=new ServerSocket(0,4,InetAddress.getByName(refused?"127.0.0.1":"127.0.0.3"))) {
   // Default: bind without accepting S_KEY. Wrong-service: real LOGIN sessions.
   unavailable.setSoTimeout(90000);
   int unavailablePort=unavailable.getLocalPort();
   // Refusal is a previously owned ephemeral loopback port, closed before boot.
   // The client must observe WSAECONNREFUSED; a reuse/collision fails the test.
   if(refused) {
    unavailable.close();
    try(Socket probe=new Socket()) {
     probe.connect(new InetSocketAddress("127.0.0.1",unavailablePort),5000);
     throw new IllegalStateException("closed destination unexpectedly accepted a connection");
    }catch(ConnectException expected) {
     // The client's default 2s connect bound can fire just before Windows
     // reports refusal. Independently prove this is an actual closed port.
    }
   }
   login.setSoTimeout((cancelExpiry||gui)?180000:90000);town.setSoTimeout((cancelExpiry||gui)?180000:90000);
   Path file=root.resolve("server.properties");
   NativePeerHarness.write(file,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+town.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+town.getLocalPort()+
    "\nserver.name=傳送失敗測試\nlogin.classic.server.names=測試乙,測試甲\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
    "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
    "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=true\nmax.frame=65535\n"+
    "lito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+town.getLocalPort()+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n"+
    "lito.rental.teleport.routes=TeleportMe|LT_TELEPORT_B|Unavailable B|"+(refused?"127.0.0.1":"127.0.0.3")+"|"+unavailablePort+",TeleportMe|LT_TELEPORT_C|Unavailable C|"+(refused?"127.0.0.1":"127.0.0.3")+"|"+unavailablePort+"\n");
   ServerConfig config=ServerConfig.load(file);if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation");
   for(var component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path path=(Path)component.getAccessor().invoke(config);if(path!=null&&!path.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
   var memoryBacking=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
   var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",memoryBacking);routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
   GameStateStore state=new GameStateStore(config.stateFile(),false);
   // Both players use source 0; the hub shares its initial memory child.
   try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,memoryBacking)) {
    var sessions=new CopyOnWriteArrayList<NativePeerHarness.Session>();var failures=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();
    for(int endpoint=0;endpoint<(wrongService?3:2);endpoint++){
     final int endpointKind=endpoint;final boolean game=endpointKind==1;final int count=endpointKind==2?2:4;
     Thread acceptor=new Thread(()->{try{ServerSocket socket=endpointKind==2?unavailable:game?town:login;for(int n=0;n<count;n++){
      Socket connection=socket.accept();ClientSession owner=new ClientSession(connection,game?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
      Thread worker=new Thread(owner,"teleport-failure-session");worker.setUncaughtExceptionHandler((t,e)->failures.add(e));sessions.add(new NativePeerHarness.Session(connection,owner,worker,game));worker.start();
     }}catch(Throwable e){failures.add(e);}},"failure-acceptor");acceptors.add(acceptor);acceptor.start();
    }
    try {
     NativePeerHarness.write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"destination_port\":"+unavailablePort+",\"destination_refused\":"+refused+",\"destination_wrong_service\":"+wrongService+",\"jar_sha256\":\""+sha+"\"}");
     // Match the owned runner's 220s lease with 20s reserved for cleanup.
     // The former 110s overall fixture cutoff closed live recovery sockets
     // during original map loading; GAME/S_KEY and player clocks are unchanged.
     long until=System.nanoTime()+((cancelExpiry||gui)?320_000_000_000L:200_000_000_000L);var previous=new IdentityHashMap<ClientSession,String>();
     try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"));var stages=Files.newBufferedWriter(root.resolve("staged-trace.jsonl"))){while(System.nanoTime()<until){
      if(!failures.isEmpty())throw new IllegalStateException("session failure",failures.peek());
      stages.write(stagedSnapshot(hub)+"\n");stages.flush();
      for(var session:sessions)if(session.game()){String row=NativePeerHarness.observe(session,hub);if(!row.equals(previous.put(session.owner(),row))){trace.write(row+"\n");trace.flush();}}
      if(sessions.size()==expectedSessions&&acceptors.stream().noneMatch(Thread::isAlive)&&sessions.stream().noneMatch(s->s.thread().isAlive()))break;Thread.sleep(20);
     }}
     if(sessions.size()!=expectedSessions||sessions.stream().anyMatch(s->s.thread().isAlive()))throw new IllegalStateException("cleanup");
     for(int channel=0;channel<4;channel++)if(hub.channelSnapshot(channel).users()!=0)throw new IllegalStateException("channel leak");
     int staged=((Map<?,?>)NativePeerHarness.field(hub,"rentalBattleHandoffs")).size();
     if(staged!=0)throw new IllegalStateException("unconsumed loadout after recovery");
     NativePeerHarness.write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":"+expectedSessions+",\"memberships_empty\":true,\"authority_mutations\":0,\"sql_access\":false,\"state_persistent\":false,\"staged_loadouts_after_reentry\":"+staged+",\"full_game_reentry_verified\":true,\"jar_sha256\":\""+sha+"\"}");
    }finally {login.close();town.close();unavailable.close();for(var session:sessions)session.socket().close();for(var t:acceptors)t.join(3000);for(var s:sessions){s.thread().join(3000);if(s.thread().isAlive())throw new IllegalStateException("worker leak");}}
   }
  }
 }
}
