use palette::rgb::LinSrgb;
use palette::{Hsl, Lab, Mix};

pub trait ColoredPen: Sized + Clone {
    fn available_colors() -> Vec<Self>;

    fn rgb_color(&self) -> LinSrgb;

    fn rgb_pixel(&self) -> (u8, u8, u8) {
        let (r, g, b) = self.rgb_color().into_components();
        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    fn hsl_color(&self) -> Hsl {
        self.rgb_color().into()
    }

    fn lab_color(&self) -> Lab {
        self.rgb_color().into()
    }

    fn closest_pen_to_color(icolor: Lab) -> Self {
        let available_pens = Self::available_colors();

        let (l, a, b) = icolor.into_components();

        let mut closest = available_pens[0].clone();
        let mut mindist = std::f32::MAX;

        for pen in available_pens {
            let (pl, pa, pb) = pen.lab_color().into_components();

            let dist = ((pl - l).powi(2) + (pa - a).powi(2) + (pb - b).powi(2)).sqrt();

            if dist < mindist {
                mindist = dist;
                closest = pen.clone();
            }
        }
        closest
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InkJoyGelPen {
    Red,
    Berry,
    Pink,
    Orange,
    Yellow,
    Green,
    Lime,
    SlateBlue,
    Blue,
    BrightBlue,
    Teal,
    Purple,
    Cocoa,
    Black,
    WhiteCanvas,
}

impl ColoredPen for InkJoyGelPen {
    fn available_colors() -> Vec<Self> {
        vec![
            InkJoyGelPen::Red,
            InkJoyGelPen::Berry,
            InkJoyGelPen::Pink,
            InkJoyGelPen::Orange,
            InkJoyGelPen::Yellow,
            InkJoyGelPen::Green,
            InkJoyGelPen::Lime,
            InkJoyGelPen::SlateBlue,
            InkJoyGelPen::Blue,
            InkJoyGelPen::BrightBlue,
            InkJoyGelPen::Teal,
            InkJoyGelPen::Purple,
            InkJoyGelPen::Cocoa,
            InkJoyGelPen::Black,
            InkJoyGelPen::WhiteCanvas,
        ]
    }

    fn rgb_color(&self) -> LinSrgb {
        let (r, g, b) = match self {
            InkJoyGelPen::Red => (0xd1, 0x24, 0x31),
            InkJoyGelPen::Berry => (0xc1, 0x52, 0x9e),
            InkJoyGelPen::Pink => (0xd8, 0x40, 0x8c),
            InkJoyGelPen::Orange => (0xf3, 0x6c, 0x38),
            InkJoyGelPen::Yellow => (0xff, 0xda, 0x3a),
            InkJoyGelPen::Green => (0x00, 0xa8, 0x5d),
            InkJoyGelPen::Lime => (0xa6, 0xd0, 0x60),
            InkJoyGelPen::SlateBlue => (0x28, 0x62, 0x8f),
            InkJoyGelPen::Blue => (0x32, 0x55, 0xa4),
            InkJoyGelPen::BrightBlue => (0x47, 0xb7, 0xe6),
            InkJoyGelPen::Teal => (0x00, 0x9b, 0xa8),
            InkJoyGelPen::Purple => (0x78, 0x5b, 0xa7),
            InkJoyGelPen::Cocoa => (0x8e, 0x61, 0x5e),
            InkJoyGelPen::Black => (0x37, 0x36, 0x3d),
            InkJoyGelPen::WhiteCanvas => (0xfc, 0xfc, 0xfc),
        };
        LinSrgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0).into_linear()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InkJoyGelPenBlend {
    pub pen_a: InkJoyGelPen,
    pub pen_b: InkJoyGelPen,
}

impl ColoredPen for InkJoyGelPenBlend {
    fn available_colors() -> Vec<Self> {
        let mut results = vec![];
        for pen_a in InkJoyGelPen::available_colors() {
            for pen_b in InkJoyGelPen::available_colors() {
                results.push(InkJoyGelPenBlend { pen_a, pen_b })
            }
        }

        results
    }

    fn rgb_color(&self) -> LinSrgb {
        self.pen_a.rgb_color().mix(&self.pen_b.rgb_color(), 0.5)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InkJoyBlackGelPen {}

impl ColoredPen for InkJoyBlackGelPen {
    fn available_colors() -> Vec<Self> {
        vec![InkJoyBlackGelPen {}]
    }

    fn rgb_color(&self) -> LinSrgb {
        InkJoyGelPen::Black.rgb_color()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InkJoyBlackGelPenBlend {
    pub pen_a: InkJoyGelPen,
    pub pen_b: InkJoyGelPen,
}

impl ColoredPen for InkJoyBlackGelPenBlend {
    fn available_colors() -> Vec<Self> {
        vec![InkJoyBlackGelPenBlend {
            pen_a: InkJoyGelPen::Black,
            pen_b: InkJoyGelPen::Black,
        }]
    }

    fn rgb_color(&self) -> LinSrgb {
        InkJoyGelPen::Black.rgb_color()
    }
}
