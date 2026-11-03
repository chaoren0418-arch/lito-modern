// Staged x86 oracle. Original row copier/raster code is never a game client.
// Execute only after owned GUI work has fully quiesced.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>
static_assert(sizeof(void*)==4,"Original water fragments are x86");
static std::array<uint16_t,32> input_b{},input_g{},input_r{};
static std::array<uint16_t,320> response_br{},response_g{};
static std::array<uint16_t,1024> normal_b{},normal_g{},normal_r{};
static std::array<uint16_t,10240> water_b{},water_g{},water_r{};
static std::array<unsigned char,32> remap{};
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);if(!f)throw std::runtime_error("Input missing");return {(std::istreambuf_iterator<char>(f)),{}};}
static void replace(unsigned char* code,size_t size,uint32_t old,uintptr_t pointer,int expected){
 int count=0;for(size_t i=0;i+4<=size;++i){uint32_t value;memcpy(&value,code+i,4);if(value==old){memcpy(code+i,&pointer,4);++count;i+=3;}}
 if(count!=expected)throw std::runtime_error("Data relocation count differs");
}
static void* load(const std::filesystem::path& dir,const wchar_t* name,size_t size,bool row){
 auto bytes=read(dir/(std::wstring(name)+L"-original.bin"));if(bytes.size()!=size)throw std::runtime_error("Original extent differs");
 auto p=static_cast<unsigned char*>(VirtualAlloc(nullptr,65536,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE));if(!p)throw std::runtime_error("Allocation failed");memcpy(p,bytes.data(),size);
 replace(p,size,0x76889c,reinterpret_cast<uintptr_t>(water_b.data()),row?1:2);
 replace(p,size,0x76d89c,reinterpret_cast<uintptr_t>(water_g.data()),row?1:2);
 replace(p,size,0x77289c,reinterpret_cast<uintptr_t>(water_r.data()),row?1:2);
 if(row){
  replace(p,size,0x6351a8,reinterpret_cast<uintptr_t>(input_b.data()),1);replace(p,size,0x6351e8,reinterpret_cast<uintptr_t>(input_g.data()),1);replace(p,size,0x635228,reinterpret_cast<uintptr_t>(input_r.data()),1);
  replace(p,size,0x634f28,reinterpret_cast<uintptr_t>(response_br.data()),2);replace(p,size,0x634ca8,reinterpret_cast<uintptr_t>(response_g.data()),1);
  replace(p,size,0x7a909c,reinterpret_cast<uintptr_t>(normal_b.data()),1);replace(p,size,0x7a989c,reinterpret_cast<uintptr_t>(normal_g.data()),1);replace(p,size,0x7aa09c,reinterpret_cast<uintptr_t>(normal_r.data()),1);p[size]=0xc3;
 }else replace(p,size,0x83009c,reinterpret_cast<uintptr_t>(remap.data()),1);
 std::ofstream f(dir/(std::wstring(name)+L"-relocated.bin"),std::ios::binary);f.write(reinterpret_cast<char*>(p),size);f.close();if(!f)throw std::runtime_error("Relocated retention failed");
 DWORD old;if(!VirtualProtect(p,65536,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),p,65536))throw std::runtime_error("Code protection failed");return p;
}
__declspec(naked) static void invoke_row(void*,void*,int,int,int){
 __asm {
  push ebp
  push ebx
  push esi
  push edi
  mov eax,[esp+20]
  mov ebp,[esp+24]
  mov ecx,[esp+28]
  mov edi,[esp+32]
  mov edx,[esp+36]
  call eax
  pop edi
  pop esi
  pop ebx
  pop ebp
  ret
 }
}
using Kernel=void(__fastcall*)(uint16_t*,const unsigned char*,int,int,int,int,int,int,const int32_t*);
int wmain(int argc,wchar_t** argv){try{
 if(argc!=3)throw std::runtime_error("Expected owned fragment directory and readonly shade.dat");const std::filesystem::path dir(argv[1]);
 auto tables=read(dir/L"water-original-static-tables.bin");if(tables.size()!=1472)throw std::runtime_error("Static table extent");
 size_t offset=0;auto take=[&](auto& table){memcpy(table.data(),tables.data()+offset,sizeof(table));offset+=sizeof(table);};take(input_b);take(input_g);take(input_r);take(response_br);take(response_g);
 auto row=load(dir,L"copy_water_rows",144,true);auto k555=reinterpret_cast<Kernel>(load(dir,L"water555",436,false));auto k565=reinterpret_cast<Kernel>(load(dir,L"water565",430,false));
 // CRT text mode, matching original fopen("r") and fscanf("%c").
 std::ifstream shade_file{std::filesystem::path(argv[2])};if(!shade_file)throw std::runtime_error("Shade resource absent");
 std::vector<int32_t> shade;char byte;while(shade_file.get(byte)){int value=static_cast<signed char>(byte);shade.push_back(value>=10?9:value);}
 if(shade.size()!=640u*420u)throw std::runtime_error("CRT shade text-mode extent");for(int value:shade)if(value<0||value>9)throw std::runtime_error("Unqualified shade value");
 std::ofstream shade_out(dir/L"shade-private-CRT-clamped.bin",std::ios::binary);for(int value:shade)shade_out.put(static_cast<char>(value));shade_out.close();if(!shade_out)throw std::runtime_error("Shade retention failed");
 std::ofstream table_out(dir/L"water-native-channel-tables.bin",std::ios::binary),vectors(dir/L"water-vectors.csv",std::ios::binary);vectors<<"mode,phase,water_x,water_y,x,y,raw,color,shade,mapped,actual\n";
 constexpr int W=640,H=408,MW=320,MH=204,P=800,FH=414,X=33,Y=3;
 std::vector<uint16_t> pixels(P*FH),expected;std::vector<unsigned char> mask(MW*MH),saved_mask;uint64_t calls=0,pixel_checks=0,guards=0,rows=0,retained=0;
 for(int mode=0;mode<2;++mode){
  for(int c=0;c<32;++c)for(int l=0;l<32;++l){const int value=c*l/31;normal_b[c*32+l]=static_cast<uint16_t>(value);normal_g[c*32+l]=static_cast<uint16_t>(value<<(mode?6:5));normal_r[c*32+l]=static_cast<uint16_t>(value<<(mode?11:10));}
  water_b.fill(0xeeee);water_g.fill(0xeeee);water_r.fill(0xeeee);
  for(int s=0;s<10;++s)for(int c=0;c<32;++c){alignas(16) int frame[32]{};int* ebp=frame+16;ebp[-3]=s*32;invoke_row(row,ebp,c*2,s*32,(s*1024+c*32)*2);++rows;}
  for(int s=0;s<10;++s)for(int c=0;c<32;++c)for(int l=0;l<32;++l){int i=s*1024+c*32+l;if(water_b[i]!=normal_b[response_br[s*32+input_b[c]]*32+l]||water_g[i]!=normal_g[response_g[s*32+input_g[c]]*32+l]||water_r[i]!=normal_r[response_br[s*32+input_r[c]]*32+l])throw std::runtime_error("Native row-copy mismatch");}
  for(const auto* table:{&water_b,&water_g,&water_r})table_out.write(reinterpret_cast<const char*>(table->data()),sizeof(water_b));
  for(int phase:{-21,-20,-1,0,1,20,21})for(int sx:{0,639})for(int sy:{0,419}){
   for(size_t i=0;i<pixels.size();++i)pixels[i]=static_cast<uint16_t>(i*7919+phase*31+sx);expected=pixels;
   for(size_t i=0;i<mask.size();++i)mask[i]=static_cast<unsigned char>(i*37+sy);
   for(int l=0;l<32;++l)remap[l]=static_cast<unsigned char>((l*13+7)&31);saved_mask=mask;
   for(int y=0;y<H;++y)for(int x=0;x<W;++x){int wx=(1280-sx-abs(phase)+2*(x/2))%640,wy=y==0?(816-sy)%408:(((816-sy)%408+y-1)%408+1);int s=shade[(wx/2)*420+wy],l=remap[mask[(y/2)*MW+x/2]/8];size_t at=(Y+y)*P+X+x;uint16_t c=pixels[at];expected[at]=water_b[s*1024+(c&31)*32+l]|water_g[s*1024+((c>>(mode?6:5))&31)*32+l]|water_r[s*1024+((c>>(mode?11:10))&31)*32+l];}
   (mode?k565:k555)(pixels.data(),mask.data(),P*2,X,Y,phase,sx,sy,shade.data());++calls;if(mask!=saved_mask)throw std::runtime_error("Mask modified");
   for(int y=0;y<FH;++y)for(int x=0;x<P;++x){size_t at=y*P+x;if(pixels[at]!=expected[at]){
    std::cerr<<"Water mismatch mode="<<mode<<" phase="<<phase<<" water_x="<<sx<<" water_y="<<sy<<" buffer_x="<<x<<" buffer_y="<<y<<" actual="<<pixels[at]<<" expected="<<expected[at]<<'\n';
    for(auto pair:{std::pair<const wchar_t*,const std::vector<uint16_t>*>{L"water-failure-pixels.bin",&pixels},{L"water-failure-expected.bin",&expected}}){std::ofstream f(dir/pair.first,std::ios::binary);f.write(reinterpret_cast<const char*>(pair.second->data()),pair.second->size()*2);}
    throw std::runtime_error("Water raster/guard mismatch");
   }if(y>=Y&&y<Y+H&&x>=X&&x<X+W)++pixel_checks;else ++guards;}
   for(int y=0;y<H;y+=17)for(int x=0;x<W;x+=37){int wx=(1280-sx-abs(phase)+2*(x/2))%640,wy=y==0?(816-sy)%408:(((816-sy)%408+y-1)%408+1);size_t at=(Y+y)*P+X+x;unsigned raw=mask[(y/2)*MW+x/2];uint16_t original=static_cast<uint16_t>(at*7919+phase*31+sx);vectors<<mode<<','<<phase<<','<<sx<<','<<sy<<','<<x<<','<<y<<','<<raw<<','<<original<<','<<shade[(wx/2)*420+wy]<<','<<unsigned(remap[raw/8])<<','<<pixels[at]<<'\n';++retained;}
  }
 }
 table_out.close();vectors.close();if(!table_out||!vectors)throw std::runtime_error("Output failed");
 VirtualFree(row,0,MEM_RELEASE);VirtualFree(reinterpret_cast<void*>(k555),0,MEM_RELEASE);VirtualFree(reinterpret_cast<void*>(k565),0,MEM_RELEASE);
 std::cout<<"{\"status\":\"original_water_row_copy_and_kernels_match_scalar\",\"row_calls\":"<<rows<<",\"native_raster_calls\":"<<calls<<",\"viewport_pixels_checked\":"<<pixel_checks<<",\"guard_pixels_checked\":"<<guards<<",\"retained_vectors\":"<<retained<<",\"full_original_initializer_executed\":false,\"game_sessions\":0}\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
