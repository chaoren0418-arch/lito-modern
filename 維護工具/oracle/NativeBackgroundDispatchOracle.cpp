// Original first-pass object selector with private recording callees.
// No actual object pixels or product code are executed here.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
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
static_assert(sizeof(void*)==4,"x86 selector only");
using U=uint32_t;using B=unsigned char;
[[noreturn]] static void die(const char* m){std::fprintf(stderr,"dispatch guard: %s\n",m);ExitProcess(77);}
static void need(bool b,const char* m){if(!b)die(m);}
static U addr(const void* p){return reinterpret_cast<U>(p);}
static constexpr std::array<U,10> keys{0x768894,0x8300e0,0x768898,0x768890,0x76888c,0x8300e4,0x8300ec,0x75ac40,0x503524,0x62f324};
struct Slot {U before=0x11223344,value=0,after=0x55667788;};
static std::array<Slot,10> slots;
static U& slot(U key){for(size_t i=0;i<keys.size();++i)if(keys[i]==key)return slots[i].value;die("unknown selector global");}
struct Source {U before=0x12345678;std::array<B,8> bytes{};U after=0x87654321;};
static Source source;
struct Record {U route=0,a=0,b=0,c=0;};
static Record recorded;
static void check(const void* p,int y,int pitch){need(p==source.bytes.data()&&y==0x2222&&pitch==744,"callee registers/arguments");need(recorded.route==0,"multiple selector calls");}
static void __fastcall r1(const void* p,int y,int pitch){check(p,y,pitch);recorded.route=1;}
static void __fastcall r2(const void* p,int y,int pitch){check(p,y,pitch);recorded.route=2;}
static void __fastcall r3(const void* p,int y,int pitch){check(p,y,pitch);recorded.route=3;}
static void __fastcall r4(const void* p,int y,int pitch){check(p,y,pitch);recorded.route=4;}
static void __fastcall r5(const void* p,int y,int pitch,U a,U b){check(p,y,pitch);recorded={5,a,b,0};}
static void __fastcall r6(const void* p,int y,int pitch,U a){check(p,y,pitch);recorded={6,a,0,0};}
static void __fastcall plain(const void* p,int y,int pitch,U a,U b){check(p,y,pitch);recorded={7,a,b,0};}
static void __fastcall r8(const void* p,int y,int pitch,U a){check(p,y,pitch);recorded={8,a,0,0};}
[[noreturn]] static void __fastcall forbidden(){die("null or invalid source error branch");}
static void* fragment_fn=nullptr;
static void* frame_pointer=nullptr;
__declspec(naked) static void invoke(){
    __asm {
        push ebp
        push ebx
        push esi
        push edi
        mov ebp,frame_pointer
        mov ebx,744
        mov esi,2222h
        mov edi,3333h
        call dword ptr [fragment_fn]
        pop edi
        pop esi
        pop ebx
        pop ebp
        ret
    }
}
static std::vector<B> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);need(bool(f),"missing input");return {(std::istreambuf_iterator<char>(f)),{}};}
static void write(const std::filesystem::path& p,const void* b,size_t n){std::ofstream f(p,std::ios::binary);f.write(static_cast<const char*>(b),n);need(bool(f),"output write");}
static std::vector<int64_t> row(const std::string& line){std::istringstream f(line);std::string s;std::vector<int64_t> v;while(std::getline(f,s,','))v.push_back(std::stoll(s));return v;}
int wmain(int argc,wchar_t** argv){
    need(argc==2,"preparation directory");std::filesystem::path out(argv[1]);auto original=read(out/"selector-original.bin");need(original.size()==389,"original selector length");
    auto code=original;code.push_back(0xc3);auto memory=static_cast<B*>(VirtualAlloc(nullptr,code.size(),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));need(memory!=nullptr,"code allocation");
    std::ofstream addresses(out/"private-addresses.csv");addresses<<"kind,key,address\nB,0,"<<addr(memory)<<'\n';
    std::ifstream relocation(out/"relocations.csv");need(bool(relocation),"relocation input");std::string line;U patches=0;
    while(std::getline(relocation,line)){
        std::istringstream f(line);std::string kind,o,k;std::getline(f,kind,',');std::getline(f,o,',');std::getline(f,k,',');U offset=std::stoul(o),key=std::stoul(k),old=0,value=0;
        need(offset+4<=original.size(),"relocation bounds");std::memcpy(&old,original.data()+offset,4);
        if(kind=="D"){need(old==key,"data operand");value=addr(&slot(key));}
        else {need(static_cast<U>(0x488d0b+offset+4+old)==key,"relative target");need((kind=="C"&&key==0x4be000)||(kind=="J"&&key==0x48af4b),"external target");value=addr(kind=="C"?reinterpret_cast<void*>(&plain):reinterpret_cast<void*>(&forbidden))-addr(memory+offset+4);}
        std::memcpy(code.data()+offset,&value,4);addresses<<kind<<','<<key<<','<<(kind=="D"?value:addr(memory+offset+4)+value)<<'\n';++patches;
    }
    addresses.close();need(bool(addresses),"address flush");std::memcpy(memory,code.data(),code.size());DWORD previous=0;need(VirtualProtect(memory,code.size(),PAGE_EXECUTE_READ,&previous)!=0,"code protection");FlushInstructionCache(GetCurrentProcess(),memory,code.size());write(out/"selector-relocated.bin",code.data(),code.size());fragment_fn=memory;
    slot(0x768894)=addr(reinterpret_cast<void*>(&r1));slot(0x8300e0)=addr(reinterpret_cast<void*>(&r2));slot(0x768898)=addr(reinterpret_cast<void*>(&r3));slot(0x768890)=addr(reinterpret_cast<void*>(&r4));
    slot(0x76888c)=addr(reinterpret_cast<void*>(&r5));slot(0x8300e4)=addr(reinterpret_cast<void*>(&r6));slot(0x8300ec)=addr(reinterpret_cast<void*>(&r8));slot(0x503524)=0x10203040;
    std::ifstream cases(out/"cases.csv");need(bool(cases),"case input");std::getline(cases,line);
    std::ofstream results(out/"records-native.bin",std::ios::binary);U count=0;std::array<U,9> routes{};
    while(std::getline(cases,line)){
        const auto c=row(line);need(c.size()==7&&c[0]==count,"case sequence");
        std::array<B,640> frame;frame.fill(0xa5);B* ebp=frame.data()+576;frame_pointer=ebp;
        auto put=[&](U offset,U v){std::memcpy(ebp-offset,&v,4);};
        U flags=static_cast<U>(c[1]),data=addr(source.bytes.data());B shade=static_cast<B>(c[5]);
        source.bytes.fill(0);if(c[4]>0){need(c[4]<=8,"header variation");source.bytes[(c[4]-1)%4]=c[4]<=4?0x80:0xff;}
        put(0x28,addr(&data));put(4,addr(&flags));put(0x1c,addr(&shade));put(0x24,(c[3]&16)?1:0);put(0x1a4,0x31415926);put(0x90,0x3333);
        ebp[-0x2e]=(c[3]&1)?1:0;ebp[-0x1e]=(c[3]&2)?1:0;ebp[-0x2d]=(c[3]&4)?1:0;ebp[-0x1d]=(c[3]&8)?1:0;
        slot(0x62f324)=static_cast<U>(c[2]);slot(0x75ac40)=static_cast<U>(c[6]);recorded={};
        const auto frame_before=frame;const auto source_before=source;const auto slots_before=slots;
        invoke();
        need(frame==frame_before&&std::memcmp(&source,&source_before,sizeof(source))==0&&std::memcmp(&slots,&slots_before,sizeof(slots))==0,"private input changed");
        need(flags==static_cast<U>(c[1])&&data==addr(source.bytes.data())&&shade==static_cast<B>(c[5]),"input scalar changed");need(recorded.route<9,"route domain");
        results.write(reinterpret_cast<const char*>(&recorded),sizeof(recorded));++routes[recorded.route];++count;
    }
    results.close();need(bool(results)&&count>262144,"result completion");
    std::cout<<"{\"status\":\"original_first_pass_selector_ran_with_recording_callees\",\"cases\":"<<count<<",\"original_bytes\":389,\"relocations\":"<<patches<<",\"guards_unchanged\":true,\"object_pixels_executed\":false}\n";
    VirtualFree(memory,0,MEM_RELEASE);return 0;
}
