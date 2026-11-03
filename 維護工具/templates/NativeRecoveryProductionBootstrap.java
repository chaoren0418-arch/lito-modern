package local.lito381.server;

import java.io.*;
import java.lang.reflect.Field;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.security.MessageDigest;
import java.sql.*;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;

/** Development adapter: original E149 factories over an owned static-only DB.
 * No main method. Compiling this class does not start a database or application.
 * The launcher must still create/import/observe the isolated database for real.
 */
final class NativeRecoveryProductionBootstrap implements AutoCloseable {
    static final String FORMAL_SHA="E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498";
    static final String STATIC_SQL_SHA="C1E29B0479FE40476AD53B4E52498451C3A1E469A27483EABE67FA880EBCA7DD";
    static final String DUMP_SHA="D4111D8AEFF50EA5D9D36630C7381132812F7F727E8432FFAF035B89C9CBB075";
    static final String JDBC_OPTIONS="?useUnicode=true&characterEncoding=UTF-8&useSSL=false&connectTimeout=2000&socketTimeout=8000&allowMultiQueries=false&allowLoadLocalInfile=false";
    private static final Path REBUILD_ROOT=Path.of("D:/lineage開發/3-lito開發/lito重建版").toAbsolutePath().normalize();
    final GameHub hub;
    final GameStateStore state;
    final ServerConfig config;
    final AccountRepository accounts;
    final LitoMapRepository maps;
    final CaptureLog capture;
    final ExecutorService clients;
    final List<ServerSocket> listeners;
    private final Thread installingThread;
    private boolean closed;

    record Session(Socket socket,ClientSession owner,Future<?> future,boolean game) {
        boolean complete(){return future.isDone();}
        void await(long seconds)throws Exception{future.get(seconds,TimeUnit.SECONDS);}
    }
    private NativeRecoveryProductionBootstrap(GameHub hub,GameStateStore state,ServerConfig config,
            AccountRepository accounts,LitoMapRepository maps,CaptureLog capture,
            ExecutorService clients,List<ServerSocket> listeners) {
        this.hub=hub;this.state=state;this.config=config;this.accounts=accounts;
        this.maps=maps;this.capture=capture;this.clients=clients;this.listeners=listeners;
        this.installingThread=Thread.currentThread();
    }
    private static void require(boolean condition,String message) {
        if(!condition)throw new IllegalStateException(message);
    }
    private static String hash(Path file)throws Exception {
        return HexFormat.of().withUpperCase().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(file)));
    }
    private static Path within(Path file,Path root)throws Exception {
        Path real=file.toRealPath();require(real.startsWith(root.toRealPath()),"path outside owned root");return real;
    }
    private static Properties properties(Path file)throws Exception {
        var p=new Properties();try(var in=Files.newBufferedReader(file,StandardCharsets.UTF_8)){p.load(in);}return p;
    }
    private static String needed(Properties p,String key) {
        String value=p.getProperty(key);require(value!=null&&!value.isBlank(),"missing proof field "+key);return value;
    }
    private static Object field(Object value,String name)throws Exception {
        Field field=value.getClass().getDeclaredField(name);field.setAccessible(true);return field.get(value);
    }
    private static String argument(String[] arguments,String key) {
        List<String> found=Arrays.stream(arguments).filter(s->s.startsWith(key+"=")).toList();
        require(found.size()==1,"owned database argument missing or duplicated: "+key);
        return found.get(0).substring(key.length()+1);
    }
    private static Properties verifyDatabase(Path root,Path ownerFile,String ownerSHA,
            Path importFile,String importSHA,Path staticSQL)throws Exception {
        root=within(root,REBUILD_ROOT);ownerFile=within(ownerFile,root);importFile=within(importFile,root);
        require(hash(ownerFile).equals(ownerSHA)&&hash(importFile).equals(importSHA),"database proof changed");
        require(hash(within(staticSQL,REBUILD_ROOT)).equals(STATIC_SQL_SHA),"static SQL differs from pinned 19-table extraction");
        Properties owner=properties(ownerFile),imported=properties(importFile);
        long pid=Long.parseLong(needed(owner,"process.pid"));long birth=Long.parseLong(needed(owner,"process.start.epoch.millis"));
        int port=Integer.parseInt(needed(owner,"port"));require(port>=49152&&port<=65535,"database must use its own ephemeral port");
        String nonce=needed(owner,"nonce");require(nonce.matches("[0-9a-f]{12}"),"invalid database namespace");
        String schema="lito_rebuild_e149_"+nonce;require(needed(owner,"schema").equals(schema),"database schema outside fixture namespace");
        require(needed(owner,"user").equals("lg06_"+nonce.substring(0,8)),"database reader must be the isolated static-only user");
        needed(owner,"password");
        Path executable=within(Path.of(needed(owner,"executable")),REBUILD_ROOT);
        require(Set.of("mysqld.exe","mariadbd.exe").contains(executable.getFileName().toString().toLowerCase(Locale.ROOT)),"unexpected database executable");
        require(hash(executable).equals(needed(owner,"executable.sha256")),"database executable changed");
        Path datadir=within(Path.of(needed(owner,"datadir")),root);
        ProcessHandle process=ProcessHandle.of(pid).orElseThrow(()->new IllegalStateException("owned database is not live"));
        ProcessHandle.Info info=process.info();require(process.isAlive(),"owned database ended");
        require(info.startInstant().orElseThrow().toEpochMilli()==birth,"database PID was reused");
        require(Path.of(info.command().orElseThrow()).toRealPath().equals(executable),"database process image mismatch");
        String[] arguments=info.arguments().orElseThrow();
        require(Path.of(argument(arguments,"--datadir")).toRealPath().equals(datadir),"database data directory mismatch");
        require(argument(arguments,"--port").equals(Integer.toString(port))&&argument(arguments,"--bind-address").equals("127.0.0.1"),"database bind mismatch");
        require(Arrays.stream(arguments).noneMatch(s->s.startsWith("--init-file")||s.startsWith("--init_file")),"unreviewed database init script");
        // Read-only OS observation. Never stop or modify any database here.
        String ps="$ErrorActionPreference='Stop';$rows=@(Get-NetTCPConnection -State Listen -LocalPort "+port+");if($rows.Count -eq 0 -or @($rows|Where-Object {$_.OwningProcess -ne "+pid+" -or $_.LocalAddress -ne '127.0.0.1'}).Count -ne 0){exit 1}";
        Process probe=new ProcessBuilder("pwsh","-NoProfile","-NonInteractive","-Command",ps).redirectErrorStream(true).start();
        if(!probe.waitFor(5,TimeUnit.SECONDS)){probe.destroyForcibly();probe.waitFor();throw new IllegalStateException("database listener observation timed out");}
        require(probe.exitValue()==0,"listener does not belong to the exact owned loopback process");
        for(String key:List.of("process.pid","process.start.epoch.millis","port","schema"))
            require(needed(imported,key).equals(needed(owner,key)),"static import belongs to another database");
        require(needed(imported,"status").equals("imported")&&needed(imported,"sql.sha256").equals(STATIC_SQL_SHA)
            &&needed(imported,"source.dump.sha256").equals(DUMP_SHA)&&needed(imported,"tables").equals("19")
            &&needed(imported,"rows").equals("6277")&&needed(imported,"statements").equals("6296"),"original static import is not evidenced");
        require(process.isAlive()&&process.info().startInstant().orElseThrow().toEpochMilli()==birth,"database ended during observation");
        return owner;
    }
    /** Ownership of accounts/maps/capture transfers only after native install.
     * Before then the caller retains cleanup responsibility. No listeners are
     * created or bound until both original resource readers have completed.
     */
    static NativeRecoveryProductionBootstrap open(Path root,ServerConfig config,
            AccountRepository accounts,LitoMapRepository maps,CaptureLog capture,
            Path ownerFile,String ownerSHA,Path importFile,String importSHA,Path staticSQL)throws Exception {
        root=within(root,REBUILD_ROOT);within(config.source(),root);
        require(config.databaseMode().equals("memory")&&config.databaseReadOnly()&&!config.statePersistenceEnabled()
            &&!config.classicWorldCompatibilityEnabled(),"fixture identities must remain isolated memory accounts");
        require(accounts.getClass()==SourceRoutedAccountRepository.class,"expected original source-routed memory repository");
        for(String source:config.classicServerNames())
            require(accounts.repositoryForClassicSource(source).getClass()==MemoryAccountRepository.class,"real or substituted account repository is forbidden");
        for(var component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class) {
            Path path=(Path)component.getAccessor().invoke(config);
            if(path!=null)require(path.toAbsolutePath().normalize().startsWith(root),"configuration path outside fixture root");
        }
        Path jar=Path.of(NativeDeItemApplication.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        require(hash(jar).equals(FORMAL_SHA),"resource factory is not the fixed E149 reference");
        Properties db=verifyDatabase(root,ownerFile,ownerSHA,importFile,importSHA,staticSQL);
        ExecutorService clients=Executors.newCachedThreadPool();List<ServerSocket> listeners=new ArrayList<>();
        GameStateStore state=new GameStateStore(config.stateFile(),config.statePersistenceEnabled(),config.ephemeralSourceProfiles());
        GameHub hub=null;NativeRecoveryProductionBootstrap application=null;
        try {
            hub=new GameHub(config,state,accounts);
            NativeServerLifetime.install(hub,clients,listeners,capture,maps,accounts);
            application=new NativeRecoveryProductionBootstrap(hub,state,config,accounts,maps,capture,clients,listeners);
            var opened=new ArrayList<Connection>();
            NativeDeTerrainScope.Sql sql=()->{
                Connection connection=DriverManager.getConnection("jdbc:mysql://127.0.0.1:"+db.getProperty("port")+"/"+db.getProperty("schema")+JDBC_OPTIONS,
                    db.getProperty("user"),db.getProperty("password"));opened.add(connection);return connection;
            };
            NativeDeWorldLog log=(level,format,values)->System.getLogger("LiTo.G06.isolated.resources").log(
                level.equals("error")?System.Logger.Level.ERROR:System.Logger.Level.DEBUG,format+" "+Arrays.toString(values));
            Throwable loadingError=null;
            try {
                // Same factory sequence as bootstrap: real JDBC, original
                // query whitelist, transaction validation, rollback and close.
                var templates=NativeDeItemTemplatesResources.read(db.getProperty("schema"),sql,log);
                var definitions=NativeDeItemDefinitionsResources.read(db.getProperty("schema"),sql,log);
                require(templates.templates()>0&&templates.counts().rolledBack()==1&&templates.counts().physicalClosed()==1
                    &&templates.counts().lifecycleFailures()==0&&definitions.counts().rolledBack()==1
                    &&definitions.counts().physicalClosed()==1&&definitions.counts().lifecycleFailures()==0,"original resources were not fully published");
                require(opened.size()==2,"unexpected resource SQL connection count");
                for(Connection connection:opened)require(connection.isClosed(),"original reader retained a SQL connection");
                NativeDeItemApplication installed=NativeDeItemApplication.install(hub,templates,definitions);
                require(field(hub,"sourceItemApplication")==installed&&!Boolean.TRUE.equals(field(installed,"closed")),"item application is not live");
                require(field(installed,"life")==field(hub,"sourcePlayerCommitLifetime"),"item application belongs to another hub lifetime");
            } catch(Throwable error) {
                loadingError=error;throw error;
            } finally {
                // Original readers own closure. If a reader fails before its
                // own cleanup barrier, close only connections created here.
                Throwable cleanupError=null;
                for(Connection connection:opened)try{if(!connection.isClosed())connection.close();}
                    catch(Throwable error){if(cleanupError==null)cleanupError=error;else cleanupError.addSuppressed(error);}
                if(cleanupError!=null){if(loadingError!=null)loadingError.addSuppressed(cleanupError);else throw cleanupError;}
            }
            return application;
        } catch(Throwable error) {
            if(application!=null){try{application.close();}catch(Throwable cleanup){error.addSuppressed(cleanup);}}
            else {if(hub!=null)try{hub.close();}catch(Throwable cleanup){error.addSuppressed(cleanup);}clients.shutdownNow();}
            if(error instanceof Exception exception)throw exception;if(error instanceof Error fatal)throw fatal;
            throw new IllegalStateException(error);
        }
    }
    void bindLogin(ServerSocket socket,int port,int backlog)throws Exception {
        require(Thread.currentThread()==installingThread&&!closed,"native application owner changed");
        NativeServerLifetime.login(hub,socket);NativeServerLifetime.bindMain(socket,new InetSocketAddress("127.0.0.1",port),backlog);
    }
    void bindGame(ServerSocket socket,String loopback,int port,int backlog)throws Exception {
        require(!closed&&InetAddress.getByName(loopback).isLoopbackAddress(),"game listener outside owned loopback");
        NativeServerLifetime.bindSource(socket,new InetSocketAddress(loopback,port),backlog,hub);
    }
    Session accept(ServerSocket listener,boolean game)throws Exception {
        require(!closed,"native application closing");Socket socket=NativeServerLifetime.acceptSocket(listener,hub);
        try {
            ClientSession owner=new ClientSession(socket,game?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
            Future<?> future=NativeServerLifetime.submitAccepted(clients,owner);require(future!=null,"native session admission rejected");
            return new Session(socket,owner,future,game);
        } catch(Throwable error) {
            NativeServerLifetime.abortAccept();try{socket.close();}catch(Throwable cleanup){error.addSuppressed(cleanup);}
            if(error instanceof Exception exception)throw exception;if(error instanceof Error fatal)throw fatal;throw new IllegalStateException(error);
        }
    }
    @Override public void close()throws Exception {
        require(Thread.currentThread()==installingThread,"native application must close on its installing thread");
        if(closed)return;closed=true;Throwable first=null;
        try{NativeServerLifetime.closeHub(hub);}catch(Throwable error){first=error;}
        for(AutoCloseable resource:Arrays.asList(capture,maps,accounts))try{NativeServerLifetime.closeResource(resource);}
            catch(Throwable error){if(first==null)first=error;else first.addSuppressed(error);}
        try{NativeServerLifetime.finishClients(clients);}catch(Throwable error){if(first==null)first=error;else first.addSuppressed(error);}
        if(first instanceof Exception exception)throw exception;if(first instanceof Error fatal)throw fatal;
        if(first!=null)throw new IllegalStateException(first);
    }
}
