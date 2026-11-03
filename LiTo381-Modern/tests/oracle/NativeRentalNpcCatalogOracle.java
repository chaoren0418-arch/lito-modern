package local.lito381.server;
import java.util.StringJoiner;
/** Read-only catalogue inspection; never creates sessions, configuration, or a store. */
public final class NativeRentalNpcCatalogOracle {
    static String quote(String s){return s==null?"null":"\""+s.replace("\\","\\\\").replace("\"","\\\"")+"\"";}
    public static void main(String[] args){
        var rows=new StringJoiner(",","[","]");
        for(var n:NativeLitoRentalTownNpcCatalog.forMap(500)){
            rows.add("{\"id\":"+n.objectId()+",\"name\":"+quote(n.name())+",\"resource\":"+n.resourceIndex()+",\"x\":"+n.x()+",\"y\":"+n.y()+",\"role\":"+quote(n.role().name())+",\"start_html\":"+quote(n.startHtml())+",\"end_html\":"+quote(n.endHtml())+",\"action\":"+quote(n.action())+"}");
        }
        System.out.println(rows);
    }
}
