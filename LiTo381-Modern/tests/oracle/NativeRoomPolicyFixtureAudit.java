package local.lito381.server;
import java.nio.file.*;
import java.util.*;
import java.security.*;

/** Constructor audit only: explicit initial memory fixture, no client or SQL. */
public final class NativeRoomPolicyFixtureAudit {
 public static void main(String[] args)throws Exception {
  Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
  Path configPath=root.resolve("server.properties");
  Files.writeString(configPath,"bind=127.0.0.1\nport=1\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\nstate.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused.properties\nsource.servers.file=\n");
  var config=ServerConfig.load(configPath);
  var child=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
  var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試甲",child);routes.put("測試乙",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
  try(var routed=new SourceRoutedAccountRepository(routes)) {
   if(routed.repositoryForClassicSource("測試甲")!=child)throw new IllegalStateException("fixture identity");
   try(var hub=new GameHub(config,new GameStateStore(config.stateFile(),false),child)) {
    for(int c=0;c<4;c++)if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("membership");
    Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
    String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
    Files.writeString(root.resolve("result.json"),"{\"status\":\"explicit_memory_child_constructor_passed\",\"jar_sha256\":\""+sha+"\",\"same_fixture_repository_identity\":true,\"client_sessions\":0,\"sql_access\":false,\"protocol_verified\":false,\"production_routing_verified\":false}");
   }
  }
 }
}
