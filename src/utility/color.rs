use crate::linear_algebra::Vector4;
use crate::scalar::Scalar;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsv {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

/// An RGBA color with every channel kept in `[0, 1]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    value: Vector4<f32>,
}

impl Color {
    #[cfg(feature = "imgui")]
    pub fn to_im_vec4(&self) -> imgui::sys::ImVec4 {
        let v = self.value;
        imgui::sys::ImVec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
    #[cfg(feature = "imgui")]
    pub fn to_im_color(&self) -> imgui::ImColor32 {
        let v = self.value;
        imgui::ImColor32::from_rgba_f32s(v.x, v.y, v.z, v.w)
    }
    pub const RED: Self = Self {
        value: Vector4::new(1.0, 0.0, 0.0, 1.0),
    };
    pub const GREEN: Self = Self {
        value: Vector4::new(0.0, 1.0, 0.0, 1.0),
    };
    pub const BLUE: Self = Self {
        value: Vector4::new(0.0, 0.0, 1.0, 1.0),
    };

    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::from_vector(Vector4::new(r, g, b, a))
    }

    #[inline]
    pub fn from_vector(value: Vector4<f32>) -> Self {
        Self {
            value: value.clamped(0.0, 1.0),
        }
    }

    #[inline]
    pub fn value(&self) -> Vector4<f32> {
        self.value
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_vector(Vector4::new(r as f32, g as f32, b as f32, a as f32) / 255.0)
    }

    pub fn from_hsv(mut hue: f32, saturation: f32, value: f32) -> Self {
        // hue is cyclic: wrap into [0, 1)
        hue -= hue.trunc();
        if hue < 0.0 {
            hue += 1.0;
        }

        let saturation = saturation.clamped(0.0, 1.0);
        let value = value.clamped(0.0, 1.0);

        let i = (hue * 6.0) as i32;
        let f = hue * 6.0 - i as f32;
        let p = value * (1.0 - saturation);
        let q = value * (1.0 - f * saturation);
        let t = value * (1.0 - (1.0 - f) * saturation);

        let (r, g, b) = match i % 6 {
            0 => (value, t, p),
            1 => (q, value, p),
            2 => (p, value, t),
            3 => (p, q, value),
            4 => (t, p, value),
            5 => (value, p, q),
            _ => unreachable!("hue is in [0, 1)"),
        };

        Self::new(r, g, b, 1.0)
    }

    pub fn to_hsv(&self) -> Hsv {
        let Vector4 {
            x: red,
            y: green,
            z: blue,
            ..
        } = self.value;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let delta = max - min;

        // hue as a fraction of the six 60-degree sectors, i.e. already in [0, 1)
        let mut hue = if delta == 0.0 {
            0.0
        } else if max == red {
            ((green - blue) / delta) / 6.0
        } else if max == green {
            (((blue - red) / delta) + 2.0) / 6.0
        } else {
            (((red - green) / delta) + 4.0) / 6.0
        };

        if hue < 0.0 {
            hue += 1.0;
        }

        Hsv {
            hue,
            saturation: if max == 0.0 { 0.0 } else { delta / max },
            value: max,
        }
    }

    pub fn set_hue(&mut self, hue: f32) {
        self.apply_hsv(Hsv {
            hue,
            ..self.to_hsv()
        });
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        self.apply_hsv(Hsv {
            saturation,
            ..self.to_hsv()
        });
    }

    pub fn set_value(&mut self, value: f32) {
        self.apply_hsv(Hsv {
            value,
            ..self.to_hsv()
        });
    }

    /// Linear interpolation. `ratio` is clamped to `[0, 1]`.
    #[must_use]
    pub fn blend(&self, other: &Color, ratio: f32) -> Self {
        let ratio = ratio.clamped(0.0, 1.0);
        Self::from_vector(self.value * (1.0 - ratio) + other.value * ratio)
    }

    /// `[r:.., g:.., b:.., a:..]` with floating point channels. [`Display`](fmt::Display) gives bytes.
    pub fn to_rgbf_string(&self) -> String {
        let Vector4 { x, y, z, w } = self.value;
        format!("[r:{x}, g:{y}, b:{z}, a:{w}]")
    }

    pub fn to_hsv_string(&self) -> String {
        let Hsv {
            hue,
            saturation,
            value,
        } = self.to_hsv();
        format!("[h:{hue}, s:{saturation}, v:{value}]")
    }

    /// Rebuilds RGB from HSV while keeping the current alpha.
    fn apply_hsv(&mut self, hsv: Hsv) {
        let alpha = self.value.w;
        *self = Self::from_hsv(hsv.hue, hsv.saturation, hsv.value);
        self.value.w = alpha;
    }
}

impl From<Hsv> for Color {
    #[inline]
    fn from(hsv: Hsv) -> Self {
        Self::from_hsv(hsv.hue, hsv.saturation, hsv.value)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let to_byte = |channel: f32| (channel * 255.0 + 0.5) as i32;
        let Vector4 { x, y, z, w } = self.value;
        write!(
            f,
            "[r:{}, g:{}, b:{}, a:{}]",
            to_byte(x),
            to_byte(y),
            to_byte(z),
            to_byte(w)
        )
    }
}
