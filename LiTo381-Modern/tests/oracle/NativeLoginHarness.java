package local.lito381.server;

import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.*;
import java.security.MessageDigest;
import java.util.*;

/** Runs the pinned deployed ClientSession, not a reimplementation of its login.
 * Only new MemoryAccountRepository instances and a non-persistent state store. */
public final class NativeLoginHarness {
    private static void write(Path path,String text)throws Exception{Files.writeString(path,text,StandardCharsets.UTF_8);}
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();String mode=args[1];
        if(!Set.of("accept","reject","retry").contains(mode))throw new IllegalArgumentException("mode");
        Files.createDirectories(root);
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket listener=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))){
            listener.setSoTimeout(20000);int port=listener.getLocalPort();
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+port+
                "\nadvertised.host=127.0.0.2\nadvertised.port="+port+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\n"+
                "database.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\n"+
                "database.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\n"+
                "lito.rental.town.directory.enabled=false\nmax.frame=65535\n");
            ServerConfig config=ServerConfig.load(configPath);
            if(!config.databaseMode().equals("memory") || !config.databaseReadOnly() || config.statePersistenceEnabled())throw new IllegalStateException("isolation config");
            for(RecordComponent field:ServerConfig.class.getRecordComponents())if(field.getType()==Path.class){
                Path value=(Path)field.getAccessor().invoke(config);
                if(value!=null && !value.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("path outside isolated root: "+field.getName());
            }
            LinkedHashMap<String,AccountRepository> routes=new LinkedHashMap<>();
            routes.put("測試甲",new MemoryAccountRepository("重建帳號","測試密碼"));
            MemoryAccountRepository memoryBacking=new MemoryAccountRepository("重建帳號","測試密碼");
            routes.put("測試乙",memoryBacking);
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);
                CaptureLog capture=new CaptureLog(config.captureDirectory());
                GameHub hub=new GameHub(config,state,memoryBacking)) {
                write(root.resolve("ready.json"),"{\"port\":"+port+",\"jar_sha256\":\""+sha+"\"}");
                try(Socket socket=listener.accept()){
                    ClientSession session=new ClientSession(socket,ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
                    Thread worker=new Thread(session,"isolated-deployed-client-session");worker.start();worker.join(25000);
                    if(worker.isAlive()){socket.close();worker.join(3000);throw new IllegalStateException("deployed session did not stop");}
                    Field phaseField=ClientSession.class.getDeclaredField("phase");phaseField.setAccessible(true);
                    String phase=String.valueOf(phaseField.get(session));
                    Field selected=ClientSession.class.getDeclaredField("selectedClassicServerName");selected.setAccessible(true);
                    String source=(String)selected.get(session);
                    boolean expected=mode.equals("reject")?phase.equals("VERSION_OK"):phase.equals("LOBBY")&&source.equals("測試乙");
                    if(!expected)throw new IllegalStateException("unexpected deployed phase/source: "+phase+"/"+source);
                    write(root.resolve("result.json"),"{\"status\":\"verified\",\"mode\":\""+mode+"\",\"phase\":\""+phase+
                        "\",\"source\":\""+source+"\",\"jar_sha256\":\""+sha+"\",\"actual_client_session\":true,\"sql_access\":false,\"state_persistent\":false}");
                }
            }
        }
    }
}
