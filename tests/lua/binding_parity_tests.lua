-- What the C++ test scripts leave out: error values, overload rules, sol2's automatic metamethods, the other engines.

local function raises(expected, f, ...)
    local ok, err = pcall(f, ...)
    assert(not ok, "expected an error")
    assert(type(err) == "string", "expected a string error, got " .. type(err))
    assert(err == expected, "got: " .. err)
end

function Mat_errors_are_plain_strings()
    local m = omath.Mat4.new()
    raises("matrix index is out of range", m.at, m, 4, 0)
    raises("matrix index is out of range", m.at, m, 0, -1)
    raises("matrix index is out of range", m.set_at, m, 0, 9, 1)
    raises("matrix rows must be tables", omath.Mat3.from_rows, {1, 2, 3})
    raises("matrix row has missing value", omath.Mat3.from_rows, {{1, 2}, {1, 2, 3}, {1, 2, 3}})
    raises("matrix row has missing value", omath.Mat3.from_rows, {{1, 2, "3"}, {1, 2, 3}, {1, 2, 3}})
end

function Mat_static_and_instance_access()
    local m = omath.Mat4.new()
    assert(omath.Mat4.row_count() == 4 and m:row_count() == 4 and m.row_count() == 4)
    assert(omath.Mat3.columns_count() == 3)
    assert(omath.Mat4.at(m, 0, 0) == 0)
    assert(math.type(m:row_count()) == "integer")
end

function Mat_set_at_returns_a_copy()
    local m = omath.Mat4.new()
    local copy = m:set_at(0, 0, 1)
    copy:set_at(0, 0, 2)
    assert(m:at(0, 0) == 1 and copy:at(0, 0) == 2)
end

function Mat_index_rounds_like_sol2()
    local m = omath.Mat4.from_rows({{1, 2, 3, 4}, {5, 6, 7, 8}, {9, 10, 11, 12}, {13, 14, 15, 16}})
    assert(m:at(0.0, 1.5) == 3)
end

function Mat_helpers_only_on_mat4_kinds()
    assert(omath.Mat4d.translation == nil and omath.Mat3.scale == nil)
    assert(omath.Mat4.look_at_left_handed ~= nil and omath.Mat4ColumnMajor.perspective_right_handed_vertical_fov ~= nil)
end

function Mat_to_string_format()
    local s = tostring(omath.Mat3.from_rows({{1, 0, 0}, {0, -1.5, 0}, {0, 0, 1}}))
    assert(s == "[[    1.000,     0.000,     0.000]\n [    0.000,    -1.500,     0.000]\n [    0.000,     0.000,     1.000]]", s)
end

function Vector_arity_and_overloads()
    assert(not pcall(omath.Vec3.new, 1, 2))
    assert(not pcall(omath.Vec3.new, 1, 2, 3, 4))
    assert(not pcall(omath.Vec2.new, 1))
    assert(not pcall(function() return omath.Vec2.new(1, 2) * omath.Vec2.new(1, 2) end))
    assert(not pcall(function() return omath.Vec2.new(1, 2) / omath.Vec2.new(1, 2) end))
    assert(not pcall(function() return 2 / omath.Vec3.new(1, 2, 3) end))
    assert(not pcall(function() return omath.Vec3.new(1, 2, 3) * "2" end))
    assert(omath.Vec3.length(omath.Vec3.new(3, 4, 0)) == 5)
end

function Vector_surface_is_exact()
    assert(omath.Vec2.cross == nil and omath.Vec2.as_table == nil)
    assert(omath.Vec4.normalized == nil and omath.Vec4.distance_to == nil)
    assert(omath.Vec3.new(1, 2, 3).nope == nil)
end

function Vector_mixed_types_are_not_equal()
    assert(omath.Vec2.new(1, 2) ~= omath.Vec3.new(1, 2, 0))
    assert(omath.Vec3.new(1, 2, 3) ~= 1)
end

function Vec3_narrows_like_a_cast()
    assert(omath.Vec3.new(1e300, 0, 0).x == math.huge)
end

function Vec3_as_array_is_one_based()
    local a = omath.Vec3.new(1, 2, 3):as_array()
    assert(#a == 3 and a[1] == 1 and a[3] == 3)
end

function Vec3_angle_between_returns_two_values()
    local ok = table.pack(omath.Vec3.new(1, 0, 0):angle_between(omath.Vec3.new(0, 1, 0)))
    assert(ok.n == 2 and ok[2] == nil)
    local failed = table.pack(omath.Vec3.new(0, 0, 0):angle_between(omath.Vec3.new(1, 0, 0)))
    assert(failed.n == 2 and failed[1] == nil and failed[2] == "impossible angle (zero-length vector)")
end

function Vec4_clamp_mutates_and_returns_a_copy()
    local v = omath.Vec4.new(5, -3, 10, 99)
    local copy = v:clamp(0, 7)
    assert(v.w == 7 and copy.w == 7)
    copy.x = 1
    assert(v.x == 5)
end

function Float_strings_match_std_format()
    assert(tostring(omath.Vec2.new(1.5, -2)) == "Vec2(1.5, -2)")
    assert(tostring(omath.Vec2.new(1e20, 1e-4)) == "Vec2(1e+20, 1e-04)")
    assert(tostring(omath.Vec2.new(0.001, 10000)) == "Vec2(0.001, 10000)")
    assert(tostring(omath.Vec2.new(100000, 123456789)) == "Vec2(1e+05, 123456792)")
    assert(tostring(omath.Vec2.new(math.huge, -math.huge)) == "Vec2(inf, -inf)")
    assert(tostring(omath.Quaternion.new(1.5e-7, 0, 0, 1)) == "Quaternion(1.5e-07, 0, 0, 1)")
    assert(tostring(omath.source.PitchAngle.from_degrees(0.5)) == "0.5deg")
end

function Color_extras()
    assert(omath.Color.red() == omath.Color.new(1, 0, 0, 1))
    assert(omath.Color.from_rgba(300, 0, 0, 255).r == omath.Color.from_rgba(44, 0, 0, 255).r)
    assert(omath.Color.from_rgba(127.5, 0, 0, 255).r == omath.Color.from_rgba(128, 0, 0, 255).r)
    assert(not pcall(omath.Color.new, 1, 2, 3))
    assert(not pcall(function() local c = omath.Color.red(); c.r = 0 end))
end

function Identity_equality_where_cpp_has_no_operator()
    local a = omath.primitives.Aabb.new()
    assert(a == a and a ~= omath.primitives.Aabb.new())
    local h = omath.Hsv.new()
    assert(h == h and h ~= omath.Hsv.new())
end

function Angle_ordering_comes_from_the_spaceship_operator()
    local one, two = omath.source.PitchAngle.from_degrees(1), omath.source.PitchAngle.from_degrees(2)
    assert(one < two and one <= one and two > one and not (two < one))
end

function Angle_cot()
    assert(math.abs(omath.source.PitchAngle.from_degrees(45):cot() - 1) < 1e-4)
end

function UpAxis_is_a_read_only_enum()
    local axis = omath.primitives.UpAxis
    assert(axis.X == 0 and axis.Y == 1 and axis.Z == 2)
    raises("sol: cannot modify the elements of an enumeration table", function() axis.X = 5 end)
    local keys = 0
    for _ in pairs(axis) do keys = keys + 1 end
    assert(keys == 3)
end

function Aabb_values_are_copies()
    local aabb = omath.primitives.Aabb.new(omath.Vec3.new(1, 1, 1), omath.Vec3.new(2, 2, 2))
    aabb.min.x = 9
    assert(aabb.min.x == 1)
end

function Aabb_axis_defaults_when_not_a_number()
    local aabb = omath.primitives.Aabb.new(omath.Vec3.new(-1, -2, -3), omath.Vec3.new(3, 6, 9))
    assert(aabb:top("x").y == 6)
end

function Ray_third_argument_must_be_boolean()
    local a, b = omath.Vec3.new(0, 0, 0), omath.Vec3.new(1, 0, 0)
    assert(not pcall(omath.collision.Ray.new, a, b, 1))
    assert(not omath.collision.Ray.new(a, b).infinite_length)
    assert(not omath.collision.Ray.new().infinite_length)
    assert(not pcall(omath.collision.Ray.new, a))
end

function ConvexCollider_errors_and_ties()
    raises("convex collider must contain at least one vertex", omath.collision.ConvexCollider.new, {})
    raises("convex collider must contain at least one vertex", omath.collision.ConvexCollider.new, {1, 2})

    -- The points stop at the first entry that is not a Vec3
    local stopped = omath.collision.ConvexCollider.new({omath.Vec3.new(1, 0, 0), "x", omath.Vec3.new(2, 0, 0)})
    assert(stopped:vertex_count() == 1)

    -- max_element keeps the first of equal elements
    local tied = omath.collision.ConvexCollider.new({omath.Vec3.new(1, 5, 0), omath.Vec3.new(1, 7, 0)})
    assert(tied:find_abs_furthest_vertex_position(omath.Vec3.new(1, 0, 0)).y == 5)

    local moved = omath.collision.ConvexCollider.new({omath.Vec3.new(1, 0, 0)})
    moved:set_origin(omath.Vec3.new(0, 1, 0))
    assert(moved:get_origin().y == 1 and moved:vertices()[1].y == 0)
end

function LineTracer_rejects_other_targets()
    local ray = omath.collision.Ray.new(omath.Vec3.new(0, 0, 0), omath.Vec3.new(1, 0, 0))
    assert(not pcall(omath.collision.LineTracer.get_ray_hit_point, ray, omath.Vec3.new()))
    assert(omath.collision.LineTracer.new == nil)
end

function Projection_error_texts()
    local vp = omath.source.ViewPort.new(1920, 1080)
    local fov = omath.source.FieldOfView.from_degrees(90)
    local angles = omath.source.ViewAngles.new(
        omath.source.PitchAngle.from_degrees(0),
        omath.source.YawAngle.from_degrees(0),
        omath.source.RollAngle.from_degrees(0))
    local cam = omath.source.Camera.new(omath.Vec3.new(), angles, vp, fov, 0.1, 1000)

    local screen, err = cam:world_to_screen(omath.Vec3.new(-100, 0, 0))
    assert(screen == nil and err == "perspective divider is less or equal to zero", err)

    screen, err = cam:world_to_screen(omath.Vec3.new(1, 100, 0))
    assert(screen == nil and err == "world position is out of screen bounds", err)

    cam:set_near_plane(0)
    local world
    world, err = cam:screen_to_world(omath.Vec3.new(960, 540, 1))
    assert(world == nil and err == "inverse view-projection matrix determinant is zero", err)
end

function Scanners_have_no_module_surface()
    assert(omath.PePatternScanner.scan_in_module == nil)
    assert(omath.ElfPatternScanner.scan_in_module == nil)
    assert(omath.MachOPatternScanner.scan_in_module == nil)
    assert(omath.hud == nil)
    assert(omath.SectionScanResult.new == nil)
    assert(omath.collision.LineTracer.nope == nil)
end

function Scanner_scan_in_file(path)
    local result = omath.PePatternScanner.scan_in_file(path, "90 ?? 02")
    assert(result ~= nil)
    assert(result.target_offset == 0 and result.raw_base_addr == 0x200)
    assert(tostring(result) == "SectionScanResult(vbase=0x1200, raw_base=0x200, offset=0)", tostring(result))
    assert(omath.PePatternScanner.scan_in_file(path, "90 01", ".rdata") == nil)
    assert(omath.PePatternScanner.scan_in_file(path, "AA BB") == nil)
    assert(omath.PePatternScanner.scan_in_file(path .. ".missing", "90 01") == nil)
end

function Aliases_share_the_type_objects()
    assert(omath.source.PitchAngle == omath._types.PitchAngle89)
    assert(omath.iw.PitchAngle == omath._types.PitchAngle89)
    assert(omath.opengl.PitchAngle == omath._types.PitchAngle90)
    assert(omath.source.ViewAngles == omath._types.ViewAngles89)
    assert(omath.cry.ViewAngles == omath._types.ViewAngles90)
    assert(omath.source.YawAngle == omath.source.RollAngle and omath.source.YawAngle == omath._types.YawRoll)
    assert(omath.unity.ViewPort == omath._types.ViewPort and omath.unreal.FieldOfView == omath._types.FieldOfView)
end

function Every_engine_projects(name, mat_type)
    local engine = omath[name]
    local angles = engine.ViewAngles.new(
        engine.PitchAngle.from_degrees(0),
        engine.YawAngle.from_degrees(0),
        engine.RollAngle.from_degrees(0))
    local cam = engine.Camera.new(
        omath.Vec3.new(1, 2, 3), angles, engine.ViewPort.new(1920, 1080), engine.FieldOfView.from_degrees(90), 0.1, 1000)

    cam:look_at(omath.Vec3.new(50, 60, 70))
    local origin = engine.Camera.calc_origin_from_view_matrix(cam:get_view_matrix())
    assert(math.abs(origin.x - 1) < 1e-2 and math.abs(origin.y - 2) < 1e-2 and math.abs(origin.z - 3) < 1e-2)

    local screen, err = cam:world_to_screen(omath.Vec3.new(50, 60, 70))
    assert(screen ~= nil, err)
    assert(math.abs(screen.x - 960) < 1 and math.abs(screen.y - 540) < 1)

    local view = cam:get_view_projection_matrix()
    assert(view:row_count() == 4)
    assert(omath[mat_type].new():row_count() == 4)
    assert(engine.Camera.extract_projection_params(cam:get_projection_matrix()) ~= nil)
    assert(engine.Camera.calc_view_angles_from_view_matrix(cam:get_view_matrix()) ~= nil)
end

function Unreal_pitch_clamps_to_the_engines_own_range()
    assert(omath.unreal.PitchAngle.from_degrees(100):as_degrees() == 90)
    assert(omath.unreal.PitchAngle ~= omath._types.PitchAngle89)
end
