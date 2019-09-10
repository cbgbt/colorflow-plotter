use std::collections::HashMap;
use std::env;

mod colormapper;

use euclid::*;
use image::*;
use indicatif::*;
use noise::*;
use palette::*;
use rand::{thread_rng, Rng};

use palette::rgb::Srgb;
use palette::Lab;

use colormapper::*;

fn compute_pixel_color_and_brightness(
    image: &image::DynamicImage,
    pixel_start: (u32, u32),
    width: u32,
    height: u32,
) -> (InkJoyGelPenBlend, f64) {
    let (x, y) = pixel_start;
    let num_pixels = (width * height) as f32;

    let mut sl = 0.0;
    let mut sa = 0.0;
    let mut sb = 0.0;

    for i in 0..width {
        for j in 0..height {
            let pixel = image.get_pixel(x + i, y + j).to_rgb();
            let (r, g, b) = (pixel.0[0] as u32, pixel.0[1] as u32, pixel.0[2] as u32);
            let pixel_srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            let pixel_lab: Lab = pixel_srgb.into();
            let (l, a, b) = pixel_lab.into_components();
            sl += l;
            sa += a;
            sb += b;
        }
    }

    let (l, a, b) = (sl / num_pixels, sa / num_pixels, sb / num_pixels);

    let target_color = Lab::new(l, a, b);

    let repr_color = InkJoyGelPenBlend::closest_pen_to_color(target_color);
    let repr_lab = repr_color.lab_color();

    // We actually want to find out what % white we want
    // and then draw 1-(that percent)...
    // We'll take an iterative approach

    let pure_white: Lab = LinSrgb::new(1.0, 1.0, 1.0).into();
    let iter_value = 0.005;

    let mut closest_dist = std::f32::MAX;
    let mut white_percent = 0.0;

    while iter_value < 1.0 {
        let trial_white_percent = white_percent + iter_value;

        let trial_color = repr_lab.mix(&pure_white, white_percent);
        let (tl, ta, tb) = trial_color.into_components();
        let dist = ((tl - l).powi(2) + (ta - a).powi(2) + (tb - b).powi(2)).sqrt();

        if dist < closest_dist {
            closest_dist = dist;
            white_percent = trial_white_percent;
        } else {
            break;
        }
    }

    //let brightness = 1.0 - white_percent;

    (repr_color, white_percent as f64)
}

pub struct PixelSpace;
pub type PVec = Vector2D<f64, PixelSpace>;
pub type PPoint = Point2D<f64, PixelSpace>;
pub type PRotation = Rotation2D<f64, PixelSpace, PixelSpace>;
pub type PBox = Box2D<f64, PixelSpace>;

fn box_intersection(b: PBox, o: PPoint, v: PVec) -> Option<(PPoint, PPoint)> {
    let mut txmin = (b.min.x - o.x) / v.x;
    let mut txmax = (b.max.x - o.x) / v.x;

    if txmin > txmax {
        std::mem::swap(&mut txmin, &mut txmax);
    }

    let mut tymin = (b.min.y - o.y) / v.y;
    let mut tymax = (b.max.y - o.y) / v.y;

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if (txmin > tymax) || (tymin > txmax) {
        return None;
    }

    let tmin = if tymin > txmin { tymin } else { txmin };

    let tmax = if tymax < txmax { tymax } else { txmax };

    let p1 = o + (v * tmin);
    let p2 = o + (v * tmax);

    Some((p1, p2))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} input_file output_file", args[0]);
        std::process::exit(1);
    }

    let input_file = args[1].clone();
    let output_file = args[2].clone();

    dotenv::dotenv().ok();

    let pixels_x: usize = env::var("PIXELS_X").unwrap().parse().unwrap();
    let pixels_y: usize = env::var("PIXELS_Y").unwrap().parse().unwrap();

    let svg_width: String = env::var("SVG_WIDTH").unwrap();
    let svg_height: String = env::var("SVG_HEIGHT").unwrap();

    let input_scale_to_x: usize = env::var("INPUT_SCALE_TO_X").unwrap().parse().unwrap();
    let input_scale_to_y: usize = env::var("INPUT_SCALE_TO_Y").unwrap().parse().unwrap();

    let pixel_width: f64 = input_scale_to_x as f64 / pixels_x as f64;
    let pixel_height: f64 = input_scale_to_y as f64 / pixels_y as f64;

    let line_min_spacing: f64 = env::var("LINE_MIN_SPACING").unwrap().parse().unwrap();
    let line_max_spacing: f64 = env::var("LINE_MAX_SPACING").unwrap().parse().unwrap();

    let brighten: i32 = env::var("BRIGHTEN").unwrap().parse().unwrap();

    let mut rng = thread_rng();
    let random_seed: u32 = env::var("RANDOM_SEED")
        .ok()
        .map_or_else(|| rng.gen(), |val| val.parse().unwrap());

    // Input image should be square...
    // (this criteria is for our pixel drawer idea)
    let input_image = image::open(&input_file)
        .unwrap()
        .brighten(brighten)
        .thumbnail_exact(input_scale_to_x as u32, input_scale_to_y as u32);

    let mut noise_fn = Billow::new().set_seed(random_seed);
    let noise_fn = Turbulence::new(&noise_fn);

    let noise_vector_field = |x: f64, y: f64| {
        let input_noise = noise_fn.get([x, y]);
        let angle = input_noise * std::f64::consts::PI;
        PVec::new(angle.cos(), angle.sin())
    };

    let xy_vector_field = |x: f64, y: f64| PVec::new(1.0, 1.0);

    let vector_field_1 = |x: f64, y: f64| {
        let x = x * 10.0 - 5.0;
        let y = -y * 10.0 - 5.0;

        PVec::new(
            (x * x + y * y).sqrt(),
            y * (f64::min(y, x) - f64::max(x.cos(), (x * x + y * y).sqrt())).sin(),
        )
        .normalize()
    };

    let vector_field = |x: f64, y: f64| {
        let x = x * 10.0 - 5.0;
        let y = y * 10.0 - 5.0;
        PVec::new(y, (x.sin() - y).cos()).normalize()
    };

    let mut document = svg::Document::new()
        .set("viewBox", (0, 0, input_scale_to_x, input_scale_to_y))
        .set("width", svg_width)
        .set("height", svg_height)
        .set("stroke-width", "0.7mm")
        .set("xmlns:inkscape", "http://www.inkscape.org/namespaces/inkscape");

    //document = draw_pixel_boundaries(document);
    document = document.add(
        svg::node::element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "white"),
    );

    println!("Computing pixels...");
    let bar = ProgressBar::new((pixels_x * pixels_y) as u64);
    let mut lines: HashMap<InkJoyGelPen, Vec<svg::node::element::Line>> = HashMap::new();
    for x in 0..pixels_x {
        for y in 0..pixels_y {
            let pixel_x = (x as f64 * pixel_width) as u32;
            let pixel_y = (y as f64 * pixel_height) as u32;

            let (pen, brightness) = compute_pixel_color_and_brightness(
                &input_image,
                (pixel_x, pixel_y),
                pixel_width as u32,
                pixel_height as u32,
            );

            let n_x = (x as f64) / pixels_x as f64;
            let n_y = (y as f64) / pixels_y as f64;
            let vector = noise_vector_field(n_x, n_y);

            let line_spacing =
                brightness * (line_max_spacing - line_min_spacing) + line_min_spacing;

            let bbox = PBox::new(
                PPoint::new(0.0, 0.0),
                PPoint::new(pixel_width, pixel_height),
            );

            let origin = PPoint::new(
                rng.gen_range(0.0, 1.0) * pixel_width / 2.0,
                rng.gen_range(0.0, 1.0) * pixel_height / 2.0,
            );
            let rotation1 = PRotation::radians(std::f64::consts::PI / 2.0);
            let rotation2 = PRotation::radians(-std::f64::consts::PI / 2.0);

            let mut count = 0;

            let mut pen_a = true;

            // Draw all vectors to the left that intersect
            let line_bound = PBox::new(
                PPoint::new(-pixel_width * 5.0, -pixel_height * 5.0),
                PPoint::new(pixel_width * 5.0, pixel_height * 5.0),
            );
            let mut o = origin;

            while line_bound.contains(o) {
                o = origin + (rotation1.transform_vector(vector) * line_spacing * count as f64);
                let actual_pen = if pen_a { &pen.pen_a } else { &pen.pen_b };
                match box_intersection(bbox, o, vector) {
                    Some((p1, p2)) => {
                        pen_a = !pen_a;
                        let line = svg::node::element::Line::new()
                            .set("x1", p1.x + pixel_x as f64)
                            .set("y1", p1.y + pixel_y as f64)
                            .set("x2", p2.x + pixel_x as f64)
                            .set("y2", p2.y + pixel_y as f64);
                        lines.entry(*actual_pen).or_insert(Vec::new()).push(line);
                    }
                    None => {}
                }

                count += 1;
            }

            pen_a = false;
            let mut count = 1;
            // Draw all vectors to the right that intersect
            o = origin;
            while line_bound.contains(o) {
                o = origin + (rotation2.transform_vector(vector) * line_spacing * count as f64);
                let actual_pen = if pen_a { &pen.pen_a } else { &pen.pen_b };
                match box_intersection(bbox, o, vector) {
                    Some((p1, p2)) => {
                        pen_a = !pen_a;
                        let line = svg::node::element::Line::new()
                            .set("x1", p1.x + pixel_x as f64)
                            .set("y1", p1.y + pixel_y as f64)
                            .set("x2", p2.x + pixel_x as f64)
                            .set("y2", p2.y + pixel_y as f64);
                        lines.entry(*actual_pen).or_insert(Vec::new()).push(line);
                    }
                    None => {}
                }

                count += 1;
            }

            bar.inc(1);
        }
    }
    bar.finish();

    // Add all of our lines to layers
    for (pen, lines) in lines.drain() {
        if pen == InkJoyGelPen::WhiteCanvas {
            continue;
        }
        let (r, g, b) = pen.rgb_pixel();
        let mut group =
            svg::node::element::Group::new()
                .set("stroke", format!("rgb({},{},{})", r, g, b))
                .set("inkscape:groupmode", "layer")
                .set("inkscape:label", format!("{:?}", pen));

        for line in lines {
            group = group.add(line);
        }

        document = document.add(group);
    }

    svg::save(&output_file, &document).unwrap();
    println!("Saved SVG to {}.", &output_file);
    println!("Random seed: {}", random_seed);
}
