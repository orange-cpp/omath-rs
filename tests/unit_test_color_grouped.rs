mod common;

use common::*;
use omath::{Color, Hsv, Vector4};

mod unit_test_color_grouped {
    use super::*;

    #[test]
    fn constructor_float() {
        let color = Color::new(0.5f32, 0.5f32, 0.5f32, 1.0f32);
        expect_float_eq(color.value().x, 0.5f32);
        expect_float_eq(color.value().y, 0.5f32);
        expect_float_eq(color.value().z, 0.5f32);
        expect_float_eq(color.value().w, 1.0f32);
    }

    #[test]
    fn constructor_vector4() {
        let vec = Vector4::new(0.2f32, 0.4f32, 0.6f32, 0.8f32);
        let color = Color::from_vector(vec);
        expect_float_eq(color.value().x, 0.2f32);
        expect_float_eq(color.value().y, 0.4f32);
        expect_float_eq(color.value().z, 0.6f32);
        expect_float_eq(color.value().w, 0.8f32);
    }

    #[test]
    fn from_rgba() {
        let color = Color::from_rgba(128, 64, 32, 255);
        expect_float_eq(color.value().x, 128.0f32 / 255.0f32);
        expect_float_eq(color.value().y, 64.0f32 / 255.0f32);
        expect_float_eq(color.value().z, 32.0f32 / 255.0f32);
        expect_float_eq(color.value().w, 1.0f32);
    }

    #[test]
    fn from_hsv() {
        let color = Color::from_hsv(0.0f32, 1.0f32, 1.0f32);
        expect_float_eq(color.value().x, 1.0f32);
        expect_float_eq(color.value().y, 0.0f32);
        expect_float_eq(color.value().z, 0.0f32);
        expect_float_eq(color.value().w, 1.0f32);
    }

    #[test]
    fn to_hsv() {
        let color1 = Color::RED;
        let Hsv {
            hue,
            saturation,
            value,
        } = color1.to_hsv(); // Red color
        expect_float_eq(hue, 0.0f32);
        expect_float_eq(saturation, 1.0f32);
        expect_float_eq(value, 1.0f32);
    }

    #[test]
    fn blend() {
        let (color1, color2) = (Color::RED, Color::GREEN);
        let blended = color1.blend(&color2, 0.5f32);
        expect_float_eq(blended.value().x, 0.5f32);
        expect_float_eq(blended.value().y, 0.5f32);
        expect_float_eq(blended.value().z, 0.0f32);
        expect_float_eq(blended.value().w, 1.0f32);
    }

    #[test]
    fn predefined_colors() {
        let red = Color::RED;
        let green = Color::GREEN;
        let blue = Color::BLUE;

        expect_float_eq(red.value().x, 1.0f32);
        expect_float_eq(red.value().y, 0.0f32);
        expect_float_eq(red.value().z, 0.0f32);
        expect_float_eq(red.value().w, 1.0f32);

        expect_float_eq(green.value().x, 0.0f32);
        expect_float_eq(green.value().y, 1.0f32);
        expect_float_eq(green.value().z, 0.0f32);
        expect_float_eq(green.value().w, 1.0f32);

        expect_float_eq(blue.value().x, 0.0f32);
        expect_float_eq(blue.value().y, 0.0f32);
        expect_float_eq(blue.value().z, 1.0f32);
        expect_float_eq(blue.value().w, 1.0f32);
    }

    #[test]
    fn blend_vector3() {
        let v1 = Color::new(1.0f32, 0.0f32, 0.0f32, 1.0f32); // Red
        let v2 = Color::new(0.0f32, 1.0f32, 0.0f32, 1.0f32); // Green
        let blended = v1.blend(&v2, 0.5f32);
        expect_float_eq(blended.value().x, 0.5f32);
        expect_float_eq(blended.value().y, 0.5f32);
        expect_float_eq(blended.value().z, 0.0f32);
    }
}

mod unit_test_color_grouped_extra {
    use super::*;

    #[test]
    fn set_hue_saturation_value() {
        let mut c = Color::RED;
        let h1 = c.to_hsv();
        expect_float_eq(h1.hue, 0.0f32);

        c.set_hue(0.5f32);
        let h2 = c.to_hsv();
        expect_near(h2.hue, 0.5f32, 1e-3f32);

        c = Color::from_hsv(0.25f32, 0.8f32, 0.6f32);
        c.set_saturation(0.3f32);
        let h3 = c.to_hsv();
        expect_near(h3.saturation, 0.3f32, 1e-3f32);

        c.set_value(1.0f32);
        let h4 = c.to_hsv();
        expect_near(h4.value, 1.0f32, 1e-3f32);
    }

    #[test]
    fn to_string_variants() {
        // to_wstring and to_u8string have no Rust counterpart: a Rust String already is UTF-8.
        let c = Color::from_rgba(10, 20, 30, 255);
        let s = c.to_string();
        assert!(s.contains("r:"));
    }

    #[test]
    fn blend_edge_cases() {
        let a = Color::RED;
        let b = Color::BLUE;
        let r0 = a.blend(&b, 0.0f32);
        expect_float_eq(r0.value().x, a.value().x);
        let r1 = a.blend(&b, 1.0f32);
        expect_float_eq(r1.value().x, b.value().x);
    }
}

mod unit_test_color_grouped_more {
    use super::*;

    #[test]
    fn default_ctor_is_zero() {
        let c = Color::default();
        expect_float_eq(c.value().x, 0.0f32);
        expect_float_eq(c.value().y, 0.0f32);
        expect_float_eq(c.value().z, 0.0f32);
        expect_float_eq(c.value().w, 0.0f32);
    }

    #[test]
    fn float_ctor_and_clamp_for_rgb() {
        let c = Color::new(1.2f32, -0.5f32, 0.5f32, 2.0f32);
        expect_float_eq(c.value().x, 1.0f32);
        expect_float_eq(c.value().y, 0.0f32);
        expect_float_eq(c.value().z, 0.5f32);
        expect_float_eq(c.value().w, 1.0f32);
    }

    #[test]
    fn from_rgba_produces_scaled_components() {
        let c = Color::from_rgba(25, 128, 230, 64);
        expect_near(c.value().x, 25.0f32 / 255.0f32, 1e-6f32);
        expect_near(c.value().y, 128.0f32 / 255.0f32, 1e-6f32);
        expect_near(c.value().z, 230.0f32 / 255.0f32, 1e-6f32);
        expect_near(c.value().w, 64.0f32 / 255.0f32, 1e-6f32);
    }

    #[test]
    fn blend_produces_intermediate() {
        let c0 = Color::new(0.0f32, 0.0f32, 0.0f32, 1.0f32);
        let c1 = Color::new(1.0f32, 1.0f32, 1.0f32, 0.0f32);
        let mid = c0.blend(&c1, 0.5f32);
        expect_float_eq(mid.value().x, 0.5f32);
        expect_float_eq(mid.value().y, 0.5f32);
        expect_float_eq(mid.value().z, 0.5f32);
        expect_float_eq(mid.value().w, 0.5f32);
    }

    #[test]
    fn hsv_round_trip() {
        let red = Color::RED;
        let hsv = red.to_hsv();
        let back = Color::from(hsv);
        expect_near(back.value().x, 1.0f32, 1e-6f32);
        expect_near(back.value().y, 0.0f32, 1e-6f32);
        expect_near(back.value().z, 0.0f32, 1e-6f32);
    }

    #[test]
    fn to_string_contains_components() {
        let c = Color::from_rgba(10, 20, 30, 40);
        let s = c.to_string();
        assert!(s.contains("r:"));
        assert!(s.contains("g:"));
        assert!(s.contains("b:"));
        assert!(s.contains("a:"));
    }
}

mod unit_test_color_grouped_more2 {
    use super::*;

    #[test]
    fn from_rgba_and_to_string() {
        let c = Color::from_rgba(255, 128, 0, 64);
        let s = c.to_string();
        assert!(s.contains("r:255"));
        assert!(s.contains("g:128"));
        assert!(s.contains("b:0"));
        assert!(s.contains("a:64"));
    }

    #[test]
    fn from_hsv_cases() {
        let eps = 1e-5f32;

        let check_hue = |h: f32| {
            let c = Color::from_hsv(h, 1.0f32, 1.0f32);
            assert!(c.value().x.is_finite());
            assert!(c.value().y.is_finite());
            assert!(c.value().z.is_finite());
            assert!(c.value().x >= -eps);
            assert!(c.value().x <= 1.0f32 + eps);
            assert!(c.value().y >= -eps);
            assert!(c.value().y <= 1.0f32 + eps);
            assert!(c.value().z >= -eps);
            assert!(c.value().z <= 1.0f32 + eps);

            let mx = c.value().x.max(c.value().y).max(c.value().z);
            let mn = c.value().x.min(c.value().y).min(c.value().z);
            assert!(mx >= 0.999f32);
            assert!(mn <= 1e-3f32 + 1e-4f32);
        };

        check_hue(0.0f32 / 6.0f32);
        check_hue(1.0f32 / 6.0f32);
        check_hue(2.0f32 / 6.0f32);
        check_hue(3.0f32 / 6.0f32);
        check_hue(4.0f32 / 6.0f32);
        check_hue(5.0f32 / 6.0f32);
    }

    #[test]
    fn to_hsv_and_setters() {
        let mut c = Color::new(0.2f32, 0.4f32, 0.6f32, 1.0f32);
        let hsv = c.to_hsv();
        expect_near(hsv.value, 0.6f32, 1e-6f32);

        c.set_hue(0.0f32);
        assert!((c.value().x).is_finite());

        c.set_saturation(0.0f32);
        assert!((c.value().y).is_finite());

        c.set_value(0.5f32);
        assert!((c.value().z).is_finite());
    }

    #[test]
    fn blend_and_static_colors() {
        let a = Color::RED;
        let b = Color::BLUE;
        let mid = a.blend(&b, 0.5f32);
        assert!(mid.value().x > 0.0f32);
        assert!(mid.value().z > 0.0f32);

        let all_a = a.blend(&b, -1.0f32);
        expect_near(all_a.value().x, a.value().x, 1e-6f32);

        let all_b = a.blend(&b, 2.0f32);
        expect_near(all_b.value().z, b.value().z, 1e-6f32);
    }

    #[test]
    fn formatter_uses_to_string() {
        let c = Color::from_rgba(10, 20, 30, 40);
        let formatted = format!("{}", c);
        assert!(formatted.contains("r:10"));
    }

    #[test]
    fn formatter_rgb() {
        let c = Color::from_rgba(255, 128, 0, 64);
        let s = format!("{}", c);
        assert!(s.contains("r:255"));
        assert!(s.contains("g:128"));
        assert!(s.contains("b:0"));
        assert!(s.contains("a:64"));
    }

    #[test]
    fn formatter_rgbf() {
        // std::format's "{:rgbf}" spec became to_rgbf_string.
        let c = Color::new(0.5f32, 0.25f32, 1.0f32, 0.75f32);
        let s = c.to_rgbf_string();
        assert!(s.contains("r:"));
        assert!(s.contains("g:"));
        assert!(s.contains("b:"));
        assert!(s.contains("a:"));
        // Values should be in [0,1] float range, not 0-255
        assert!(!s.contains("r:127"));
        assert!(!s.contains("r:255"));
    }

    #[test]
    fn formatter_hsv() {
        // std::format's "{:hsv}" spec became to_hsv_string.
        let c = Color::RED;
        let s = c.to_hsv_string();
        assert!(s.contains("h:"));
        assert!(s.contains("s:"));
        assert!(s.contains("v:"));
    }
}

mod unit_test_color_grouped_fixes {
    use super::*;

    #[test]
    fn hsv_setters_preserve_alpha() {
        let mut c = Color::from_rgba(255, 0, 0, 128);
        let alpha: f32 = c.value().w;

        c.set_hue(0.5f32);
        expect_float_eq(c.value().w, alpha);

        c.set_saturation(0.3f32);
        expect_float_eq(c.value().w, alpha);

        c.set_value(0.7f32);
        expect_float_eq(c.value().w, alpha);
    }

    #[test]
    fn from_hsv_wraps_hue() {
        let wrapped_up = Color::from_hsv(1.25f32, 1.0f32, 1.0f32);
        let wrapped_down = Color::from_hsv(-0.75f32, 1.0f32, 1.0f32);
        let expected = Color::from_hsv(0.25f32, 1.0f32, 1.0f32);

        expect_near(wrapped_up.value().x, expected.value().x, 1e-5f32);
        expect_near(wrapped_up.value().y, expected.value().y, 1e-5f32);
        expect_near(wrapped_up.value().z, expected.value().z, 1e-5f32);
        expect_near(wrapped_down.value().x, expected.value().x, 1e-5f32);
        expect_near(wrapped_down.value().y, expected.value().y, 1e-5f32);
        expect_near(wrapped_down.value().z, expected.value().z, 1e-5f32);
    }

    #[test]
    fn hsv_round_trip() {
        for i in 0..36 {
            let hue = i as f32 / 36.0f32;
            let c = Color::from_hsv(hue, 0.75f32, 0.5f32);
            let hsv = c.to_hsv();
            expect_near(hsv.hue, hue, 1e-4f32);
            expect_near(hsv.saturation, 0.75f32, 1e-4f32);
            expect_near(hsv.value, 0.5f32, 1e-4f32);
        }
    }

    #[test]
    fn equality() {
        let a = Color::new(0.1f32, 0.2f32, 0.3f32, 0.4f32);
        let b = Color::new(0.1f32, 0.2f32, 0.3f32, 0.4f32);
        let c = Color::new(0.1f32, 0.2f32, 0.3f32, 0.5f32);

        assert!(a == b);
        assert!(!(a == c));
        assert!(a != c);
    }

    #[test]
    fn to_string_rounds_channels() {
        let blended = Color::RED.blend(&Color::GREEN, 0.5f32);
        assert_eq!(blended.to_string(), "[r:128, g:128, b:0, a:255]");
        assert_eq!(format!("{}", blended), "[r:128, g:128, b:0, a:255]");
    }
}
