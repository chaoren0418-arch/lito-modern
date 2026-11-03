// Private x86 floor-dispatch fragment and original raster writers. Not product code.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <stdexcept>
#include <string>
#include <vector>
using U=uint32_t;
static_assert(sizeof(void*)==4,"Requires private x86 runtime");
static void need(bool v,const char* s){if(!v)throw std::runtime_error(s);}
struct Slot {U a=0xaabb1234,value=0,b=0x1234bbaa;void check(){need(a==0xaabb1234&&b==0x1234bbaa,"Global guard");}};
static Slot plane_slot,surface_slot,map_slot;
struct Surface {void** vtable;};static Surface surface;static std::array<void*,33> vtable{};static U unlock_count=0;
static int __stdcall unlock_stub(Surface* p,void* rect){need(p==&surface&&rect==nullptr,"Unlock arguments");++unlock_count;return 0;}
static U addr(const void* p){return reinterpret_cast<U>(p);}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);need(bool(f),"Input absent");return {(std::istreambuf_iterator<char>(f)),{}};}
static void write(const std::filesystem::path& p,const void* b,size_t n){std::ofstream f(p,std::ios::binary);f.write(static_cast<const char*>(b),n);need(bool(f),"Write failed");}
static std::map<U,U> addresses;
static std::map<std::string,unsigned char*> code;
static void load(const std::filesystem::path& out,const std::string& name,U original,bool tail_ret){
 auto b=read(out/(name+"-original.bin"));if(tail_ret)b.push_back(0xc3);auto p=static_cast<unsigned char*>(VirtualAlloc(nullptr,b.size(),MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE));need(p!=nullptr,"Allocation failed");std::memcpy(p,b.data(),b.size());code[name]=p;addresses[original]=addr(p);
}
__declspec(naked) static void invoke(void*,const void*,void*,U,void*,U,void*) {__asm{
 push ebp
 mov ebp,esp
 push ebx
 push esi
 push edi
 mov eax,[ebp+8]
 mov edi,[ebp+12]
 mov esi,[ebp+16]
 mov ebx,[ebp+20]
 mov edx,[ebp+28]
 mov ecx,[ebp+32]
 push ebp
 mov ebp,ecx
 call eax
 pop ebp
 pop edi
 pop esi
 pop ebx
 pop ebp
 ret
}}
static uint16_t color(U i){static const uint16_t keys[]={0,0x18c6,0x7fff,0x739c,0x6318,0x3e0,0x7c00,0x1f};return i<8?keys[i]:static_cast<uint16_t>(i*37+1);}
static std::vector<unsigned char> resource(U type){
 std::vector<unsigned char> b{static_cast<unsigned char>(type)};U index=0;auto pixel=[&](){uint16_t v=color(index++);b.push_back(static_cast<unsigned char>(v));b.push_back(static_cast<unsigned char>(v>>8));};
 if(!(type&2)){for(U i=0;i<288;++i)pixel();return b;}
 b.insert(b.end(),{2,1,20,4});
 for(const auto& spans:std::vector<std::vector<std::pair<U,U>>>{{{2,2},{4,5}},{},{{0,12}},{{8,1},{6,3}}}){b.push_back(static_cast<unsigned char>(spans.size()));for(auto [skip,count]:spans){b.push_back(static_cast<unsigned char>(skip));b.push_back(static_cast<unsigned char>(count));for(U i=0;i<count;++i)pixel();}}
 return b;
}
int wmain(int argc,wchar_t** argv){try{
 need(argc==2,"Expected output folder");std::filesystem::path out(argv[1]);vtable[32]=reinterpret_cast<void*>(&unlock_stub);surface.vtable=vtable.data();surface_slot.value=addr(&surface);addresses[0x6476c4]=addr(&plane_slot.value);addresses[0x6476d0]=addr(&surface_slot.value);addresses[0x62f324]=addr(&map_slot.value);
 load(out,"rle",0x4be000,false);load(out,"right",0x4c14c0,false);load(out,"left",0x4c1670,false);load(out,"floor",0x488c5f,true);
 std::ifstream relocs(out/"relocations.tsv");std::string name,kind;U offset,target,next;
 while(relocs>>name>>kind>>offset>>target>>next){need(code.count(name)&&addresses.count(target),"Unknown relocation");U value=addresses.at(target);if(kind=="relative")value-=addr(code.at(name))+next;else need(kind=="absolute","Unknown relocation kind");std::memcpy(code.at(name)+offset,&value,4);}
 for(const auto& [name,p]:code){size_t n=read(out/(name+"-original.bin")).size()+(name=="floor"?1:0);write(out/(name+"-relocated.bin"),p,n);DWORD old;need(VirtualProtect(p,n,PAGE_EXECUTE_READ,&old)!=0,"Code protect");FlushInstructionCache(GetCurrentProcess(),p,n);}
 std::ofstream address_file(out/"addresses.tsv");for(auto [a,b]:addresses)address_file<<a<<' '<<b<<'\n';address_file.close();
 constexpr U W=744,H=552,G=64;std::ofstream cases(out/"cases.csv");cases<<"id,type,parity,map,pitch,start,unlock_calls\n";U id=0,total_unlocks=0;uint64_t cells=0;
 for(U type:{0,1,2,3,6,7,8,9,16,17,18,19,22,23,34,35}){
  auto data=resource(type),saved=data;write(out/("source-"+std::to_string(type)+".bin"),data.data(),data.size());
  for(U parity:{0,1})for(U map:{302,303}){
   std::vector<uint16_t> frame(G+W*H+G,0x6bad);std::fill(frame.begin()+G,frame.end()-G,0x7777);std::vector<unsigned char> plane(G+W*H+G,0xa6);std::fill(plane.begin()+G,plane.end()-G,0);plane_slot.value=addr(plane.data()+G);map_slot.value=map;unlock_count=0;
   std::array<U,160> locals{};locals.fill(0xa5c3d7e9);U* base=locals.data()+128;base[-0x1a4/4]=addr(frame.data()+G);base[-0x6c/4]=W;auto saved_locals=locals;const U start=250*W+180;
   invoke(code.at("floor"),data.data(),frame.data()+G+start,W*2,frame.data()+G,parity,base);
   need(locals==saved_locals&&data==saved,"Input/frame locals changed");plane_slot.check();surface_slot.check();map_slot.check();need(unlock_count==((type&8)||(type&2)?0u:1u),"Unexpected writer path");
   for(U i=0;i<G;++i){need(frame[i]==0x6bad&&frame[frame.size()-1-i]==0x6bad,"Frame guard");need(plane[i]==0xa6&&plane[plane.size()-1-i]==0xa6,"Plane guard");}
   for(U i=0;i<W*H;++i){const unsigned char expected=map==303&&frame[G+i]!=0x7777?1:0;need(plane[G+i]==expected,"Gate/color-copy coverage disagreement");}
   write(out/(std::to_string(id)+"-frame.bin"),frame.data()+G,W*H*2);write(out/(std::to_string(id)+"-gate.bin"),plane.data()+G,W*H);cases<<id<<','<<type<<','<<parity<<','<<map<<','<<W<<','<<start<<','<<unlock_count<<'\n';++id;total_unlocks+=unlock_count;cells+=2ull*W*H;
  }
 }
 need(id==64&&total_unlocks==16,"Case/path count");std::cout<<"{\"status\":\"original_floor_dispatch_and_gate_executed\",\"cases\":64,\"color_and_gate_cells\":"<<cells<<",\"surface_Unlock_stub_calls\":"<<total_unlocks<<",\"guard_failures\":0,\"whole_client_executed\":false}"<<std::endl;return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}}
