package local.lito381.server;
import java.util.*;
import java.nio.file.*;
import java.security.MessageDigest;
public final class NativeMerchantOracle {
    static String hex(byte[] b){return HexFormat.of().formatHex(b);}
    public static void main(String[] args)throws Exception{
        var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=hex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        long npc=1812987916L;var rows=new ArrayList<Protocol.ClassicBuyListRow>();
        var offered=NativeLitoRentalMerchantCatalog.offeredBy(12);
        for(int i=0;i<offered.size();i++){var item=offered.get(i);rows.add(new Protocol.ClassicBuyListRow(i,item.definition().icon(),item.lightGemPrice(),item.definition().name(),item.buyListDetail()));}
        if(rows.isEmpty())throw new IllegalStateException("empty Love merchant");
        byte[] talk=new Protocol.Writer().c(0x29).d(npc).bytes();if(Protocol.dialogObjectId(talk)!=npc)throw new IllegalStateException();
        byte[] action=new Protocol.Writer().c(0x27).d(npc).s("buy").s("").bytes();Protocol.hypertextActionRequest(action);
        byte[] buy=new Protocol.Writer().c(0x28).d(npc).c(0).c(1).c(0).d(0).d(1).bytes();Protocol.classicShopTransaction(buy);
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"npc\":"+npc+",\"rows\":"+rows.size()+",\"talk\":\""+hex(talk)+"\",\"action\":\""+hex(action)+"\",\"buy\":\""+hex(buy)+"\",\"dialog\":\""+hex(Protocol.hypertext(npc,"Love_Start","",List.of("中文", "")))+"\",\"list\":\""+hex(Protocol.litoRentalBuyList(npc,rows))+"\",\"sell\":\""+hex(Protocol.classicSellList(npc,List.of(new Protocol.ClassicSellListRow(0xffffffffL,0xffffffffL))))+"\",\"boundary_buy\":\""+hex(Protocol.litoRentalBuyList(npc,List.of(new Protocol.ClassicBuyListRow(0xffffffffL,65535,0xffffffffL,"中文",new byte[255]))))+"\"}");
    }
}
