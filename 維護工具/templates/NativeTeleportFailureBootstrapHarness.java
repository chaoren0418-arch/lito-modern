package local.lito381.server;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicBoolean;

/** Isolated G06 fixture with original E149 static-resource and native lifetime.
 * This is not a complete Main process or a native-owner qualification grant.
 */
public final class NativeTeleportFailureHarness {
 static final Path REBUILD=Path.of("D:/lineage開發/3-lito開發/lito重建版").toAbsolutePath().normalize();
 static final String FORMAL="E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498";
 static void require(boolean ok,String message){if(!ok)throw new IllegalStateException(message);}
 static String hash(Path file)throws Exception {
  return HexFormat.of().withUpperCase().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(file)));
 }
 static Throwable append(Throwable first,Throwable next){if(first==null)return next;if(first!=next)first.addSuppressed(next);return first;}
 static void rethrow(Throwable error)throws Exception {
  if(error instanceof Exception exception)throw exception;if(error instanceof Error fatal)throw fatal;throw new IllegalStateException(error);
 }
 static String stagedSnapshot(GameHub hub)throws Exception {
  // Copy immutable key/value references under the hub monitor. Release it
  // before observing any inventory; never nest hub and inventory monitors.
  var references=new ArrayList<Map.Entry<?,?>>();
  synchronized(hub) {
   for(var e:((Map<?,?>)NativePeerHarness.field(hub,"rentalBattleHandoffs")).entrySet())
    references.add(new AbstractMap.SimpleImmutableEntry<>(e.getKey(),e.getValue()));
  }
  long now=System.nanoTime();var rows=new ArrayList<String>();
  for(var entry:references) {
   Object value=entry.getValue();long expiry=(long)NativePeerHarness.field(value,"expiresAtNanos");
   var inventory=(RuntimeInventory)NativePeerHarness.field(value,"inventory");var ids=new ArrayList<String>();
   for(Long item:NativeRecoveryReadonlySnapshot.inventoryIds(inventory))ids.add(String.valueOf(item));
   String character=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(String.valueOf(NativePeerHarness.field(value,"character")).getBytes(StandardCharsets.UTF_8)));
   String key=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(String.valueOf(entry.getKey()).getBytes(StandardCharsets.UTF_8)));
   rows.add("{\"key_sha256\":\""+key+"\",\"character_sha256\":\""+character+"\",\"expires_at_nanos\":"+expiry+",\"expired\":"+(now>=expiry)+",\"inventory_ids\":["+String.join(",",ids)+"]}");
  }
  Collections.sort(rows);return "{\"observed_now_nanos\":"+now+",\"staged\":["+String.join(",",rows)+"]}";
 }
 public static void main(String[] args)throws Exception {
  require(args.length==8&&args[7].equals("--execute-owned-tests"),"requires root, scenario, actual owner/import files+hashes, static SQL and explicit execution flag");
  Path root=Path.of(args[0]).toRealPath();require(root.startsWith(REBUILD.toRealPath()),"fixture outside rebuild root");
  require(!Files.exists(root.resolve("ready.json"))&&!Files.exists(root.resolve("result.json")),"preserve earlier attempt; use a fresh owned case");
  Path ownerFile=Path.of(args[2]).toRealPath(),importFile=Path.of(args[4]).toRealPath(),staticSQL=Path.of(args[6]).toRealPath();
  require(ownerFile.startsWith(root)&&importFile.startsWith(root),"proofs outside owned case");
  require(hash(ownerFile).equals(args[3])&&hash(importFile).equals(args[5]),"proof hash mismatch");
  String sha=hash(Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI()));
  require(sha.equals(FORMAL),"formal JAR changed");
  boolean gui=args[1].startsWith("gui-");String scenario=args[1].replaceFirst("^gui-","");
  require(Set.of("timeout","refused","wrong-service","cancel-retry","cancel-late","cancel-expiry","cancel-repeat","refused-cancel-repeat","cancel-expiry-source").contains(scenario),"unknown recovery scenario");
  boolean cancelExpiry=scenario.equals("cancel-expiry")||scenario.equals("cancel-expiry-source");
  boolean refused=scenario.equals("refused")||scenario.equals("refused-cancel-repeat"),wrongService=scenario.equals("wrong-service");
  int expectedSessions=wrongService?10:8;
  // Reserve exact ports while configuration and resources are initialized.
  // These sockets never serve protocol data. Release each only immediately
  // before the original native bind; a collision fails rather than rebinding.
  try(ServerSocket loginReservation=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));
      ServerSocket townReservation=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"));
      ServerSocket unavailableReservation=new ServerSocket(0,4,InetAddress.getByName(refused?"127.0.0.1":"127.0.0.3"));
      ServerSocket login=new ServerSocket();ServerSocket town=new ServerSocket();ServerSocket unavailable=new ServerSocket()) {
   int loginPort=loginReservation.getLocalPort(),townPort=townReservation.getLocalPort(),unavailablePort=unavailableReservation.getLocalPort();
   if(refused) {
    unavailableReservation.close();
    try(Socket probe=new Socket()) {
     probe.connect(new InetSocketAddress("127.0.0.1",unavailablePort),5000);
     throw new IllegalStateException("closed destination unexpectedly accepted a connection");
    }catch(ConnectException expected){/* Only an actual refusal qualifies this future scenario. */}
   }
   Path file=root.resolve("server.properties");
   NativePeerHarness.write(file,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+townPort+"\nadvertised.host=127.0.0.2\nadvertised.port="+townPort+
    "\nserver.name=傳送失敗測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
    "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
    "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=true\nmax.frame=65535\n"+
    "lito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+townPort+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n"+
    "lito.rental.teleport.routes=TeleportMe|LT_TELEPORT_B|Unavailable B|"+(refused?"127.0.0.1":"127.0.0.3")+"|"+unavailablePort+",TeleportMe|LT_TELEPORT_C|Unavailable C|"+(refused?"127.0.0.1":"127.0.0.3")+"|"+unavailablePort+"\n");
   ServerConfig config=ServerConfig.load(file);
   require(config.databaseMode().equals("memory")&&config.databaseReadOnly()&&!config.statePersistenceEnabled(),"fixture account isolation");
   var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));routes.put("測試乙",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
   AccountRepository accounts=new SourceRoutedAccountRepository(routes);LitoMapRepository maps=null;CaptureLog capture=null;
   NativeRecoveryProductionBootstrap app=null;boolean bootstrapOwnsResources=false;
   var sessions=new CopyOnWriteArrayList<NativeRecoveryProductionBootstrap.Session>();
   var failures=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();var stopping=new AtomicBoolean();
   Throwable problem=null;int staged=-1;boolean flowComplete=false;
   try {
    maps=LitoMapRepository.open(config);capture=new CaptureLog(config.captureDirectory());
    bootstrapOwnsResources=true;
    app=NativeRecoveryProductionBootstrap.open(root,config,accounts,maps,capture,ownerFile,args[3],importFile,args[5],staticSQL);
    maps.healthCheck();
    loginReservation.close();app.bindLogin(login,loginPort,4);
    townReservation.close();app.bindGame(town,"127.0.0.2",townPort,4);
    if(!refused){unavailableReservation.close();app.bindGame(unavailable,"127.0.0.3",unavailablePort,4);}
    long acceptTimeout=scenario.equals("cancel-expiry-source")?420000:(cancelExpiry||gui)?180000:90000;
    login.setSoTimeout((int)acceptTimeout);town.setSoTimeout((int)acceptTimeout);if(!refused)unavailable.setSoTimeout(90000);
    final var application=app;final GameHub hub=app.hub;
    for(int endpoint=0;endpoint<(wrongService?3:2);endpoint++) {
     final int kind=endpoint,count=endpoint==2?2:4;final boolean game=endpoint==1;
     Thread acceptor=new Thread(()->{
      try {ServerSocket listener=kind==2?unavailable:game?town:login;
       for(int n=0;n<count;n++)sessions.add(application.accept(listener,game));
      }catch(Throwable error){if(!stopping.get())failures.add(error);}
     },"G06-native-acceptor-"+endpoint);acceptors.add(acceptor);acceptor.start();
    }
    NativePeerHarness.write(root.resolve("ready.json"),"{\"observer_kind\":\"G06_E149_resources_direct_fields_v2\",\"trace_observer_kind\":\"G06_E149_direct_fields_v1\",\"original_resource_factories_initialized\":true,\"native_server_lifetime_installed\":true,\"static_sql_access\":true,\"player_sql_access\":false,\"static_tables\":19,\"static_rows\":6277,\"full_main_process_qualified\":false,\"owner_proof_sha256\":\""+args[3]+"\",\"import_proof_sha256\":\""+args[5]+"\",\"requested_source\":\"測試乙\",\"default_source\":\"測試甲\",\"scenario\":\""+scenario+"\",\"port\":"+loginPort+",\"destination_port\":"+unavailablePort+",\"destination_refused\":"+refused+",\"destination_wrong_service\":"+wrongService+",\"jar_sha256\":\""+sha.toLowerCase(Locale.ROOT)+"\"}");
    long until=System.nanoTime()+(scenario.equals("cancel-expiry-source")?440_000_000_000L:(cancelExpiry||gui)?320_000_000_000L:200_000_000_000L);
    var previous=new IdentityHashMap<ClientSession,String>();
    try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"));var stages=Files.newBufferedWriter(root.resolve("staged-trace.jsonl"))) {
     while(System.nanoTime()<until) {
      require(failures.isEmpty(),"native acceptor failed; inspect suppressed original failure");
      stages.write(stagedSnapshot(hub)+"\n");stages.flush();
      for(var session:sessions) {
       if(session.complete())session.await(1); // propagate original worker failure
       if(session.game()) {
        String row=NativeRecoveryReadonlySnapshot.observe(session.owner(),hub);
        if(!row.equals(previous.put(session.owner(),row))){trace.write(row+"\n");trace.flush();}
        // The controller pauses only its test driver after receiving the
        // actual continued inventory. Acknowledge an actual stable selected
        // observation; do not let a 20ms sampling race lose this phase.
        for(String role:List.of("a","b")) {
         String account=role.equals("a")?"PeerFixture":"PeerFixture2";
         Path request=root.resolve(role+"-continuation-observe-request");
         Path observed=root.resolve(role+"-continuation-observed.json");
         if(Files.exists(request)&&!Files.exists(observed)
            &&row.endsWith(",\"stable_observed\":true,\"full_native_owner_qualification_inferred\":false}")) {
          ClientSession current=session.owner();Object owner=NativePeerHarness.field(current,"sourceItemOwner");
          String character=String.valueOf(NativePeerHarness.field(current,"selectedCharacter"));
          if(account.equals(NativePeerHarness.field(current,"account"))
             &&"LOBBY".equals(String.valueOf(NativePeerHarness.field(current,"phase")))
             &&"測試乙".equals(NativePeerHarness.field(current,"selectedClassicServerName"))
             &&owner!=null&&Boolean.TRUE.equals(NativePeerHarness.field(owner,"active"))
             &&Files.readString(request,StandardCharsets.UTF_8).equals(account+"\n"+character+"\n測試乙")) {
           trace.write(row+"\n");trace.flush();NativePeerHarness.write(observed,row);
          }
         }
        }
       }
      }
      if(sessions.size()==expectedSessions&&acceptors.stream().noneMatch(Thread::isAlive)&&sessions.stream().allMatch(NativeRecoveryProductionBootstrap.Session::complete))break;
      Thread.sleep(20);
     }
    }
    require(sessions.size()==expectedSessions&&sessions.stream().allMatch(NativeRecoveryProductionBootstrap.Session::complete),"session count/exit barrier");
    for(int channel=0;channel<4;channel++)require(hub.channelSnapshot(channel).users()==0,"channel leak");
    synchronized(hub){staged=((Map<?,?>)NativePeerHarness.field(hub,"rentalBattleHandoffs")).size();}
    require(staged==0,"unconsumed loadout after recovery");flowComplete=true;
   }catch(Throwable error){problem=error;}
   finally {
    stopping.set(true);
    if(app!=null)try{app.close();}catch(Throwable error){problem=append(problem,error);}
    else if(!bootstrapOwnsResources)for(AutoCloseable resource:Arrays.asList(capture,maps,accounts))if(resource!=null)
     try{resource.close();}catch(Throwable error){problem=append(problem,error);}
    for(ServerSocket listener:List.of(login,town,unavailable))try{listener.close();}catch(Throwable error){problem=append(problem,error);}
    for(var session:sessions)try{session.socket().close();}catch(Throwable error){problem=append(problem,error);}
    for(Thread acceptor:acceptors)try{acceptor.join(3000);require(!acceptor.isAlive(),"acceptor leak");}catch(Throwable error){problem=append(problem,error);}
    for(var session:sessions)try{session.await(5);}catch(Throwable error){problem=append(problem,error);}
    for(Throwable error:failures)problem=append(problem,error);
   }
   if(problem!=null){try(var failureWriter=new PrintWriter(Files.newBufferedWriter(root.resolve("original-fixture-failure.txt")))){problem.printStackTrace(failureWriter);}rethrow(problem);}
   require(flowComplete,"flow did not complete");
   NativePeerHarness.write(root.resolve("result.json"),"{\"observer_direct_fields\":true,\"original_resource_factories_initialized\":true,\"native_server_lifetime_installed\":true,\"native_owner_runtime_qualified\":false,\"status\":\"sessions_closed\",\"actual_client_sessions\":"+expectedSessions+",\"memberships_empty\":true,\"authority_mutations\":0,\"static_sql_access\":true,\"player_sql_access\":false,\"state_persistent\":false,\"staged_loadouts_after_reentry\":"+staged+",\"original_futures_completed\":true,\"native_cleanup_completed\":true,\"controller_wire_and_full_owner_validation_required\":true,\"full_main_process_qualified\":false,\"jar_sha256\":\""+sha.toLowerCase(Locale.ROOT)+"\"}");
  }
 }
}
