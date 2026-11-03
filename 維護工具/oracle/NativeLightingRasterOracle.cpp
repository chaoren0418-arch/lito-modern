// Original raster kernels with address-only private data relocations.
// Channel/remap tables are declared synthetic fixtures, not original init proof.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*) == 4, "Original kernels are x86");
static std::array<uint16_t,1024> blue{},green{},red{};
static std::array<unsigned char,32> remap{};
static unsigned char* gate_pointer;
using Kernel=void(__fastcall*)(uint16_t*,const unsigned char*,int,int,int,unsigned char);
static constexpr int W=640,H=408,MW=320,MH=204,PITCH=800,X=33,Y=3,FH=414,GW=744;
static uint64_t checked=0,exhaustive=0,guard_checked=0,calls=0;

static void relocate(unsigned char* code,size_t bytes,uint32_t old,uintptr_t replacement,int expected) {
    int found=0;
    for(size_t i=0;i+4<=bytes;++i){uint32_t v;memcpy(&v,code+i,4);
        if(v==old){memcpy(code+i,&replacement,4);++found;i+=3;}}
    if(found!=expected)throw std::runtime_error("Unexpected data operand count");
}
static Kernel load(const std::filesystem::path& source,const std::filesystem::path& destination,size_t bytes) {
    std::ifstream file(source,std::ios::binary);
    std::vector<unsigned char> data((std::istreambuf_iterator<char>(file)),{});
    if(!file || data.size()!=bytes)throw std::runtime_error("Kernel extent differs");
    auto code=static_cast<unsigned char*>(VirtualAlloc(nullptr,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));
    if(!code)throw std::runtime_error("Private allocation failed");
    memcpy(code,data.data(),bytes);
    relocate(code,bytes,0x6476c4,reinterpret_cast<uintptr_t>(&gate_pointer),1);
    relocate(code,bytes,0x83009c,reinterpret_cast<uintptr_t>(remap.data()),1);
    relocate(code,bytes,0x80d89c,reinterpret_cast<uintptr_t>(blue.data()),2);
    relocate(code,bytes,0x80e09c,reinterpret_cast<uintptr_t>(green.data()),2);
    relocate(code,bytes,0x80e89c,reinterpret_cast<uintptr_t>(red.data()),2);
    std::ofstream out(destination,std::ios::binary);out.write(reinterpret_cast<char*>(code),bytes);out.close();
    if(!out)throw std::runtime_error("Relocated kernel retention failed");
    DWORD old;
    if(!VirtualProtect(code,65536,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),code,65536))
        throw std::runtime_error("Private code protection failed");
    return reinterpret_cast<Kernel>(code);
}
static uint16_t scalar(uint16_t color,unsigned char raw,bool mode,bool gated,unsigned char gate) {
    const unsigned light=remap[raw/8];
    if(light==31 || (gated && gate==0))return color;
    const unsigned r=(color>>(mode?11:10))&31,g=(color>>(mode?6:5))&31,b=color&31;
    return blue[b*32+light]|green[g*32+light]|red[r*32+light];
}
static bool sample_color(uint16_t c) {
    switch(c){case 0:case 1:case 15:case 31:case 32:case 63:case 64:case 255:
    case 1023:case 1024:case 2047:case 2048:case 0x3fff:case 0x7fff:case 0x8000:case 0xfffe:case 0xffff:return true;default:return false;}
}
int wmain(int argc,wchar_t** argv) {
    try {
        if(argc!=6)throw std::runtime_error("Expected555.bin565.bin relocated555 relocated565 vectors.csv");
        auto k555=load(argv[1],argv[3],399),k565=load(argv[2],argv[4],393);
        std::ofstream vectors(std::filesystem::path(argv[5]),std::ios::binary);
        vectors<<"mode,pass,raw,color,gate,mapped,actual\n";
        std::vector<unsigned char> mask(MW*MH),gate(GW*FH),saved_mask;
        std::vector<uint16_t> pixels(PITCH*FH),expected,initial;
        gate_pointer=gate.data();uint64_t retained=0;
        for(int mode=0;mode<2;++mode)for(int pass=0;pass<3;++pass) {
            for(unsigned i=0;i<32;++i)remap[i]=static_cast<unsigned char>(pass?((i*13+7)&31):i);
            for(unsigned c=0;c<32;++c)for(unsigned l=0;l<32;++l) {
                const auto b=pass?((c*17+l*7+3)&31):c*l/31;
                const auto g=pass?((c*17+l*7+11)&31):c*l/31;
                const auto r=pass?((c*17+l*7+19)&31):c*l/31;
                blue[c*32+l]=static_cast<uint16_t>(b);
                green[c*32+l]=static_cast<uint16_t>(g<<(mode?6:5));
                red[c*32+l]=static_cast<uint16_t>(r<<(mode?11:10));
            }
            for(size_t i=0;i<gate.size();++i)gate[i]=static_cast<unsigned char>(pass==2?0:((i*3+1)%5!=0));
            const int frames=pass?1:65;
            for(int frame=0;frame<frames;++frame) {
                for(size_t i=0;i<pixels.size();++i)pixels[i]=static_cast<uint16_t>((i*97+0x5a35)&65535);
                expected=pixels;
                for(int cell=0;cell<MW*MH;++cell) {
                    const uint64_t n=static_cast<uint64_t>(frame)*MW*MH+cell;
                    const bool wanted=n<(1u<<22);
                    const unsigned char raw=static_cast<unsigned char>(pass?(cell*37)&255:(wanted?n/16384:255));
                    mask[cell]=raw;
                    for(int q=0;q<4;++q) {
                        const int xx=(cell%MW)*2+(q&1),yy=(cell/MW)*2+(q>>1);
                        const size_t at=(Y+yy)*PITCH+X+xx,ga=(Y+yy)*GW+X+xx;
                        const uint16_t color=static_cast<uint16_t>(pass?(cell*7919+q*1231)&65535:(n*4+q)&65535);
                        pixels[at]=color;expected[at]=scalar(color,raw,mode!=0,pass!=0,gate[ga]);
                        if(!pass && wanted)++exhaustive;
                    }
                }
                initial=pixels;saved_mask=mask;
                (mode?k565:k555)(pixels.data(),mask.data(),PITCH*2,X*2,Y,static_cast<unsigned char>(pass!=0));
                ++calls;
                if(mask!=saved_mask)throw std::runtime_error("Input mask changed");
                for(int yy=0;yy<FH;++yy)for(int xx=0;xx<PITCH;++xx) {
                    const size_t at=yy*PITCH+xx;
                    if(pixels[at]!=expected[at]) {
                        std::cerr<<"Mismatch mode="<<mode<<" pass="<<pass<<" frame="<<frame<<" x="<<xx<<" y="<<yy<<" actual="<<pixels[at]<<" expected="<<expected[at]<<'\n';
                        return 2;
                    }
                    if(yy>=Y && yy<Y+H && xx>=X && xx<X+W)++checked;else++guard_checked;
                }
                for(int cell=0;cell<MW*MH;++cell)for(int q=0;q<4;++q) {
                    const uint64_t n=static_cast<uint64_t>(frame)*MW*MH+cell;
                    const int xx=(cell%MW)*2+(q&1),yy=(cell/MW)*2+(q>>1);
                    const size_t at=(Y+yy)*PITCH+X+xx,ga=(Y+yy)*GW+X+xx;
                    if((!pass && n<(1u<<22) && sample_color(initial[at])) || (pass && cell%MW==0)) {
                        vectors<<mode<<','<<pass<<','<<unsigned(mask[cell])<<','<<initial[at]<<','<<unsigned(gate[ga])<<','<<unsigned(remap[mask[cell]/8])<<','<<pixels[at]<<'\n';++retained;
                    }
                }
            }
        }
        vectors.close();if(!vectors)throw std::runtime_error("Vector output failed");
        VirtualFree(reinterpret_cast<void*>(k555),0,MEM_RELEASE);VirtualFree(reinterpret_cast<void*>(k565),0,MEM_RELEASE);
        std::cout<<"{\"status\":\"original_raster_kernels_match_scalar_for_synthetic_tables\",\"native_calls\":"<<calls<<",\"exhaustive_color_raw_pairs\":"<<exhaustive<<",\"viewport_pixels_checked\":"<<checked<<",\"guard_pixels_checked\":"<<guard_checked<<",\"retained_vectors\":"<<retained<<",\"original_table_generation_verified\":false,\"game_sessions\":0,\"full_world_framebuffer_verified\":false}\n";
        return 0;
    }catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}
}
