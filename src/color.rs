#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

fn map_err(r: Result<u8, std::num::ParseIntError>) -> Result<u8, String> {
    r.map_err(|e| format!("Error parsing hex: {}", e))
}

impl Color {
    pub fn gray(lightness: f32) -> Color {
        Self {
            r: lightness,
            g: lightness,
            b: lightness,
            a: 1.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0 - s) * self.r + s * rhs.r,
            g: (1.0 - s) * self.g + s * rhs.g,
            b: (1.0 - s) * self.b + s * rhs.b,
            a: (1.0 - s) * self.a + s * rhs.a,
        }
    }

    pub fn alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn hex(hex: &str) -> Result<Color, String> {
        if hex.len() == 9 && hex.starts_with("#") { // #FFFFFFFF (Red Green Blue Alpha)
            Ok(Color {
                r: map_err(u8::from_str_radix(&hex[1..3], 16))? as f32 / 255.0,
                g: map_err(u8::from_str_radix(&hex[3..5], 16))? as f32 / 255.0,
                b: map_err(u8::from_str_radix(&hex[5..7], 16))? as f32 / 255.0,
                a: map_err(u8::from_str_radix(&hex[7..9], 16))? as f32 / 255.0,
            })
        } else if hex.len() == 7 && hex.starts_with("#") { // #FFFFFF (Red Green Blue)
            Ok(Color {
                r: map_err(u8::from_str_radix(&hex[1..3], 16))? as f32 / 255.0,
                g: map_err(u8::from_str_radix(&hex[3..5], 16))? as f32 / 255.0,
                b: map_err(u8::from_str_radix(&hex[5..7], 16))? as f32 / 255.0,
                a: 1.0,
            })
        } else {
            Err("Error parsing hex. Example of valid formats: #FFFFFF or #ffffffff".to_string())
        }
    }
}
