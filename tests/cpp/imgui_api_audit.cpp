#define OMATH_IMGUI_INTEGRATION
#include <omath/linear_algebra/vector2.hpp>
#include <omath/linear_algebra/vector4.hpp>

void verify_upstream_conversion_api() {
    const auto a = omath::Vector2<float>::from_im_vec2(ImVec2{1, 2});
    const auto b = omath::Vector4<float>::from_im_vec4(ImVec4{1, 2, 3, 4});
    static_cast<void>(a);
    static_cast<void>(b);
}
