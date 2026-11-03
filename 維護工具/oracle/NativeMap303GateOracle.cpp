// Private x86 terrain/gate fragments only; never linked into the product.
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
#include <stdexcept>
#include <string>
#include <vector>
static_assert(sizeof(void*)==4,"Original fragments require x86");
using U=uint32_t;
static void need(bool v,const char* m){if(!v)throw std::runtime_error(m);}
struct Slot{U before=0x1a2b3c4d,value=0,after=0xa1b2c3d4;void check(){need(before==0x1a2b3c4d&&after==0xa1b2c3d4,"Global guard");}};
static Slot plane_slot,surface_slot;
struct Surface {void** vtable;};
static Surface surface;
static std::array<void*,33> table{};
static U unlock_calls=0;
static int __stdcall unlock_stub(Surface* object,void* rect){need(object==&surface&&rect==nullptr,"Unlock arguments");++unlock_calls;return 0;}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);need(bool(f),"Missing input");return {(std::istreambuf_iterator<char>(f)),{}};}
static void write(const std::filesystem::path& p,const void* data,size_t size){std::ofstream f(p,std::ios::binary);f.write(static_cast<const char*>(data),size);need(bool(f),"Output failed");}
static U addr(const void* p){return reinterpret_cast<U>(p);}
static void* code(const std::filesystem::path& folder,const char* name,std::initializer_list<std::pair<U,U>> reloc){
    auto original=read(folder/(std::string(name)+"-original.bin"));auto bytes=original;
    for(auto [offset,value]:reloc){need(offset+4<=bytes.size(),"Relocation bound");std::memcpy(bytes.data()+offset,&value,4);}
    auto p=VirtualAlloc(nullptr,bytes.size(),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);need(p!=nullptr,"Code allocation");std::memcpy(p,bytes.data(),bytes.size());DWORD prior=0;need(VirtualProtect(p,bytes.size(),PAGE_EXECUTE_READ,&prior)!=0,"Code protection");FlushInstructionCache(GetCurrentProcess(),p,bytes.size());
    write(folder/(std::string(name)+"-relocated.bin"),bytes.data(),bytes.size());return p;
}
using Rle=void(__fastcall*)(const void*,void*,int,unsigned char,void*);
using Raw=void(__fastcall*)(const void*,void*,int,void*,int,unsigned char);
static constexpr U GW=744,GH=552,G=64;
static std::vector<unsigned char> rle(int kind){
    std::vector<unsigned char> b;int x=kind==0?0:kind==1?-7:5,y=kind==0?0:kind==1?3:-4;
    b={static_cast<unsigned char>(x),static_cast<unsigned char>(y),0,4};
    auto row=[&](std::initializer_list<std::pair<U,U>> spans){b.push_back(static_cast<unsigned char>(spans.size()));for(auto [skip,count]:spans){b.push_back(static_cast<unsigned char>(skip));b.push_back(static_cast<unsigned char>(count));for(U i=0;i<count;++i){uint16_t v=static_cast<uint16_t>(0x8001+b.size()%0x7000);b.push_back(static_cast<unsigned char>(v));b.push_back(static_cast<unsigned char>(v>>8));}}};
    row({{3,2},{4,5},{0,0}});row({});row({{0,12}});row({{8,1},{7,3}});return b;
}
int wmain(int argc,wchar_t** argv){
 try{
    need(argc==2,"Expected folder");std::filesystem::path folder(argv[1]);
    table[32]=reinterpret_cast<void*>(&unlock_stub);surface.vtable=table.data();surface_slot.value=addr(&surface);
    auto a=code(folder,"rle",{{174,addr(&plane_slot.value)}});
    auto b=code(folder,"right",{{126,addr(&plane_slot.value)},{314,addr(&plane_slot.value)},{400,addr(&surface_slot.value)}});
    auto c=code(folder,"left",{{123,addr(&plane_slot.value)},{308,addr(&plane_slot.value)},{394,addr(&surface_slot.value)}});
    std::ofstream addresses(folder/"addresses.tsv");addresses<<std::hex<<0x6476c4<<' '<<addr(&plane_slot.value)<<'\n'<<0x6476d0<<' '<<addr(&surface_slot.value)<<'\n';addresses.close();
    std::ofstream meta(folder/"cases.csv");meta<<"id,function,pitch,reverse,enabled,seed,kind,start_pixel,unlock_calls\n";
    U cases=0;uint64_t compared=0;
    for(U fn=0;fn<3;++fn)for(int pitch:{640,744,800})for(int reverse:{0})for(int enabled:{0,1})for(int seed:{0,7}){
        const U start=250*static_cast<U>(pitch)+180;const U count=static_cast<U>(pitch)*GH;
        std::vector<uint16_t> frame(G+count+G,0x6bad);std::fill(frame.begin()+G,frame.end()-G,0x7777);
        std::vector<unsigned char> gate(G+GW*GH+G,0xa6);std::fill(gate.begin()+G,gate.end()-G,static_cast<unsigned char>(seed));
        plane_slot.value=addr(gate.data()+G);unlock_calls=0;
        std::vector<uint16_t> raw(4096);for(U i=0;i<raw.size();++i)raw[i]=static_cast<uint16_t>(0x8001+i%0x7000);const auto raw_saved=raw;
        int kind=(pitch==640?0:pitch==744?1:2);auto packed=rle(kind),packed_saved=packed;
        void* target=frame.data()+G+start;void* origin=frame.data()+G;
        if(fn==0)reinterpret_cast<Rle>(a)(packed.data(),target,(reverse?-1:1)*2*pitch,static_cast<unsigned char>(enabled),origin);
        else reinterpret_cast<Raw>(fn==1?b:c)(raw.data(),target,2*pitch-(fn==1?8:4),origin,pitch,static_cast<unsigned char>(enabled));
        plane_slot.check();surface_slot.check();need(raw==raw_saved&&packed==packed_saved,"Source changed");
        need(unlock_calls==(fn==0?0:1),"Unlock count");
        for(U i=0;i<G;++i){need(frame[i]==0x6bad&&frame[frame.size()-1-i]==0x6bad,"Frame guard");need(gate[i]==0xa6&&gate[gate.size()-1-i]==0xa6,"Gate guard");}
        std::vector<unsigned char> expected(GW*GH,static_cast<unsigned char>(seed));
        for(U i=0;i<count;++i)if(frame[G+i]!=0x7777){U y=i/static_cast<U>(pitch),x=i%static_cast<U>(pitch);need(x<GW,"Synthetic run exceeds gate row");if(enabled)expected[y*GW+x]=1;}
        need(std::equal(expected.begin(),expected.end(),gate.begin()+G),"Gate differs from color-copy coverage");
        std::string id=std::to_string(cases);write(folder/(id+"-frame.bin"),frame.data()+G,count*2);write(folder/(id+"-gate.bin"),gate.data()+G,GW*GH);
        meta<<cases<<','<<(fn==0?"rle":fn==1?"right":"left")<<','<<pitch<<','<<reverse<<','<<enabled<<','<<seed<<','<<kind<<','<<start<<','<<unlock_calls<<'\n';++cases;compared+=count+GW*GH;
    }
    need(cases==36,"Case count");
    std::cout<<"{\"status\":\"original_map303_gate_writers_executed\",\"cases\":"<<cases<<",\"color_and_gate_cells_checked\":"<<compared<<",\"guard_failures\":0,\"whole_client_executed\":false,\"game_sessions\":0}"<<std::endl;
    return 0;
 }catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}
}
