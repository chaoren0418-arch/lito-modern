// Private x86 initialization/background kernels. Never linked into the product.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>
static_assert(sizeof(void*)==4,"Original kernels require x86");
using U=uint32_t;using W=uint16_t;
static constexpr U GB=0x630000,GE=0x832000,GUARD=64,HEIGHT=552,GATE_WIDTH=744;
[[noreturn]] static void die(const char* s){std::fprintf(stderr,"oracle guard: %s\n",s);ExitProcess(77);}
static void need(bool b,const char* s){if(!b)die(s);}
static U addr(const void* p){return reinterpret_cast<U>(p);}
static std::vector<unsigned char> arena;
static std::array<unsigned char,16> constants;
static void* host_ftol=nullptr;
static U ftol_calls=0;
__declspec(naked) static void ftol_proxy(){
    __asm {
        inc ftol_calls
        jmp dword ptr [host_ftol]
    }
}
[[noreturn]] static void __fastcall forbidden(){die("unexpected original error/restore path");}
static unsigned char* global(U address){
    if(GB<=address&&address<GE)return arena.data()+GUARD+address-GB;
    if(0x4fa328<=address&&address<0x4fa338)return constants.data()+address-0x4fa328;
    die("unmapped original data address");
}
static U get(U address){U v=0;std::memcpy(&v,global(address),4);return v;}
static void set(U address,U value){std::memcpy(global(address),&value,4);}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);need(bool(f),"input missing");return {(std::istreambuf_iterator<char>(f)),{}};}
static void write(const std::filesystem::path& p,const void* b,size_t n){std::ofstream f(p,std::ios::binary);f.write(static_cast<const char*>(b),n);need(bool(f),"binary output");}
static std::vector<int> row(const std::string& s){std::istringstream f(s);std::string p;std::vector<int> r;while(std::getline(f,p,','))r.push_back(std::stoi(p));return r;}
static void* load_code(const std::filesystem::path& out,const char* name,std::ofstream& addresses){
    auto code=read(out/(std::string(name)+"-original.bin"));
    auto memory=static_cast<unsigned char*>(VirtualAlloc(nullptr,code.size(),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));need(memory!=nullptr,"code allocation");
    addresses<<name<<",B,0,"<<addr(memory)<<'\n';
    std::ifstream f(out/(std::string(name)+"-relocations.csv"));need(bool(f),"relocation input");std::string line;
    while(std::getline(f,line)){
        std::istringstream s(line);std::string kind,ot,kt;std::getline(s,kind,',');std::getline(s,ot,',');std::getline(s,kt,',');U offset=std::stoul(ot),key=std::stoul(kt),value=0,old=0;need(offset+4<=code.size(),"relocation bounds");std::memcpy(&old,code.data()+offset,4);
        if(kind=="D"||kind=="I"){need(old==key,"original data operand");value=addr(global(key));}
        else if(kind=="F"){need(key==0x4f1592,"conversion call target");value=addr(reinterpret_cast<void*>(&ftol_proxy))-addr(memory+offset+4);}
        else {need(kind=="E"&&key==0x5091863,"error call target");value=addr(reinterpret_cast<void*>(&forbidden))-addr(memory+offset+4);}
        if(kind=="F"||kind=="E"){
            U original_start=std::string(name)=="init"?0x4c18e0:std::string(name)=="rgb565"?0x4bf0b0:0x4bf500;
            need(static_cast<U>(original_start+offset+4+old)==key,"original relative call");
        }
        std::memcpy(code.data()+offset,&value,4);addresses<<name<<','<<kind<<','<<offset<<','<<(kind=="F"?addr(reinterpret_cast<void*>(&ftol_proxy)):kind=="E"?addr(reinterpret_cast<void*>(&forbidden)):value)<<'\n';
    }
    std::memcpy(memory,code.data(),code.size());DWORD prior=0;need(VirtualProtect(memory,code.size(),PAGE_EXECUTE_READ,&prior)!=0,"code protection");FlushInstructionCache(GetCurrentProcess(),memory,code.size());write(out/(std::string(name)+"-relocated.bin"),code.data(),code.size());return memory;
}
struct Surface {void** table;W* frame;U pitch,height,locks=0,unlocks=0;};
static std::array<void*,33> vtable{};
static int __stdcall unlock(Surface* s,void* rect){need(s&&rect==nullptr,"unlock arguments");++s->unlocks;return 0;}
static int __stdcall lock(Surface* s,void* rect,U* desc,U flags,void* event){
    need(s&&rect==nullptr&&desc&&desc[0]==0x6c&&flags==1&&event==nullptr,"lock arguments");
    desc[2]=s->height;desc[3]=s->pitch;desc[4]=s->pitch*2;desc[9]=addr(s->frame);++s->locks;return 0;
}
struct Image {int width,height;std::vector<unsigned char> bytes;unsigned char* data(){return bytes.data()+GUARD;}};
static Image image(int width,int height,int mode,bool overlay){
    Image result{width,height,std::vector<unsigned char>(GUARD+8+width*height*2+GUARD,0xa5)};
    std::memset(result.data(),0,8);W w=static_cast<W>(width),h=static_cast<W>(height);std::memcpy(result.data(),&w,2);std::memcpy(result.data()+2,&h,2);
    for(int y=0;y<height;++y)for(int x=0;x<width;++x){
        int r=overlay?(5*x+3*y)%32:x%32,g=overlay?(7*x+y)%32:y%32,b=overlay?(x+11*y)%32:(x+3*y)%32;
        W color=static_cast<W>((r<<(mode?11:10))|(g<<(mode?6:5))|b);
        if(mode&&((x+y)&1))color|=0x20; // Exercise the native five-bit green quantizer.
        std::memcpy(result.data()+8+(y*width+x)*2,&color,2);
    }return result;
}
struct Rect {int left,top,right,bottom;};
using Init=void(__cdecl*)();
using Kernel=void(__fastcall*)(const void*,Surface*,Rect*,int,int,const void*,int,int,int,int);
static void arena_guards(){for(U i=0;i<GUARD;++i)need(arena[i]==0xa5&&arena[GUARD+GE-GB+i]==0xa5,"data arena guard");}
int wmain(int argc,wchar_t** argv){
    need(argc==2,"expected preparation directory");std::filesystem::path out(argv[1]);
    auto initial=read(out/"data-initial.bin");need(initial.size()==GE-GB,"data arena length");arena.assign(GUARD+GE-GB+GUARD,0xa5);
    auto constant_input=read(out/"screen-constants.bin");need(constant_input.size()==constants.size(),"constant length");std::memcpy(constants.data(),constant_input.data(),constants.size());
    HMODULE crt=LoadLibraryW(L"C:\\Windows\\SysWOW64\\msvcrt.dll");need(crt!=nullptr,"host x86 CRT load");host_ftol=reinterpret_cast<void*>(GetProcAddress(crt,"_ftol"));need(host_ftol!=nullptr,"host conversion export");
    wchar_t module_path[1024]{};need(GetModuleFileNameW(crt,module_path,1024)>0,"loaded CRT path");
    {std::ofstream p(out/"host-CRT-path.txt");for(const wchar_t* c=module_path;*c;++c){need(*c<128,"CRT path ASCII");p<<static_cast<char>(*c);}p<<'\n';need(bool(p),"CRT provenance output");}
    HANDLE module_file=CreateFileW(module_path,GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,nullptr,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,nullptr);need(module_file!=INVALID_HANDLE_VALUE,"module file identity handle");
    wchar_t final_path[1024]{};DWORD path_size=GetFinalPathNameByHandleW(module_file,final_path,1024,FILE_NAME_NORMALIZED|VOLUME_NAME_DOS);CloseHandle(module_file);need(path_size>0&&path_size<1024,"physical module path");
    {std::ofstream p(out/"host-CRT-final-path.txt");for(const wchar_t* c=final_path;*c;++c){need(*c<128,"physical CRT path ASCII");p<<static_cast<char>(*c);}p<<'\n';need(bool(p),"physical CRT path output");}
    const auto module_bytes=reinterpret_cast<const unsigned char*>(crt);U pe_offset=0;std::memcpy(&pe_offset,module_bytes+60,4);W machine=0,magic=0;U image_size=0;std::memcpy(&machine,module_bytes+pe_offset+4,2);std::memcpy(&magic,module_bytes+pe_offset+24,2);std::memcpy(&image_size,module_bytes+pe_offset+24+56,4);need(machine==0x14c&&magic==0x10b,"loaded x86 CRT PE header");
    U ftol_rva=addr(host_ftol)-addr(crt);need(ftol_rva<image_size-48,"loaded conversion RVA");write(out/"host-ftol-loaded.bin",host_ftol,48);
    {std::ofstream p(out/"host-CRT-module.json");p<<"{\"machine\":"<<machine<<",\"optional_header_magic\":"<<magic<<",\"image_size\":"<<image_size<<",\"ftol_RVA\":"<<ftol_rva<<",\"captured_ftol_bytes\":48}\n";need(bool(p),"loaded CRT metadata output");}
    std::ofstream addresses(out/"private-addresses.csv");addresses<<"function,kind,offset,address\n";
    auto init=reinterpret_cast<Init>(load_code(out,"init",addresses));auto rgb565=reinterpret_cast<Kernel>(load_code(out,"rgb565",addresses));auto rgb555=reinterpret_cast<Kernel>(load_code(out,"rgb555",addresses));addresses.close();need(bool(addresses),"address flush");
    vtable[25]=reinterpret_cast<void*>(&lock);vtable[27]=reinterpret_cast<void*>(&forbidden);vtable[32]=reinterpret_cast<void*>(&unlock);
    std::ifstream cases(out/"cases.csv");need(bool(cases),"cases input");std::string line;std::getline(cases,line);std::vector<std::vector<int>> records;while(std::getline(cases,line))records.push_back(row(line));need(records.size()==1584,"case count");
    std::ofstream changes(out/"pixels-native.bin",std::ios::binary),results(out/"results-native.csv");need(bool(changes)&&bool(results),"result output");results<<"case,bottom,locks,unlocks,changes\n";U case_count=0,total_changes=0,total_ftol=0;
    for(int mode=0;mode<2;++mode){
        std::memcpy(global(GB),initial.data(),initial.size());*global(0x641298)=static_cast<unsigned char>(mode);ftol_calls=0;init();need(ftol_calls==1024,"original conversion call count");total_ftol+=ftol_calls;arena_guards();
        need(get(0x768878)==static_cast<U>(mode?0x4bf0b0:0x4bf500),"original background dispatch");
        write(out/("initialized-mode"+std::to_string(mode)+".bin"),global(GB),GE-GB);
        auto base=image(32,32,mode,false),overlay=image(64,64,mode,true);auto base_copy=base.bytes,overlay_copy=overlay.bytes;
        for(const auto& c:records){
            need(c.size()==15&&c[14]==1,"case shape/version");if(c[1]!=mode)continue;need(c[0]==static_cast<int>(case_count),"case order");
            int pitch=c[2];std::vector<W> frame(GUARD+pitch*HEIGHT+GUARD,0xa55a);W* target=frame.data()+GUARD;std::fill(target,target+pitch*HEIGHT,0x1777);
            std::vector<unsigned char> gate(GUARD+GATE_WIDTH*HEIGHT+GUARD,0x5a);auto plane=gate.data()+GUARD;
            for(U y=0;y<HEIGHT;++y)for(U x=0;x<GATE_WIDTH;++x)plane[y*GATE_WIDTH+x]=c[5]==0?0:c[5]==1?1:((x+y)%2?7:0);
            auto gate_copy=gate;Surface surface{vtable.data(),target,static_cast<U>(pitch),HEIGHT};set(0x6476c4,addr(plane));set(0x831560,addr(target+pitch*HEIGHT));auto arena_copy=arena;
            Rect rect{c[10],c[11],c[12],c[13]};(mode?rgb565:rgb555)(base.data(),&surface,&rect,c[8],c[9],overlay.data(),c[6],c[7],c[3],c[4]);
            need(surface.locks==1&&surface.unlocks==2,"surface lifecycle");need(gate==gate_copy&&base.bytes==base_copy&&overlay.bytes==overlay_copy,"source/gate changed");need(arena==arena_copy,"lookup/global state changed");
            for(U i=0;i<GUARD;++i)need(frame[i]==0xa55a&&frame[GUARD+pitch*HEIGHT+i]==0xa55a,"frame guard");
            U count=0;for(U i=0;i<static_cast<U>(pitch)*HEIGHT;++i)if(target[i]!=0x1777)++count;
            U number=static_cast<U>(c[0]);changes.write(reinterpret_cast<const char*>(&number),4);changes.write(reinterpret_cast<const char*>(&count),4);
            for(U i=0;i<static_cast<U>(pitch)*HEIGHT;++i)if(target[i]!=0x1777){changes.write(reinterpret_cast<const char*>(&i),4);changes.write(reinterpret_cast<const char*>(&target[i]),2);}
            results<<c[0]<<','<<rect.bottom<<','<<surface.locks<<','<<surface.unlocks<<','<<count<<'\n';total_changes+=count;++case_count;
        }
    }
    changes.close();results.close();need(bool(changes)&&bool(results)&&case_count==1584,"result completion");
    std::cout<<"{\"status\":\"original_background_initializer_and_pixel_kernels_ran_with_host_ftol\",\"cases\":"<<case_count<<",\"changed_words\":"<<total_changes<<",\"host_ftol_calls\":"<<total_ftol<<",\"lock_calls\":"<<case_count<<",\"unlock_calls\":"<<case_count*2<<",\"guards_unchanged\":true,\"historical_CRT_DLL_identity_verified\":false}\n";
    FreeLibrary(crt);return 0;
}
