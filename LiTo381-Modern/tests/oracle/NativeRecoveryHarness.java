package local.lito381.server;

import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.*;
import java.security.MessageDigest;
import java.util.*;

/** Close an owned GAME socket, then accept fresh LOGIN/GAME on the same hub. */
public final class NativeRecoveryHarness {
    static void write(Path p,String s)throws Exception{Files.writeString(p,s,StandardCharsets.UTF_8);}
    static Object field(Object o,String n)throws Exception{Field f=o.getClass().getDeclaredField(n);f.setAccessible(true);return f.get(o);}
    static void empty(GameHub hub)throws Exception{
        for(int c=0;c<4;c++){
            for(var room:hub.roomSnapshots(c))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room leaked");
            if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("channel leaked");
        }
        if(hub.pendingClassicSourceHandoff("RecoveryFixture")!=null)throw new IllegalStateException("handoff leaked");
    }
    static void awaitEmpty(GameHub hub)throws Exception{
        long end=System.nanoTime()+3_000_000_000L;
        while(true){try{empty(hub);return;}catch(IllegalStateException e){if(System.nanoTime()>=end)throw e;Thread.sleep(20);}}
    }
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
        String mode=args[1];if(!Set.of("room","world").contains(mode))throw new IllegalArgumentException("mode");
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket login=new ServerSocket(0,2,InetAddress.getByName("127.0.0.1"));
            ServerSocket game=new ServerSocket(0,2,InetAddress.getByName("127.0.0.2"))){
            login.setSoTimeout(60000);game.setSoTimeout(60000);
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=false\nmax.frame=65535\n");
            ServerConfig config=ServerConfig.load(configPath);
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation");
            for(RecordComponent f:ServerConfig.class.getRecordComponents())if(f.getType()==Path.class){Path p=(Path)f.getAccessor().invoke(config);if(p!=null&&!p.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            var routes=new LinkedHashMap<String,AccountRepository>();
            for(String name:List.of("測試甲","測試乙"))routes.put(name,new MemoryAccountRepository("RecoveryFixture","TestOnlyPassword"));
            MemoryAccountRepository memoryBacking=(MemoryAccountRepository)routes.get("測試乙");
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,memoryBacking)){
                write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"jar_sha256\":\""+sha+"\"}");
                List<String> observations=new ArrayList<>();
                for(int cycle=0;cycle<2;cycle++){
                    try(Socket loginSocket=login.accept()){
                        ClientSession lc=new ClientSession(loginSocket,ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
                        Thread lt=new Thread(lc,"recovery-login-"+cycle);lt.start();
                        try(Socket gameSocket=game.accept()){
                            ClientSession gc=new ClientSession(gameSocket,ClientSession.Endpoint.GAME,config,accounts,state,capture,hub);
                            Thread gt=new Thread(gc,"recovery-game-"+cycle);gt.start();
                            try{
                                String signal=cycle==0?"drop-ready":"recovered";
                                long end=System.nanoTime()+65_000_000_000L;
                                while(!Files.exists(root.resolve(signal)) && gt.isAlive() && System.nanoTime()<end)Thread.sleep(10);
                                if(!Files.exists(root.resolve(signal)))throw new IllegalStateException("missing "+signal);
                                String phase=String.valueOf(field(gc,"phase"));
                                if(!phase.equals(mode.equals("world")?"BATTLE":"ROOM"))throw new IllegalStateException("unexpected phase "+phase);
                                observations.add("{\"cycle\":"+cycle+",\"phase\":\""+phase+"\",\"id\":"+gc.playerObjectId()+
                                    ",\"room\":"+field(gc,"currentRoomId")+",\"x\":"+field(gc,"positionX")+",\"y\":"+field(gc,"positionY")+
                                    ",\"hp\":"+field(gc,"playerHp")+",\"mp\":"+field(gc,"playerMp")+"}");
                                if(cycle==0){gameSocket.close();gt.join(5000);lt.join(5000);if(gt.isAlive()||lt.isAlive())throw new IllegalStateException("drop worker leaked");awaitEmpty(hub);write(root.resolve("drop-closed"),"closed");}
                                else {write(root.resolve("recovery-observed"),"observed");gt.join(8000);lt.join(5000);if(gt.isAlive()||lt.isAlive())throw new IllegalStateException("recovered worker leaked");awaitEmpty(hub);}
                            }finally{gameSocket.close();gt.join(3000);}
                        }finally{loginSocket.close();lt.join(3000);}
                    }
                }
                write(root.resolve("result.json"),"{\"status\":\"reconnected\",\"jar_sha256\":\""+sha+"\",\"mode\":\""+mode+
                    "\",\"actual_client_sessions\":4,\"socket_faults\":1,\"authority_mutations\":0,\"sql_access\":false,\"memberships_empty\":true,\"observations\":["+String.join(",",observations)+"]}");
            }
        }
    }
}
