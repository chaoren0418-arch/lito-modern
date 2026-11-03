package local.lito381.server;
import java.nio.file.*;import java.security.MessageDigest;import java.util.*;
/** Read-only formal product catalogue; no sessions, inventory or clock changes. */
public final class NativeRentalMerchantOracle{
 static String q(String s){return "\""+s.replace("\\","\\\\").replace("\"","\\\"")+"\"";}
 public static void main(String[] args)throws Exception{
  var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());var shops=new StringJoiner(",");
  for(var npc:NativeLitoRentalTownNpcCatalog.forMap(500)){
   var items=NativeLitoRentalMerchantCatalog.offeredBy(npc.resourceIndex());if(items.isEmpty())continue;
   var rows=new StringJoiner(",");for(var item:items){var d=item.definition();rows.add("{\"resource\":"+item.resourceIndex()+",\"source_id\":"+item.sourceItemId()+",\"catalog_id\":"+d.catalogId()+",\"name\":"+q(d.name())+",\"seconds\":"+item.maximumSeconds()+",\"gems\":"+item.lightGemPrice()+",\"weight\":"+d.unitWeight()+",\"stackable\":"+d.stackable()+",\"use_type\":"+d.useType()+",\"icon\":"+d.icon()+",\"ground_gfx\":"+d.groundGfx()+"}");}
   shops.add("{\"npc\":"+npc.objectId()+",\"resource\":"+npc.resourceIndex()+",\"name\":"+q(npc.name())+",\"html\":"+q(npc.startHtml())+",\"items\":["+rows+"]}");
  }
  System.out.println("{\"jar_sha256\":"+q(HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar))))+",\"shops\":["+shops+"]}");
 }
}
