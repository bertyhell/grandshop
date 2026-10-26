//! View-only transforms of the preview: rotation in quarter turns and mirroring.
//! They are never saved; the viewer applies them to the image it shows.
//!
//! A transform is "rotate clockwise `quarter_turns` times, then mirror horizontally on
//! screen if `mirrored`". Every combination of rotations and flips reduces to this form.

use image::RgbaImage;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewTransform {
    quarter_turns: u8,
    mirrored: bool,
}

/// Rectangle in normalised coordinates, 0 to 1 on both axes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Region {
    pub const FULL: Region = Region { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
}

impl ViewTransform {
    pub fn rotate_clockwise(self) -> Self {
        // Mirroring reverses the direction of a rotation applied before it
        let delta = if self.mirrored { 3 } else { 1 };
        Self { quarter_turns: (self.quarter_turns + delta) % 4, ..self }
    }

    pub fn rotate_counter_clockwise(self) -> Self {
        let delta = if self.mirrored { 1 } else { 3 };
        Self { quarter_turns: (self.quarter_turns + delta) % 4, ..self }
    }

    pub fn flip_horizontal(self) -> Self {
        Self { mirrored: !self.mirrored, ..self }
    }

    /// A vertical flip is a horizontal one plus half a turn
    pub fn flip_vertical(self) -> Self {
        Self { quarter_turns: (self.quarter_turns + 2) % 4, mirrored: !self.mirrored }
    }

    pub fn is_identity(self) -> bool {
        self == Self::default()
    }

    /// Whether width and height swap
    pub fn swaps_axes(self) -> bool {
        self.quarter_turns % 2 == 1
    }

    pub fn apply(self, image: &RgbaImage) -> RgbaImage {
        let rotated = match self.quarter_turns {
            1 => image::imageops::rotate90(image),
            2 => image::imageops::rotate180(image),
            3 => image::imageops::rotate270(image),
            _ => image.clone(),
        };
        if self.mirrored { image::imageops::flip_horizontal(&rotated) } else { rotated }
    }

    /// Maps a normalised point on screen back to the untransformed image.
    pub fn to_source(self, (mut u, mut v): (f32, f32)) -> (f32, f32) {
        if self.mirrored {
            u = 1.0 - u;
        }
        // Undo each clockwise quarter turn, which mapped (u, v) to (1 - v, u)
        for _ in 0..self.quarter_turns {
            (u, v) = (v, 1.0 - u);
        }
        (u, v)
    }

    /// Maps a region on screen back to the untransformed image.
    pub fn region_to_source(self, region: Region) -> Region {
        let (u0, v0) = self.to_source((region.left, region.top));
        let (u1, v1) = self.to_source((region.right, region.bottom));
        Region { left: u0.min(u1), top: v0.min(v1), right: u0.max(u1), bottom: v0.max(v1) }
    }

    /// Maps a region of the untransformed image to where it shows on screen.
    pub fn region_from_source(self, region: Region) -> Region {
        // The transforms are their own family: the inverse of `self` maps points forward
        let inverse = self.inverse();
        inverse.region_to_source(region)
    }

    fn inverse(self) -> Self {
        // Mirrored transforms are their own inverse; plain rotations reverse direction
        if self.mirrored { self } else { Self { quarter_turns: (4 - self.quarter_turns) % 4, mirrored: false } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// 3×2 image where every pixel is unique
    fn sample() -> RgbaImage {
        RgbaImage::from_fn(3, 2, |x, y| Rgba([x as u8, y as u8, 0, 255]))
    }

    /// Checks `to_source` against what `apply` really does, for every pixel centre.
    fn assert_consistent(transform: ViewTransform) {
        let source = sample();
        let shown = transform.apply(&source);
        for (x, y, pixel) in shown.enumerate_pixels() {
            let u = (x as f32 + 0.5) / shown.width() as f32;
            let v = (y as f32 + 0.5) / shown.height() as f32;
            let (su, sv) = transform.to_source((u, v));
            let sx = (su * source.width() as f32) as u32;
            let sy = (sv * source.height() as f32) as u32;
            assert_eq!(pixel, source.get_pixel(sx, sy), "{transform:?} at ({x}, {y})");
        }
    }

    fn all_transforms() -> Vec<ViewTransform> {
        let mut transforms = vec![ViewTransform::default()];
        for _ in 0..3 {
            let last = *transforms.last().unwrap();
            transforms.push(last.rotate_clockwise());
        }
        let mirrored: Vec<_> = transforms.iter().map(|t| t.flip_horizontal()).collect();
        transforms.extend(mirrored);
        transforms
    }

    #[test]
    fn point_mapping_matches_the_pixels() {
        for transform in all_transforms() {
            assert_consistent(transform);
        }
    }

    #[test]
    fn region_round_trip() {
        let region = Region { left: 0.1, top: 0.2, right: 0.4, bottom: 0.9 };
        for transform in all_transforms() {
            let back = transform.region_from_source(transform.region_to_source(region));
            for (a, b) in [(back.left, region.left), (back.top, region.top), (back.right, region.right), (back.bottom, region.bottom)] {
                assert!((a - b).abs() < 1e-6, "{transform:?}: {back:?}");
            }
        }
    }

    #[test]
    fn rotations_and_flips_compose_like_on_screen() {
        let image = sample();
        let identity = ViewTransform::default();
        let left = identity.rotate_counter_clockwise();
        assert_eq!(left.apply(&image), image::imageops::rotate270(&image));
        assert!(left.rotate_clockwise().is_identity());

        let flipped = identity.flip_vertical();
        assert_eq!(flipped.apply(&image), image::imageops::flip_vertical(&image));
        assert!(flipped.flip_vertical().is_identity());

        // Flip, then rotate: the rotation happens on the flipped picture
        let flip_then_rotate = identity.flip_horizontal().rotate_clockwise();
        let expected = image::imageops::rotate90(&image::imageops::flip_horizontal(&image));
        assert_eq!(flip_then_rotate.apply(&image), expected);
    }
}
