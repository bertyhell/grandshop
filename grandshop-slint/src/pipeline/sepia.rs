use image::RgbaImage;
use rayon::prelude::*;

/// Same recombination matrix as the Electron version.
const SEPIA: [[f32; 3]; 3] = [
    [0.393, 0.769, 0.189],
    [0.349, 0.686, 0.168],
    [0.272, 0.534, 0.131],
];

pub fn sepia(image: &RgbaImage) -> RgbaImage {
    let mut out = image.clone();
    out.par_chunks_exact_mut(4).for_each(|pixel| {
        let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(f32::from);
        for (channel, row) in SEPIA.iter().enumerate() {
            pixel[channel] = (row[0] * r + row[1] * g + row[2] * b).round().clamp(0.0, 255.0) as u8;
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn applies_the_sepia_matrix() {
        let image = RgbaImage::from_pixel(1, 1, Rgba([100, 50, 20, 200]));
        // r = 39.3 + 38.45 + 3.78, g = 34.9 + 34.3 + 3.36, b = 27.2 + 26.7 + 2.62
        assert_eq!(sepia(&image).get_pixel(0, 0), &Rgba([82, 73, 57, 200]));
    }

    #[test]
    fn clamps_bright_pixels() {
        let image = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        assert_eq!(sepia(&image).get_pixel(0, 0), &Rgba([255, 255, 239, 255]));
    }
}
