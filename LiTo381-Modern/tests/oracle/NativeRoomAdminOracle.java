package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Codec domains only. Passwords are synthetic fixture strings, no live users. */
public final class NativeRoomAdminOracle {
    static byte[] body(int op,String name,int attr,String password,int capacity,int[] options,int[] selections){
        var w=new Protocol.Writer().c(op).s(name).c(attr).s(password).h(capacity);
        if(options!=null){for(int x:options)w.c(x);w.c(selections.length);for(int x:selections)w.c(x);}
        return w.bytes();
    }
    public static void main(String[] args)throws Exception {
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringJoiner rows=new StringJoiner(","),capacities=new StringJoiner(","),create=new StringJoiner(","),observer=new StringJoiner(","),kick=new StringJoiner(","),kickStatus=new StringJoiner(",");
        for(int op:new int[]{0xcc,0xdd})for(int attr:new int[]{0,4,8,16,32,60})for(int capacity:new int[]{2,8,32,200})for(int layout=0;layout<3;layout++){
            String name=layout==2?"N".repeat(89):"建房測試";String password=(attr&4)!=0?"密碼":"";
            int[] options=layout==0?null:layout==1?new int[]{0,0,0,0}:new int[]{7,2,127,31};
            int[] selections=layout==2?new int[]{1,2,3,4,5,6,7,255}:new int[]{};
            byte[] b=body(op,name,attr,password,capacity,options,selections);
            var r=Protocol.litoRoomSettingsRequest(b,op);
            if(!r.name().equals(name)||!r.password().equals(password)||r.capacity()!=capacity||r.attributes()!=attr||r.hasOptions()!=(options!=null))throw new AssertionError("settings roundtrip");
            rows.add("{\"opcode\":"+op+",\"name\":\""+name+"\",\"attributes\":"+attr+",\"password\":\""+password+"\",\"capacity\":"+capacity+
                ",\"options\":"+(options==null?"null":Arrays.toString(options))+",\"selections\":"+Arrays.toString(selections)+",\"body\":\""+HexFormat.of().formatHex(b)+"\"}");
        }
        int attrs=0,wireCaps=0;
        for(int attr=0;attr<256;attr++){
            try{Protocol.litoRoomSettingsRequest(body(0xcc,"Room",attr,(attr&4)!=0?"1234":"",8,null,new int[]{}),0xcc);attrs++;}
            catch(IllegalArgumentException expected){}
        }
        for(int cap=0;cap<=65535;cap++){
            try{Protocol.litoRoomSettingsRequest(body(0xcc,"Room",0,"",cap,null,new int[]{}),0xcc);wireCaps++;}
            catch(IllegalArgumentException expected){}
            if(GameHub.nativeRoomCapacity(cap))capacities.add(Integer.toString(cap));
        }
        if(attrs!=16||wireCaps!=199)throw new AssertionError("settings domain");
        for(int channel=0;channel<4;channel++)for(int level:new int[]{1,20,30,47,52,99})
            create.add("{\"channel\":"+channel+",\"level\":"+level+",\"allowed\":"+GameHub.canCreateRoom(channel,level)+"}");
        int observerRejected=0;
        for(int value=0;value<256;value++){
            byte[] b=new byte[]{(byte)0xe1,(byte)value};
            try {var state=Protocol.litoObserverRequest(b);
                observer.add("{\"value\":"+value+",\"enabled\":"+state.enabled()+",\"request\":\""+HexFormat.of().formatHex(b)+"\",\"reply\":\""+HexFormat.of().formatHex(Protocol.litoObserverResult(state))+"\"}");
            }catch(IllegalArgumentException expected){observerRejected++;}
        }
        for(String name:new String[]{"測試角色","PeerFixture2(測試乙)","N".repeat(63)}){
            byte[] b=new Protocol.Writer().c(0xd4).s(name).bytes();
            if(!Protocol.litoKickRequest(b).equals(name))throw new AssertionError("kick name");
            kick.add("{\"name\":\""+name+"\",\"request\":\""+HexFormat.of().formatHex(b)+"\",\"notice\":\""+HexFormat.of().formatHex(new Protocol.Writer().c(0xd6).s(name).bytes())+"\"}");
        }
        for(var state:Protocol.LitoKickStatusResult.values())kickStatus.add("{\"state\":\""+state+"\",\"body\":\""+HexFormat.of().formatHex(Protocol.litoKickStatus(state))+"\"}");
        System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"settings\":["+rows+"],\"accepted_attributes\":"+attrs+
            ",\"accepted_wire_capacities\":"+wireCaps+",\"native_capacities\":["+capacities+"],\"create_gates\":["+create+"],\"observer\":["+observer+"],\"observer_rejected\":"+observerRejected+
            ",\"kick\":["+kick+"],\"kick_status\":["+kickStatus+"],\"live_verified\":false}");
    }
}
