package local.lito381.server;

import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.RecordComponent;
import java.security.MessageDigest;
import java.util.*;
import java.util.concurrent.*;

/** Original C02 sessions, default memory accounts; no authority or clock edits. */
public final class NativeEnvironmentHarness {
    record Session(Socket socket, ClientSession owner, Thread worker, boolean game) {}
    static void write(Path path,String value)throws Exception {Files.writeString(path,value,StandardCharsets.UTF_8);}
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        if(!sha.equalsIgnoreCase("C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539"))throw new IllegalStateException("Unreviewed ClientSession JAR");
        try(ServerSocket login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));
            ServerSocket game=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"))){
            login.setSoTimeout(600000);game.setSoTimeout(600000);
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nmax.frame=65535\nbattle.ai.enabled=false\n"+
                "lito.rental.town.directory.enabled=true\nlito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+game.getLocalPort()+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n");
            ServerConfig config=ServerConfig.load(configPath);
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation config");
            for(RecordComponent component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path value=(Path)component.getAccessor().invoke(config);if(value!=null&&!value.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            var first=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
            var second=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
            var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試甲",first);routes.put("測試乙",second);
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,second)){
                var sessions=new CopyOnWriteArrayList<Session>();var failures=new ConcurrentLinkedQueue<Throwable>();var acceptors=new ArrayList<Thread>();
                for(boolean isGame:new boolean[]{false,true}){
                    ServerSocket listener=isGame?game:login;
                    Thread acceptor=new Thread(()->{try {
                        for(int i=0;i<2;i++){
                            Socket socket=listener.accept();
                            ClientSession c=new ClientSession(socket,isGame?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
                            Thread worker=new Thread(c,"environment-"+(isGame?"GAME":"LOGIN")+i);worker.setUncaughtExceptionHandler((ignored,error)->failures.add(error));
                            sessions.add(new Session(socket,c,worker,isGame));worker.start();
                        }
                    }catch(Throwable e){failures.add(e);}},"environment-acceptor-"+isGame);
                    acceptors.add(acceptor);acceptor.start();
                }
                try {
                    write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"game_port\":"+game.getLocalPort()+",\"jar_sha256\":\""+sha+"\"}");
                    long until=System.nanoTime()+600_000_000_000L;
                    while(System.nanoTime()<until){
                        if(!failures.isEmpty())throw new IllegalStateException("original session failed",failures.peek());
                        if(sessions.size()==4&&acceptors.stream().noneMatch(Thread::isAlive)&&sessions.stream().noneMatch(s->s.worker().isAlive()))break;
                        Thread.sleep(20);
                    }
                    if(sessions.size()!=4||sessions.stream().anyMatch(s->s.worker().isAlive()))throw new IllegalStateException("owned sessions did not close");
                    for(int channel=0;channel<4;channel++){
                        for(var room:hub.roomSnapshots(channel))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room membership leaked");
                        if(hub.channelSnapshot(channel).users()!=0)throw new IllegalStateException("channel membership leaked");
                    }
                    if(hub.pendingClassicSourceHandoff("PeerFixture")!=null)throw new IllegalStateException("handoff leaked");
                    write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":4,\"LOGIN_sessions\":2,\"GAME_sessions\":2,\"memberships_empty\":true,\"authority_mutations\":0,\"sql_access\":false,\"state_persistent\":false,\"jar_sha256\":\""+sha+"\"}");
                } finally {
                    login.close();game.close();for(Session s:sessions)s.socket().close();
                    for(Thread acceptor:acceptors)acceptor.join(3000);
                    for(Session s:sessions){s.worker().join(3000);if(s.worker().isAlive())throw new IllegalStateException("owned worker leaked");}
                }
            }
        }
    }
}
