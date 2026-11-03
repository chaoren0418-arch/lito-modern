package local.lito381.server;

import java.net.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.concurrent.*;

/** Owned loopback sockets; actual formal sessions and passive observations only. */
public final class NativeTeleportHarness {
    public static void main(String[] args)throws Exception {
        boolean same=args.length>1 && args[1].equals("same-endpoint");
        boolean gui=args.length>1 && args[1].equals("gui-cross-endpoint");
        Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));
            ServerSocket town=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"));
            ServerSocket b=new ServerSocket(0,4,InetAddress.getByName("127.0.0.3"));
            ServerSocket c=new ServerSocket(0,4,InetAddress.getByName("127.0.0.4"))) {
            List<ServerSocket> sockets=List.of(login,town,b,c);
            for(var socket:sockets)socket.setSoTimeout(gui?240000:60000);
            Path configFile=root.resolve("server.properties");
            NativePeerHarness.write(configFile,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+town.getLocalPort()+
                "\nadvertised.host=127.0.0.2\nadvertised.port="+town.getLocalPort()+
                "\nserver.name=重建傳送測試\nlogin.classic.server.names=測試乙,測試甲\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=true\nmax.frame=65535\n"+
                "lito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+town.getLocalPort()+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n"+
                "lito.rental.teleport.routes=TeleportMe|LT_TELEPORT_B|Owned B|"+(same?"127.0.0.2":"127.0.0.3")+"|"+(same?town.getLocalPort():b.getLocalPort())+",TeleportMe|LT_TELEPORT_C|Owned C|"+(same?"127.0.0.2":"127.0.0.4")+"|"+(same?town.getLocalPort():c.getLocalPort())+"\n");
            ServerConfig config=ServerConfig.load(configFile);
            var configurations=new ArrayList<ServerConfig>();configurations.add(config);configurations.add(config);
            for(int i=2;i<4;i++){
                Path path=root.resolve("destination-"+i+".properties");
                NativePeerHarness.write(path,Files.readString(configFile)
                    .replace("lito.rental.town.directory.enabled=true","lito.rental.town.directory.enabled=false")
                    .replace("advertised.host=127.0.0.2","advertised.host=127.0.0."+(i+1))
                    .replace("advertised.port="+town.getLocalPort(),"advertised.port="+sockets.get(i).getLocalPort()));
                configurations.add(ServerConfig.load(path));
            }
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation");
            for(var component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path path=(Path)component.getAccessor().invoke(config);if(path!=null&&!path.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            var memoryBacking=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
            var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",memoryBacking);
            routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            // Both players use source 0; the hub shares its initial memory child.
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,memoryBacking)) {
                var sessions=new CopyOnWriteArrayList<NativePeerHarness.Session>();var failures=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();
                for(int i=0;i<sockets.size();i++){
                    int endpoint=i;ServerSocket socket=sockets.get(i);int count=same?(i==0?2:i==1?4:0):(i<2?2:1);
                    Thread acceptor=new Thread(()->{try{for(int n=0;n<count;n++){
                        Socket connection=socket.accept();ClientSession owner=new ClientSession(connection,endpoint==0?ClientSession.Endpoint.LOGIN:ClientSession.Endpoint.GAME,configurations.get(endpoint),accounts,state,capture,hub);
                        Thread worker=new Thread(owner,"teleport-session-"+endpoint+"-"+n);worker.setUncaughtExceptionHandler((thread,error)->failures.add(error));
                        sessions.add(new NativePeerHarness.Session(connection,owner,worker,endpoint!=0));worker.start();
                    }}catch(Throwable error){failures.add(error);}},"teleport-acceptor-"+i);acceptors.add(acceptor);acceptor.start();
                }
                try {
                    NativePeerHarness.write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"town_port\":"+town.getLocalPort()+",\"b_port\":"+(same?town.getLocalPort():b.getLocalPort())+",\"c_port\":"+(same?town.getLocalPort():c.getLocalPort())+",\"same_endpoint\":"+same+",\"jar_sha256\":\""+sha+"\"}");
                    var previous=new IdentityHashMap<ClientSession,String>();long until=System.nanoTime()+(gui?300_000_000_000L:120_000_000_000L);
                    try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"))){
                        while(System.nanoTime()<until){
                            if(!failures.isEmpty())throw new IllegalStateException("session failure",failures.peek());
                            for(var session:sessions)if(session.game()){
                                String row=NativePeerHarness.observe(session,hub);
                                if(!row.equals(previous.put(session.owner(),row))){trace.write(row+"\n");trace.flush();}
                            }
                            if(sessions.size()==6&&acceptors.stream().noneMatch(Thread::isAlive)&&sessions.stream().noneMatch(s->s.thread().isAlive()))break;
                            Thread.sleep(20);
                        }
                    }
                    if(sessions.size()!=6||sessions.stream().anyMatch(s->s.thread().isAlive()))throw new IllegalStateException("session cleanup");
                    for(int channel=0;channel<4;channel++){
                        if(hub.channelSnapshot(channel).users()!=0)throw new IllegalStateException("channel leak");
                        for(var room:hub.roomSnapshots(channel))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room leak");
                    }
                    if(!((Map<?,?>)NativePeerHarness.field(hub,"rentalBattleHandoffs")).isEmpty())throw new IllegalStateException("loadout leak");
                    if(!((Map<?,?>)NativePeerHarness.field(hub,"authenticatedGameAccounts")).isEmpty())throw new IllegalStateException("account ownership leak");
                    NativePeerHarness.write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":6,\"sql_access\":false,\"authority_mutations\":0,\"memberships_empty\":true,\"staged_loadouts_empty\":true,\"authenticated_accounts_empty\":true,\"state_persistent\":false,\"jar_sha256\":\""+sha+"\"}");
                }finally {
                    for(var socket:sockets)socket.close();for(var session:sessions)session.socket().close();
                    for(var acceptor:acceptors)acceptor.join(3000);for(var session:sessions){session.thread().join(3000);if(session.thread().isAlive())throw new IllegalStateException("worker leak");}
                }
            }
        }
    }
}
