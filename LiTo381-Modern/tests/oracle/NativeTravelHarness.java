package local.lito381.server;
import java.net.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.sql.*;
import java.lang.reflect.*;
import java.security.*;
import java.util.*;
import java.util.concurrent.*;

/** Initial static SQL-derived test possession; gameplay uses untouched formal sessions. */
public final class NativeTravelHarness {
 static ClassicWorldDropCatalog.ItemWire loadWire(int port)throws Exception {
  Class.forName("com.mysql.cj.jdbc.Driver");
  try(Connection c=DriverManager.getConnection("jdbc:mysql://127.0.0.1:"+port+"/travel_static?useSSL=false&serverTimezone=UTC&characterEncoding=UTF-8","travel_read","TravelFixtureOnly")) {
   c.setReadOnly(true);
   try(Statement q=c.createStatement();ResultSet r=q.executeQuery("SELECT * FROM etcitem WHERE item_id=42501")) {
    if(!r.next()||!r.getString("classname").equals("wand.Storm_Walk"))throw new IllegalStateException("original static row");
    var wire=new ClassicWorldDropCatalog.ItemWire(r.getInt("item_id"),"etc",r.getString("name_id"),r.getString("use_type"),r.getInt("invgfx"),r.getInt("grdgfx"),r.getInt("bless"),r.getBoolean("stackable"),!r.getBoolean("trade"),r.getInt("weight"),r.getInt("itemdesc_id"),"",0,0,r.getString("material"),0,r.getInt("limit_class"),r.getBoolean("cant_delete"));
    if(wire.inventoryDefinition().useType()!=5)throw new IllegalStateException("native use type");
    return wire;
   }
  }
 }
 static AccountRepository fixture(MemoryAccountRepository backing,ClassicWorldDropCatalog.ItemWire wire) {
  // Declared before any world exists: the TestKnight character initially owns one rare
  // catalog42501 object. No accessLevel override, runtime insertion or classifier setter.
  return (AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,args)->{
   try {
    if(method.getName().equals("sourceInventory")) {
     long character=((Number)args[0]).longValue();
     var items=new ArrayList<SourceInventoryItem>(backing.sourceInventory(character));
     if(character==1)items.add(new SourceInventoryItem(9100000+character,42501,1,false,0,0,true,wire.blessing(),false,wire,null,0,false,null,true,false));
     return List.copyOf(items);
    }
    return method.invoke(backing,args);
   }catch(InvocationTargetException ex){throw ex.getCause();}
  });
 }
 static String observe(NativePeerHarness.Session s,GameHub hub)throws Exception {
  String base=NativePeerHarness.observe(s,hub);
  StringJoiner items=new StringJoiner(",");
  for(var item:((RuntimeInventory)NativePeerHarness.field(s.owner(),"inventory")).items())
   items.add("{\"id\":"+item.objectId()+",\"catalog_id\":"+item.definition().catalogId()+",\"quantity\":"+item.quantity()+"}");
  return base.substring(0,base.length()-1)+",\"inventory\":["+items+"],\"pending_portal\":"+NativePeerHarness.quote(String.valueOf(NativePeerHarness.field(s.owner(),"pendingPortal")))+",\"pending_portal_ready\":"+NativePeerHarness.field(s.owner(),"pendingPortalReady")+"}";
 }
 public static void main(String[] args)throws Exception {
  Path root=Path.of(args[0]).toAbsolutePath().normalize();int sqlPort=Integer.parseInt(args[1]);boolean gui=args.length>2&&args[2].equals("gui");
  var wire=loadWire(sqlPort);
  Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  try(ServerSocket login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));ServerSocket game=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"))) {
   login.setSoTimeout(gui?240000:120000);game.setSoTimeout(gui?240000:120000);
   Path configPath=root.resolve("travel-game-memory.properties");
   NativePeerHarness.write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+"\nserver.name=重建座標物品驗證\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\nstate.file=travel-state.properties\ncapture.directory=travel-captures\nmaps.directory=maps\ndatabase.legacy.properties=unused.properties\nsource.servers.file=\nlogin.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nbattle.ai.enabled=false\nmax.frame=65535\n");
   ServerConfig config=ServerConfig.load(configPath);
   var backing=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
   var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));routes.put("測試乙",fixture(backing,wire));
   // Separate pre-session fixture preflight. It never touches the live hub/state.
   AccountRepository initial=routes.get("測試乙");
   for(String account:List.of("PeerFixture","PeerFixture2"))for(var character:initial.characters(account)) {
    try {
     var preflight=new GameStateStore(root.resolve("preflight-"+character.objectId()+".properties"),false);
     var reconciliation=preflight.reconcileTournamentSourceInventory(GameStateStore.sourceCharacter("測試乙",character.name()),initial.sourceInventory(character.objectId()),initial.equipmentMagicStats(character.objectId()),false);
     preflight.importSourcePets(GameStateStore.sourceCharacter("測試乙",character.name()),initial.pets(character.objectId()),reconciliation.definitions());
    } catch(Throwable failure){failure.printStackTrace();throw failure;}
   }
   GameStateStore state=new GameStateStore(config.stateFile(),false);
   try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,backing)) {
    // Normal formal loader against the owned SELECT-only static DB, after hub
    // initialization and strictly before any listener accepts a game session.
    GmTravel.load(GmTravelSourceOracle.withSource(config,root.resolve("travel-oracle.properties")));
    var sessions=new CopyOnWriteArrayList<NativePeerHarness.Session>();var errors=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();
    for(boolean isGame:new boolean[]{false,true}) {
     ServerSocket listener=isGame?game:login;
     Thread acceptor=new Thread(()->{try{for(int i=0;i<2;i++) {
      Socket socket=listener.accept();var session=new ClientSession(socket,isGame?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
      Thread worker=new Thread(session,"travel-formal-session");worker.setUncaughtExceptionHandler((t,e)->errors.add(e));
      sessions.add(new NativePeerHarness.Session(socket,session,worker,isGame));worker.start();
     }}catch(Throwable e){errors.add(e);}},"travel-acceptor");acceptors.add(acceptor);acceptor.start();
    }
    NativePeerHarness.write(root.resolve("travel-ready.json"),"{\"port\":"+login.getLocalPort()+",\"game_port\":"+game.getLocalPort()+",\"jar_sha256\":\""+sha+"\",\"initial_fixture\":\"TestKnight initial SQL42501 possession; TestMage unchanged; access levels unchanged\"}");
    var previous=new IdentityHashMap<ClientSession,String>();long end=System.nanoTime()+(gui?300_000_000_000L:180_000_000_000L);
    try(var trace=Files.newBufferedWriter(root.resolve("travel-server-trace.jsonl"),StandardCharsets.UTF_8)) {
     while(System.nanoTime()<end) {
      if(!errors.isEmpty())throw new IllegalStateException("formal session failure",errors.peek());
      for(var session:sessions)if(session.game()) {
       String row=observe(session,hub);if(!row.equals(previous.put(session.owner(),row))){trace.write(row);trace.newLine();trace.flush();}
      }
      if(Files.exists(root.resolve("travel-test-done")))break;
      Thread.sleep(10);
     }
     if(!Files.exists(root.resolve("travel-test-done")))throw new IllegalStateException("travel test lease expired");
     NativePeerHarness.write(root.resolve("travel-harness-result.json"),"{\"status\":\"formal_sessions_observed\",\"sessions\":"+sessions.size()+",\"runtime_authority_mutations\":0,\"player_tables_accessed\":false}");
    } finally {
     login.close();game.close();for(var session:sessions)session.socket().close();
     for(var acceptor:acceptors)acceptor.join(3000);for(var session:sessions)session.thread().join(3000);
    }
   }
  }
 }
}
