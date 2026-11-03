package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Direct current-JAR inventory wire vectors. Does not emulate ClientSession. */
public final class NativeInventoryOracle {
 public static void main(String[] args)throws Exception {
  long id=0xfedcba98L;
  var first=new Protocol.InventoryWireItem(id,255,254,65535,3,0xffffffffL,255,"魔法寶石",new byte[]{0,1,(byte)255});
  var second=new Protocol.InventoryWireItem(7,0,0,12,0,2,1,"治癒藥水",new byte[0]);
  var replaced=new Protocol.InventoryWireItem(id,1,2,3,1,9,0,"替換",new byte[]{4});
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  p.put("add",Protocol.addInventory(first));p.put("batch",Protocol.inventoryBatch(List.of(first,second)));
  p.put("empty",Protocol.inventoryBatch(List.of()));p.put("replace",Protocol.addInventory(replaced));
  p.put("duplicate",Protocol.inventoryBatch(List.of(first,replaced)));
  p.put("remove",Protocol.removeInventory(id));p.put("count",Protocol.changeInventoryCount(id,123));
  p.put("zero_count",Protocol.changeInventoryCount(id,0));p.put("name",Protocol.changeInventoryDescription(id,"已裝備"));
  p.put("charge",Protocol.changeInventoryCharge(id,255));p.put("blessing",Protocol.changeInventoryBlessing(id,131));
  p.put("extended",Protocol.changeInventoryDescriptionExtended(id,"更新名稱",0xffffffffL,new byte[]{7,0,9}));
  byte[] details=new byte[255];Arrays.fill(details,(byte)255);
  p.put("limits",Protocol.addInventory(new Protocol.InventoryWireItem(0xffffffffL,255,255,65535,3,0xffffffffL,255,"a".repeat(255),details)));
  p.put("formatting",Protocol.addInventory(new Protocol.InventoryWireItem(8,0,0,0,0,1,0,"\fY物品\t",new byte[0])));
  p.put("blank_name",Protocol.addInventory(new Protocol.InventoryWireItem(9,0,0,0,0,1,0,"",new byte[0])));
  var many=new ArrayList<Protocol.InventoryWireItem>();
  for(int i=1;i<=255;i++)many.add(new Protocol.InventoryWireItem(i,0,0,0,0,1,0,"批次",new byte[0]));
  p.put("max_batch",Protocol.inventoryBatch(many));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
