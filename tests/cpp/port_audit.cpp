// Additional coverage outside the original parity fixture. No omath implementation is copied here.
#define main unused_baseline_entry
#include "reference.cpp"
#undef main
#include <omath/algorithm/targeting.hpp>

int main(int argc, char** argv) {
    using namespace omath::hud;
    using namespace omath::hud::widget;
    std::cout << std::setprecision(9);
    const std::string mode = argc > 1 ? argv[1] : "corner_glow";
    if (mode == "fov_wrap" || mode == "fov_roll") {
        using namespace omath::source_engine;
        Camera camera{V{}, ViewAngles{PitchAngle::from_degrees(0),
                                     YawAngle::from_degrees(mode == "fov_wrap" ? 179.f : 0.f),
                                     RollAngle::from_degrees(mode == "fov_roll" ? 45.f : 0.f)},
                      {800,600}, omath::projection::FieldOfView::from_degrees(90),0.1f,1000.f};
        const std::vector<V> targets = mode == "fov_wrap" ? std::vector<V>{{-100,-1.7455f,0},{-100,17.6327f,0}}
                                                         : std::vector<V>{{100,0,0},{100,10,0}};
        using Iterator = std::vector<V>::const_iterator;
        const auto selected = omath::algorithm::get_closest_target_by_fov<Camera,Iterator,bool(const V&)>(
            targets.cbegin(),targets.cend(),camera,[](const V& target)->const V& {return target;});
        std::cout << "target " << std::distance(targets.cbegin(),selected) << '\n';
        return 0;
    }
    if (mode == "gradient_channel") {
        // This is the channel conversion in ImguiHudRenderer's static circle/text gradient paths.
        std::cout << "shade " << static_cast<int>(0.5f * 255.f) << '\n';
        return 0;
    }
    auto renderer = std::make_shared<Recorder>();
    const omath::Color red{1, 0, 0, 1}, blue{0, 0, 1, 1}, clear{};
    // This constructor exists in both the pinned revision and the captured
    // working tree. It produces the same [75,10]-[125,110] rectangle.
    EntityOverlay entity{{100,10},{100,110},4.f,renderer};
    if (mode == "corner_glow") {
        entity.contents(CorneredBox{red, {0, 1, 0, 0.5f}, clear, 0.2f, 1.f,
                                   Glow{blue, 2.f, 1.f}});
    } else if (mode == "dashed_right") {
        entity.add_right_dashed_bar(red, blue, clear, 4, 0.25f, 8, 5);
    } else if (mode == "dashed_left") {
        entity.add_left_dashed_bar(red, blue, clear, 4, 0.25f, 8, 5);
    } else if (mode == "dashed_top") {
        entity.add_top_dashed_bar(red, blue, clear, 4, 0.25f, 8, 5);
    } else if (mode == "dashed_bottom") {
        entity.add_bottom_dashed_bar(red, blue, clear, 4, 0.25f, 8, 5);
    } else {
        std::cerr << "unknown audit fixture\n";
        return 2;
    }
    return 0;
}
