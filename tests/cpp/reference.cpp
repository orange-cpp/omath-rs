#include <omath/engines/source_engine/camera.hpp>
#include <omath/engines/iw_engine/camera.hpp>
#include <omath/engines/unity_engine/camera.hpp>
#include <omath/engines/unreal_engine/camera.hpp>
#include <omath/engines/frostbite_engine/camera.hpp>
#include <omath/engines/rage_engine/camera.hpp>
#include <omath/engines/serious_engine/camera.hpp>
#include <omath/engines/cry_engine/camera.hpp>
#include <omath/engines/opengl_engine/camera.hpp>
// Independent executable compiled against the unchanged C++ checkout by tools/compare_cpp.py.
#include <omath/linear_algebra/vector3.hpp>
#include <omath/linear_algebra/mat.hpp>
#include <omath/pathfinding/navigation_mesh.hpp>
#include <omath/utility/pattern_scan.hpp>
#include <omath/hud/entity_overlay.hpp>
#include <omath/hud/screen_overlay.hpp>
#include <chrono>
#include <fstream>
#include <iostream>
#include <iomanip>
#include <sstream>
using V=omath::Vector3<float>;
using P=omath::Vector2<float>;
using M=omath::Mat<4,4>;
void point(P p){std::cout<<' '<<p.x<<' '<<p.y;}
void color(const omath::Color& c){auto v=c.value();std::cout<<' '<<v.x<<' '<<v.y<<' '<<v.z<<' '<<v.w;}
struct Recorder:omath::hud::HudRendererInterface {
    void add_line(const P&a,const P&b,const omath::Color&c,float t) override {std::cout<<"line";point(a);point(b);color(c);std::cout<<' '<<t<<'\n';}
    void add_polyline(const std::span<const P>&v,const omath::Color&c,float t) override {std::cout<<"poly "<<v.size();for(auto p:v)point(p);color(c);std::cout<<' '<<t<<'\n';}
    void add_filled_polyline(const std::span<const P>&v,const omath::Color&c) override {std::cout<<"fillpoly "<<v.size();for(auto p:v)point(p);color(c);std::cout<<'\n';}
    void add_rectangle(const P&a,const P&b,const omath::Color&c) override {std::cout<<"rect";point(a);point(b);color(c);std::cout<<'\n';}
    void add_filled_rectangle(const P&a,const P&b,const omath::Color&c) override {std::cout<<"fillrect";point(a);point(b);color(c);std::cout<<'\n';}
    void add_circle(const P&a,float radius,const omath::Color&c,float t,int s) override {std::cout<<"circle";point(a);std::cout<<' '<<radius;color(c);std::cout<<' '<<t<<' '<<s<<'\n';}
    void add_filled_circle(const P&a,float radius,const omath::Color&c,int s) override {std::cout<<"fillcircle";point(a);std::cout<<' '<<radius;color(c);std::cout<<' '<<s<<'\n';}
    void add_arc(const P&a,float radius,float lo,float hi,const omath::Color&c,float t,int s) override {std::cout<<"arc";point(a);std::cout<<' '<<radius<<' '<<lo<<' '<<hi;color(c);std::cout<<' '<<t<<' '<<s<<'\n';}
    void add_image(const std::any&id,const P&a,const P&b,const omath::Color&c) override {std::cout<<"image "<<std::any_cast<std::size_t>(id);point(a);point(b);color(c);std::cout<<'\n';}
    void add_text(const P&a,const omath::Color&c,const std::string_view&text) override {std::cout<<"text "<<text;point(a);color(c);std::cout<<'\n';}
    P calc_text_size(const std::string_view&text) override {return {float(text.size()*8),12};}
};
void hud() {
    using namespace omath::hud;using namespace omath::hud::widget;
    auto r=std::make_shared<Recorder>();const omath::Color red{1,0,0,1},green{0,1,0,1},blue{0,0,1,1},clear{};
    EntityOverlay e{{100,10},{100,110},4,r};
    e.add_2d_box(red).add_right_bar(green,blue,clear,4,0.25f).add_right_label(red,2,Outlined::Off,"HP")
     .add_top_bar(red,blue,clear,3,0.75f).add_centered_top_label(blue,2,Outlined::On,"name")
     .add_left_progress_ring(red,blue,6,0.5f,2,5,16).add_bottom_icon(std::size_t{7},16,16)
     .add_snap_line({0,0},red,1).add_dashed_box(blue,8,5,1).add_skeleton(green);
    ScreenOverlay screen{{800,600},r};
    screen.contents(Crosshair{red},FovCircle{green,40},ThreatArrow{0.5f,blue},HitMarker{red});
}
int main(int argc,char**argv) {
    std::cout<<std::setprecision(9);
    std::string mode=argc>1?argv[1]:"cases";
    if(mode=="nav") {omath::pathfinding::NavigationMesh nav;std::ifstream file(argv[2]);std::stringstream data;data<<file.rdbuf();nav.deserialize(data.str());std::cout<<nav.serialize();return 0;}
    if(mode=="hud"){hud();return 0;}
    if(mode=="bench") {
        for(std::string kernel:{"vector","matrix","scan"}) {
            const int iterations=kernel=="vector"?500000:kernel=="matrix"?100000:256;
            double checksum=0;M a{{1,2,3,4},{5,6,7,8},{9,10,11,12},{13,14,15,16}};
            std::vector<std::byte> bytes(65536);bytes.back()=std::byte{0xab};
            const auto begin=std::chrono::steady_clock::now();
            for(int i=0;i<iterations;++i) {volatile int index=i;const int value=index;
                if(kernel=="vector"){const V v{float(value&1023)/7,float(value%31),float(value%17)};checksum+=v.dot(V{1.25f,-2.5f,0.75f});}
                else if(kernel=="matrix"){a.at(0,0)=float(value%31);const auto result=a*a;checksum+=result.at(0,0)+result.at(3,3);}
                else {auto found=omath::PatternScanner::scan_for_pattern(bytes.begin(),bytes.end(),"00 00 AB");checksum+=std::distance(bytes.begin(),found);}
            }
            const auto ns=std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now()-begin).count();
            std::cout<<kernel<<' '<<iterations<<' '<<ns<<' '<<std::setprecision(17)<<checksum<<'\n';
        }return 0;
    }
    {using namespace omath::source_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera source "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::iw_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera iw "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::unity_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera unity "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::unreal_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<double>{3,4,5},ViewAngles{PitchAngle::from_degrees(double(i-6)),YawAngle::from_degrees(double(i*5)),RollAngle::from_degrees(double(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),double(0.1),double(1000)};const auto target=k_abs_forward*double(100)+k_abs_right*double(i*2)+k_abs_up*double(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera unreal "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::frostbite_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera frostbite "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::rage_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera rage "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::serious_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera serious "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::cry_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera cry "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    {using namespace omath::opengl_engine;for(int i=0;i<12;++i){Camera camera{omath::Vector3<float>{3,4,5},ViewAngles{PitchAngle::from_degrees(float(i-6)),YawAngle::from_degrees(float(i*5)),RollAngle::from_degrees(float(i%3))},omath::projection::ViewPort{800,600},omath::projection::FieldOfView::from_degrees(90),float(0.1),float(1000)};const auto target=k_abs_forward*float(100)+k_abs_right*float(i*2)+k_abs_up*float(i);const auto screen=camera.world_to_screen_unclipped(target);std::cout<<"camera opengl "<<i;if(screen)std::cout<<' '<<screen->x<<' '<<screen->y;else std::cout<<" none";const auto matrix=camera.get_view_matrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)std::cout<<' '<<matrix.at(r,c);std::cout<<'\n';}}
    for(int i=0;i<256;++i){const V a{float(i-128)/7,float(i%31)-15,float(i%17)-8};const V b{float(i%23),-3.5f,2.25f};const auto c=a.cross(b);std::cout<<"vec "<<i<<' '<<a.dot(b)<<' '<<c.x<<' '<<c.y<<' '<<c.z<<' '<<a.length()<<'\n';}
    for(int i=0;i<64;++i){M a;for(int row=0;row<4;++row)for(int col=0;col<4;++col)a.at(row,col)=float((i+row*4+col)%17)-8;const auto result=a*a;std::cout<<"mat "<<i;for(int row=0;row<4;++row)for(int col=0;col<4;++col)std::cout<<' '<<result.at(row,col);std::cout<<' '<<a.determinant()<<'\n';}
}
