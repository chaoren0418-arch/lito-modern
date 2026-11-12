package local.lito381.server;

import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.function.Consumer;

/** Offline formal E149 parsers/writers only. No ClientSession, socket, DB or Main. */
public final class NativeItemOptionsOracle {
    static final long ITEM = 0xfedcba98L, TARGET = 0xffffffffL;
    static final Map<String,byte[]> vectors = new LinkedHashMap<>();
    static void vector(String name, byte[] bytes, Consumer<byte[]> parser) {
        parser.accept(bytes);
        for (int i=0;i<bytes.length;i++) reject(Arrays.copyOf(bytes,i),parser);
        reject(Arrays.copyOf(bytes,bytes.length+1),parser);
        vectors.put(name,bytes);
    }
    static void reject(byte[] bytes, Consumer<byte[]> parser) {
        try { parser.accept(bytes); } catch (IllegalArgumentException expected) { return; }
        throw new AssertionError("malformed request accepted");
    }
    static void require(boolean condition) { if (!condition) throw new AssertionError(); }
    public static void main(String[] args) throws Exception {
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        require(sha.equals("e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498"));
        vector("destroy", new Protocol.Writer().c(0x76).d(ITEM).bytes(),
            b -> require(Protocol.inventoryObjectRequest(b,0x76)==ITEM));
        vector("appraisal", new Protocol.Writer().c(0x1c).d(ITEM).d(TARGET).bytes(),
            b -> { var r=Protocol.appraisalInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.targetObjectId()==TARGET); });
        vector("choice", new Protocol.Writer().c(0x1c).d(ITEM).d(TARGET).bytes(),
            b -> { var r=Protocol.choiceInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.targetObjectId()==TARGET); });
        vector("light", new Protocol.Writer().c(0x1c).d(ITEM).d(TARGET).bytes(),
            b -> { var r=Protocol.lightInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.clientTick()==TARGET); });
        vector("home", new Protocol.Writer().c(0x1c).d(ITEM).h(65535).h(0).h(0).bytes(),
            b -> { var r=Protocol.homeTeleportInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.currentMapId()==65535); });
        vector("polymorph", new Protocol.Writer().c(0x1c).d(ITEM).s("orc").bytes(),
            b -> { var r=Protocol.defaultInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.argument().equals("orc")); });
        vector("whistle",new Protocol.Writer().c(0x1c).d(ITEM).s("").bytes(),
            b -> require(Protocol.defaultInventoryUse(b).inventoryObjectId()==ITEM));
        vector("read_letter",new Protocol.Writer().c(0x1c).d(ITEM).s("").bytes(),
            b -> require(Protocol.defaultInventoryUse(b).argument().isEmpty()));
        vector("compose_letter",new Protocol.Writer().c(0x1c).d(ITEM).h(0).s("Receiver").S("測試😀").S("第一行\r\n第二行").bytes(),
            b -> {var r=Protocol.letterComposeInventoryUse(b); require(r.inventoryObjectId()==ITEM
                && r.letterCode()==0 && r.receiver().equals("Receiver") && r.subject().equals("測試😀") && r.content().equals("第一行\r\n第二行"));});
        vectors.put("read_mail",Protocol.readMail(ITEM,466,0,"Sender","Receiver","測試😀","第一行\r\n第二行",0,"26/11/06"));
        require(InventoryCatalog.require(40310).useType()==12 && InventoryCatalog.require(49016).useType()==13
            && InventoryCatalog.require(49017).useType()==13 && InventoryCatalog.require(40315).useType()==15);
        require(InventoryCatalog.require(40315).effect()==InventoryCatalog.Effect.WHISTLE);
        require(InventoryCatalog.require(40100).useType()==6 && InventoryCatalog.require(40100).effect()==InventoryCatalog.Effect.NONE);
        for(int id=40090;id<=40094;id++) require(InventoryCatalog.require(id).useType()==28
            && InventoryCatalog.require(id).effect()==InventoryCatalog.Effect.NONE);
        vector("target_location",new Protocol.Writer().c(0x1c).d(ITEM).d(TARGET).h(32768).h(65535).bytes(),
            b -> { var r=Protocol.targetLocationInventoryUse(b); require(r.inventoryObjectId()==ITEM && r.targetObjectId()==TARGET && r.x()==32768 && r.y()==65535); });
        for(int id:new int[]{12,21,73,100,107}) {
            vector("skill_"+id,new Protocol.Writer().c(0x14).c((id-1)/8).c((id-1)%8).d(TARGET).bytes(),
                b -> {var r=Protocol.objectSpellRequest(b); require(r.skillId()==id && r.targetObjectId()==TARGET);});
        }
        for(int gfx:new int[]{194,197}) vectors.put("effect_"+gfx,Protocol.effectObject(TARGET,gfx));
        vectors.put("charge_zero",Protocol.changeInventoryCharge(ITEM,0));
        vectors.put("count_one",Protocol.changeInventoryCount(ITEM,1));
        vectors.put("remove",Protocol.removeInventory(ITEM));
        var spring=BattleSpellCatalog.groundSpell(63);
        require(spring.itemConsumeId()==40318 && spring.itemConsumeCount()==1 && spring.mpCost()==20);
        StringJoiner rows=new StringJoiner(",");
        for(var e:vectors.entrySet()) rows.add("\""+e.getKey()+"\":\""+HexFormat.of().formatHex(e.getValue())+"\"");
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"scope\":\"formal parsers and writers; no gameplay execution\",\"packets\":{"+rows+"}}");
    }
}
