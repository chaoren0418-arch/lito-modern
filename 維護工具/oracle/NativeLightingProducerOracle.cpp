// Original private x86 mask-producer graph. Actor/catalog tables are explicit
// fixtures; no game process, network session or original global memory is used.
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
static_assert(sizeof(void*)==4,"Isolated original x86 graph only");
static constexpr size_t PIXELS=320*204, GUARD=64;
static std::array<unsigned char,PIXELS+2*GUARD> guarded_mask{};
static unsigned char* mask_pointer=guarded_mask.data()+GUARD;
static std::vector<unsigned char> lmp;
static std::array<unsigned char*,16> stencil_pointers{};
static std::array<int,32> dimensions{};
static std::array<unsigned char,512> bank_normal{},bank_alternate{};
static std::array<unsigned char,32> remap{};
static std::array<unsigned char,1920> offsets{};
static std::array<unsigned char,256> fold{},lead{};
static std::array<unsigned char,128> classes{},subtypes{};
static unsigned char* class_pointer=classes.data();
static unsigned char* subtype_pointer=subtypes.data();
static std::array<unsigned char,128*71*12> frames{};
static unsigned char* frame_pointer=frames.data();
static std::array<std::array<unsigned char,128>,128> actors{};
static std::array<unsigned char*,128> actor_pointers{};
static unsigned char** actor_list=actor_pointers.data();
static unsigned char* own_actor=actors[0].data();
static const char own_name[]="OWNER",same_owner[]="owner",other_owner[]="Other",empty_name[]="";
static int origin_u,origin_v,camera_x,camera_y,actor_count,cached_level;
static uint32_t blind_index=0xb104fa87,blind_value,blind_key=0;
static uint32_t* blind_values=&blind_value;
static unsigned char underwater;
struct Blit {int x,y,w,h,offset;};
static std::array<Blit,128> blits{};static uint32_t blit_count;
using Blend=void(__cdecl*)(unsigned char*,const unsigned char*,int,int,int,int);
static Blend original_blend;
static void __cdecl observe_blend(unsigned char* dst,const unsigned char* src,int x,int y,int w,int h){
    if(blit_count>=blits.size())std::abort();
    blits[blit_count++]={x,y,w,h,static_cast<int>(src-lmp.data())};
    original_blend(dst,src,x,y,w,h);
}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);if(!f)throw std::runtime_error("Missing graph input");return {(std::istreambuf_iterator<char>(f)),{}};}
template<class T>static T get(std::istream& f){T v{};f.read(reinterpret_cast<char*>(&v),sizeof(v));if(!f)throw std::runtime_error("Truncated graph commands");return v;}
static uintptr_t address(uint32_t old){
    if(old>=0x62e7e8&&old<0x62ef68)return reinterpret_cast<uintptr_t>(offsets.data()+old-0x62e7e8);
    if(old>=0x75ab60&&old<0x75abe0)return reinterpret_cast<uintptr_t>(dimensions.data())+old-0x75ab60;
    switch(old){
    case 0x75abe8:return reinterpret_cast<uintptr_t>(&mask_pointer);
    case 0x75ab20:return reinterpret_cast<uintptr_t>(stencil_pointers.data());
    case 0x75a998:return reinterpret_cast<uintptr_t>(&camera_x);case 0x75a99c:return reinterpret_cast<uintptr_t>(&camera_y);
    case 0x75aa50:return reinterpret_cast<uintptr_t>(&origin_u);case 0x75aa54:return reinterpret_cast<uintptr_t>(&origin_v);
    case 0x834ee4:return reinterpret_cast<uintptr_t>(&own_actor);case 0x834f08:return reinterpret_cast<uintptr_t>(&actor_count);case 0x834f10:return reinterpret_cast<uintptr_t>(&actor_list);
    case 0x831690:return reinterpret_cast<uintptr_t>(&blind_index);case 0x831694:return reinterpret_cast<uintptr_t>(&blind_values);case 0x831698:return reinterpret_cast<uintptr_t>(&blind_key);
    case 0x64bc2d:return reinterpret_cast<uintptr_t>(&underwater);
    case 0x6348a4:return reinterpret_cast<uintptr_t>(&cached_level);case 0x6348a8:return reinterpret_cast<uintptr_t>(bank_normal.data());case 0x634aa8:return reinterpret_cast<uintptr_t>(bank_alternate.data());case 0x83009c:return reinterpret_cast<uintptr_t>(remap.data());
    case 0x647e74:return reinterpret_cast<uintptr_t>(&class_pointer);case 0x647e64:return reinterpret_cast<uintptr_t>(&subtype_pointer);case 0x834e70:return reinterpret_cast<uintptr_t>(&frame_pointer);
    case 0x834960:return reinterpret_cast<uintptr_t>(fold.data());case 0x834a60:return reinterpret_cast<uintptr_t>(lead.data());
    default:throw std::runtime_error("Unreviewed original global operand");}
}
static void put(std::array<unsigned char,128>& a,size_t off,int32_t v){memcpy(a.data()+off,&v,4);}
int wmain(int argc,wchar_t** argv){try{
    SetErrorMode(SEM_FAILCRITICALERRORS|SEM_NOGPFAULTERRORBOX|SEM_NOOPENFILEERRORBOX);
    if(argc!=2)throw std::runtime_error("Expected owned graph evidence directory");const std::filesystem::path dir(argv[1]);
    lmp=read(dir/L"source.lmp");if(lmp.size()!=152736)throw std::runtime_error("LMP extent");const auto saved_lmp=lmp;
    auto normal=read(dir/L"bank-normal.bin"),alternate=read(dir/L"bank-alternate.bin"),offset_data=read(dir/L"offset-tables.bin");
    if(normal.size()!=512||alternate.size()!=512||offset_data.size()!=1920)throw std::runtime_error("Original tables extent");
    memcpy(bank_normal.data(),normal.data(),512);memcpy(bank_alternate.data(),alternate.data(),512);memcpy(offsets.data(),offset_data.data(),1920);
    for(int i=0;i<256;++i)fold[i]=static_cast<unsigned char>(i>='A'&&i<='Z'?i+32:i);lead.fill(0);
    for(int i=0;i<16;++i){uint32_t off;memcpy(&off,lmp.data()+i*4,4);memcpy(dimensions.data()+i*2,lmp.data()+64+i*8,8);stencil_pointers[i]=lmp.data()+off;}
    auto image=read(dir/L"graph-original.bin"),relocations=read(dir/L"relocations.bin");if(image.size()!=0x100000||relocations.size()%8)throw std::runtime_error("Graph/relocation extent");
    auto code=static_cast<unsigned char*>(VirtualAlloc(nullptr,image.size(),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));if(!code)throw std::runtime_error("Private code allocation");memcpy(code,image.data(),image.size());
    for(size_t i=0;i<relocations.size();i+=8){uint32_t at,old,current;memcpy(&at,relocations.data()+i,4);memcpy(&old,relocations.data()+i+4,4);if(at+4>image.size())throw std::runtime_error("Relocation bound");memcpy(&current,code+at,4);if(current!=old)throw std::runtime_error("Relocation operand changed");auto value=address(old);memcpy(code+at,&value,4);}
    // Observe calls at the old blit entry, then execute its unmodified code copy.
    original_blend=reinterpret_cast<Blend>(code+0x80000);code[0x3b170]=0xe9;int32_t jump=static_cast<int32_t>(reinterpret_cast<uintptr_t>(&observe_blend)-reinterpret_cast<uintptr_t>(code+0x3b170+5));memcpy(code+0x3b171,&jump,4);
    DWORD previous;if(!VirtualProtect(code,image.size(),PAGE_EXECUTE_READ,&previous)||!FlushInstructionCache(GetCurrentProcess(),code,image.size()))throw std::runtime_error("Private executable protection");
    using Producer=void(__fastcall*)(int);auto produce=reinterpret_cast<Producer>(code+0x5a40);
    using Classifier=unsigned char(__fastcall*)(void*);using Predicate=int(__fastcall*)(void*);using Frames=int(__fastcall*)(void*);
    auto classify=reinterpret_cast<Classifier>(code+0x689f0);auto predicate=reinterpret_cast<Predicate>(code+0x68bd0);auto count_frames=reinterpret_cast<Frames>(code+0x68ea0);
    std::ifstream input(dir/L"commands.bin",std::ios::binary);std::ofstream output(dir/L"native-results.bin",std::ios::binary);const uint32_t cases=get<uint32_t>(input);if(cases>4096)throw std::runtime_error("Case budget");
    uint64_t total_blits=0,guard_bytes=0,actor_bytes=0;
    for(uint32_t c=0;c<cases;++c){
        std::array<int32_t,11> h{};for(auto&v:h)v=get<int32_t>(input);const int ambient=h[0];underwater=static_cast<unsigned char>(h[1]);blind_value=h[2];origin_u=h[3];origin_v=h[4];camera_x=h[5];camera_y=h[6];cached_level=h[8];actor_count=h[10];
        if(actor_count<0||actor_count>128||h[1]<0||h[1]>1||blind_value>2)throw std::runtime_error("Fixture bounds");
        guarded_mask.fill(0xa7);for(size_t i=0;i<PIXELS;++i)mask_pointer[i]=static_cast<unsigned char>(i*13+h[7]);for(int i=0;i<32;++i)remap[i]=static_cast<unsigned char>((i*7+h[9])&31);
        actors={};frames.fill(0);classes.fill(1);subtypes.fill(0);blit_count=0;
        for(int i=0;i<actor_count;++i){std::array<int32_t,14> a{};for(auto&v:a)v=get<int32_t>(input);auto&b=actors[i];actor_pointers[i]=b.data();put(b,0x2c,a[0]);put(b,0x30,a[1]);b[0x12]=static_cast<unsigned char>(a[2]);b[0x10]=static_cast<unsigned char>(a[3]);b[0x18]=static_cast<unsigned char>(a[4]);b[0x11]=static_cast<unsigned char>(a[5]);b[0x13]=static_cast<unsigned char>(a[6]);int16_t gfx=static_cast<int16_t>(i);memcpy(b.data()+0x14,&gfx,2);
            if(a[4]<0||a[4]>=71||a[5]<0||a[5]>7||a[6]<0||a[6]>11)throw std::runtime_error("Fixture phase bounds");frames[(i*71+a[4])*12+1]=static_cast<unsigned char>(a[7]);classes[i]=static_cast<unsigned char>(a[8]);subtypes[i]=static_cast<unsigned char>(a[9]);b[0x1f]=static_cast<unsigned char>(a[10]);b[0x50]=static_cast<unsigned char>(a[11]);b[0x23]=static_cast<unsigned char>(a[12]);
            const char* own=own_name;memcpy(b.data()+0x58,&own,4);const char* master=a[13]<0?nullptr:a[13]==0?empty_name:a[13]==1?same_owner:other_owner;memcpy(b.data()+0x64,&master,4);
        }
        const auto saved_actors=actors;produce(ambient);if(saved_actors!=actors||lmp!=saved_lmp)throw std::runtime_error("Original producer changed actor/source input");actor_bytes+=sizeof(actors);
        for(size_t i=0;i<GUARD;++i)if(guarded_mask[i]!=0xa7||guarded_mask[GUARD+PIXELS+i]!=0xa7)throw std::runtime_error("Mask guard changed");guard_bytes+=2*GUARD;
        output.write(reinterpret_cast<const char*>(mask_pointer),PIXELS);output.write(reinterpret_cast<const char*>(remap.data()),32);output.write(reinterpret_cast<const char*>(&cached_level),4);output.write(reinterpret_cast<const char*>(&blit_count),4);output.write(reinterpret_cast<const char*>(blits.data()),blit_count*sizeof(Blit));total_blits+=blit_count;
        for(int i=0;i<actor_count;++i){const unsigned char probe[3]={classify(actors[i].data()),static_cast<unsigned char>(predicate(actors[i].data())),static_cast<unsigned char>(count_frames(actors[i].data()))};output.write(reinterpret_cast<const char*>(probe),3);}
        if(saved_actors!=actors)throw std::runtime_error("Original classifier/query changed actor input");
    }
    if(input.peek()!=EOF||!output)throw std::runtime_error("Trailing commands/output failure");output.close();VirtualFree(code,0,MEM_RELEASE);
    std::cout<<"{\"cases\":"<<cases<<",\"blits\":"<<total_blits<<",\"mask_bytes\":"<<cases*PIXELS<<",\"actor_bytes_unchanged\":"<<actor_bytes<<",\"guard_bytes_checked\":"<<guard_bytes<<"}\n";return 0;
}catch(const std::exception&e){std::cerr<<e.what()<<'\n';return 1;}}
