package local.lito381.server;
import java.util.*;
public final class NativeQuestRouteOracle {
 static int route(int sx,int sy,int tx,int ty,int range){var arena=ArenaMapCatalog.forMap(554);var seen=new HashSet<Long>();var q=new ArrayDeque<int[]>();q.add(new int[]{sx,sy,0});seen.add(((long)sx<<32)|sy);int[] dx={0,1,1,1,0,-1,-1,-1},dy={-1,-1,0,1,1,1,0,-1};
  while(!q.isEmpty()){var p=q.remove();if(Math.max(Math.abs(p[0]-tx),Math.abs(p[1]-ty))<=range)return p[2];if(p[2]>=128)continue;for(int h=0;h<8;h++){int x=p[0]+dx[h],y=p[1]+dy[h];long key=((long)x<<32)|(y&0xffffffffL);if(arena.nativeCanStep(p[0],p[1],x,y)&&seen.add(key))q.add(new int[]{x,y,p[2]+1});}if(seen.size()>65536)throw new IllegalStateException("source graph budget");}return -1;
 }
 public static void main(String[] args){System.out.println("CURRENT_FORMAL_WALL_LOS="+BattleSpellCatalog.hasNativeLineOfSight(ArenaMapCatalog.forMap(554),32746,32819,32752,32825));for(int[] s:new int[][]{{32801,32885},{32755,32785}})for(int[] g:new int[][]{{32745,32830},{32774,32865},{32780,32802},{32800,32856}})System.out.println("CURRENT_FORMAL_554 "+Arrays.toString(s)+" -> "+Arrays.toString(g)+" range6 steps="+route(s[0],s[1],g[0],g[1],6));}
}