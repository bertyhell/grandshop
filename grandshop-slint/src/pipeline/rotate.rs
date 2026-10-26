use image::{Rgba, RgbaImage};
use rayon::prelude::*;

/// Size of the canvas that fits the image rotated by `angle` degrees.
pub fn rotated_size(width: u32, height: u32, angle: f32) -> (u32, u32) {
    let (sin, cos) = (angle as f64).to_radians().sin_cos();
    let (w, h) = (width as f64, height as f64);
    let new_width = (w * cos.abs() + h * sin.abs()).round().max(1.0) as u32;
    let new_height = (w * sin.abs() + h * cos.abs()).round().max(1.0) as u32;
    (new_width, new_height)
}

/// Rotates clockwise by `angle` degrees. The canvas grows to fit and the corners are filled
/// with `background`, like sharp's `rotate(angle, { background })`.
pub fn rotate(image: &RgbaImage, angle: f32, background: Rgba<u8>) -> RgbaImage {
    let turns = angle.rem_euclid(360.0);
    // Quarter turns are exact and fast
    for (quarter, rotate_quarter) in [
        (0.0, RgbaImage::clone as fn(&RgbaImage) -> RgbaImage),
        (90.0, |i: &RgbaImage| image::imageops::rotate90(i)),
        (180.0, |i: &RgbaImage| image::imageops::rotate180(i)),
        (270.0, |i: &RgbaImage| image::imageops::rotate270(i)),
        (360.0, RgbaImage::clone),
    ] {
        if (turns - quarter).abs() < 1e-4 {
            return rotate_quarter(image);
        }
    }

    let (width, height) = image.dimensions();
    let (new_width, new_height) = rotated_size(width, height, angle);
    let (sin, cos) = (angle as f64).to_radians().sin_cos();
    let (center_x, center_y) = (width as f64 / 2.0, height as f64 / 2.0);
    let (new_center_x, new_center_y) = (new_width as f64 / 2.0, new_height as f64 / 2.0);

    let sample = |x: i64, y: i64| -> [f64; 4] {
        if x < 0 || y < 0 || x >= width as i64 || y >= height as i64 {
            background.0.map(f64::from)
        } else {
            image.get_pixel(x as u32, y as u32).0.map(f64::from)
        }
    };

    let mut out = RgbaImage::new(new_width, new_height);
    out.par_chunks_exact_mut(new_width as usize * 4).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            // Undo the clockwise rotation (y points down) to find the source position
            let dx = x as f64 + 0.5 - new_center_x;
            let dy = y as f64 + 0.5 - new_center_y;
            let source_x = dx * cos + dy * sin + center_x - 0.5;
            let source_y = -dx * sin + dy * cos + center_y - 0.5;

            // Bilinear interpolation, blending into the background at the edges
            let (x0, y0) = (source_x.floor(), source_y.floor());
            let (fx, fy) = (source_x - x0, source_y - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let [a, b, c, d] = [sample(x0, y0), sample(x0 + 1, y0), sample(x0, y0 + 1), sample(x0 + 1, y0 + 1)];
            for channel in 0..4 {
                let top = a[channel] * (1.0 - fx) + b[channel] * fx;
                let bottom = c[channel] * (1.0 - fx) + d[channel] * fx;
                pixel[channel] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

    fn sample() -> RgbaImage {
        RgbaImage::from_fn(40, 20, |x, y| Rgba([x as u8 * 6, y as u8 * 12, 100, 255]))
    }

    #[test]
    fn quarter_turns_are_exact() {
        let image = sample();
        assert_eq!(rotate(&image, 0.0, WHITE), image);
        assert_eq!(rotate(&image, 360.0, WHITE), image);
        assert_eq!(rotate(&image, 90.0, WHITE), image::imageops::rotate90(&image));
        assert_eq!(rotate(&image, -90.0, WHITE), image::imageops::rotate270(&image));
        assert_eq!(rotate(&image, 180.0, WHITE), image::imageops::rotate180(&image));
    }

    #[test]
    fn canvas_grows_and_corners_get_the_background() {
        let out = rotate(&sample(), 45.0, WHITE);
        assert_eq!(out.dimensions(), rotated_size(40, 20, 45.0));
        assert_eq!(out.dimensions(), (42, 42));
        assert_eq!(out.get_pixel(0, 0), &WHITE);
        assert_eq!(out.get_pixel(41, 41), &WHITE);
        // The middle of the image stays in the middle
        let centre = out.get_pixel(21, 21);
        assert!(centre[0].abs_diff(120) <= 12 && centre[1].abs_diff(120) <= 18, "{centre:?}");
    }

    #[test]
    fn small_angles_turn_clockwise() {
        // A red pixel right of the centre moves down when turning clockwise
        let mut image = RgbaImage::from_pixel(41, 41, WHITE);
        image.put_pixel(35, 20, Rgba([255, 0, 0, 255]));
        let out = rotate(&image, 30.0, WHITE);
        let (x, y, _) = out
            .enumerate_pixels()
            .min_by_key(|(_, _, p)| p[1] as u32 + p[2] as u32)
            .unwrap();
        let centre = out.width() as f32 / 2.0;
        assert!(y as f32 > centre + 1.0 && x as f32 > centre, "red ended up at ({x}, {y})");
    }
}
