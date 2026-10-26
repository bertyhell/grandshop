//! Fixed shapes for the crop frame: choosing one picks the largest centred crop of that
//! shape, and dragging the frame keeps it.

/// Kept part of the image in pixels, from `x0`, `y0` up to `x1`, `y1`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl CropBox {
    fn width(&self) -> f32 {
        self.x1 - self.x0
    }

    fn height(&self) -> f32 {
        self.y1 - self.y0
    }
}

/// Width divided by height of each preset; 0 is free. The shape stands like the image does,
/// so a portrait photo gets 3:4 instead of 4:3.
pub fn ratio(preset: i32, width: u32, height: u32) -> Option<f32> {
    let landscape = match preset {
        1 => 1.0,
        2 => 4.0 / 3.0,
        3 => 3.0 / 2.0,
        4 => 16.0 / 9.0,
        _ => return None,
    };
    Some(if height > width { 1.0 / landscape } else { landscape })
}

/// Largest crop of shape `ratio` in the middle of a `width` × `height` image
pub fn largest_centred(width: f32, height: f32, ratio: f32) -> CropBox {
    let (w, h) = if width / height > ratio { (height * ratio, height) } else { (width, width / ratio) };
    let (x0, y0) = ((width - w) / 2.0, (height - h) / 2.0);
    CropBox { x0, y0, x1: x0 + w, y1: y0 + h }
}

/// Turns the frame the user dragged to (`after`) into one of shape `ratio` inside the image.
/// A corner drag keeps the opposite corner in place; an edge drag keeps the opposite edge and
/// grows or shrinks the other direction around its middle.
pub fn constrain(before: CropBox, after: CropBox, ratio: f32, width: f32, height: f32) -> CropBox {
    // Sides are whole pixels, so anything under half a pixel is rounding from the screen
    let moved = |a: f32, b: f32| (a - b).abs() > 0.5;
    let (left, right) = (moved(before.x0, after.x0), moved(before.x1, after.x1));
    let (top, bottom) = (moved(before.y0, after.y0), moved(before.y1, after.y1));

    if (left || right) && (top || bottom) {
        // Corner: the opposite corner stays, the frame covers the pointer
        let anchor_x = if left { after.x1 } else { after.x0 };
        let anchor_y = if top { after.y1 } else { after.y0 };
        let room_x = if left { anchor_x } else { width - anchor_x };
        let room_y = if top { anchor_y } else { height - anchor_y };
        let w = after.width().max(after.height() * ratio).min(room_x).min(room_y * ratio).max(1.0);
        let h = w / ratio;
        let (x0, x1) = if left { (anchor_x - w, anchor_x) } else { (anchor_x, anchor_x + w) };
        let (y0, y1) = if top { (anchor_y - h, anchor_y) } else { (anchor_y, anchor_y + h) };
        return CropBox { x0, y0, x1, y1 };
    }
    if left || right {
        let anchor = if left { after.x1 } else { after.x0 };
        let room = if left { anchor } else { width - anchor };
        let w = after.width().min(room).min(height * ratio).max(1.0);
        let h = w / ratio;
        let middle = ((before.y0 + before.y1) / 2.0).clamp(h / 2.0, height - h / 2.0);
        let (x0, x1) = if left { (anchor - w, anchor) } else { (anchor, anchor + w) };
        return CropBox { x0, y0: middle - h / 2.0, x1, y1: middle + h / 2.0 };
    }
    if top || bottom {
        let anchor = if top { after.y1 } else { after.y0 };
        let room = if top { anchor } else { height - anchor };
        let h = after.height().min(room).min(width / ratio).max(1.0);
        let w = h * ratio;
        let middle = ((before.x0 + before.x1) / 2.0).clamp(w / 2.0, width - w / 2.0);
        let (y0, y1) = if top { (anchor - h, anchor) } else { (anchor, anchor + h) };
        return CropBox { x0: middle - w / 2.0, y0, x1: middle + w / 2.0, y1 };
    }
    after
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(b: CropBox) -> f32 {
        b.width() / b.height()
    }

    fn inside(b: CropBox, width: f32, height: f32) -> bool {
        b.x0 >= -1e-3 && b.y0 >= -1e-3 && b.x1 <= width + 1e-3 && b.y1 <= height + 1e-3
    }

    #[test]
    fn presets_follow_the_orientation() {
        assert_eq!(ratio(0, 400, 300), None);
        assert_eq!(ratio(2, 400, 300), Some(4.0 / 3.0));
        assert_eq!(ratio(2, 300, 400), Some(3.0 / 4.0));
        assert_eq!(ratio(1, 300, 400), Some(1.0));
    }

    #[test]
    fn largest_centred_crop() {
        assert_eq!(largest_centred(400.0, 300.0, 1.0), CropBox { x0: 50.0, y0: 0.0, x1: 350.0, y1: 300.0 });
        assert_eq!(largest_centred(400.0, 300.0, 16.0 / 9.0), CropBox { x0: 0.0, y0: 37.5, x1: 400.0, y1: 262.5 });
    }

    #[test]
    fn corner_drag_keeps_the_opposite_corner_and_the_shape() {
        let before = CropBox { x0: 100.0, y0: 100.0, x1: 300.0, y1: 250.0 };
        let after = CropBox { x0: 100.0, y0: 100.0, x1: 380.0, y1: 260.0 };
        let out = constrain(before, after, 4.0 / 3.0, 400.0, 300.0);
        assert_eq!((out.x0, out.y0), (100.0, 100.0));
        assert!((shape(out) - 4.0 / 3.0).abs() < 1e-4, "{out:?}");
        assert!(inside(out, 400.0, 300.0), "{out:?}");
    }

    #[test]
    fn edge_drag_grows_around_the_middle_and_stays_inside() {
        let before = CropBox { x0: 100.0, y0: 100.0, x1: 200.0, y1: 200.0 };
        let after = CropBox { x0: 100.0, y0: 100.0, x1: 390.0, y1: 200.0 };
        let out = constrain(before, after, 1.0, 400.0, 300.0);
        assert_eq!(out.x0, 100.0);
        assert!((shape(out) - 1.0).abs() < 1e-4, "{out:?}");
        assert!(inside(out, 400.0, 300.0), "{out:?}");
        assert_eq!(out.height(), 290.0, "limited by the room on the right");
    }

    #[test]
    fn top_edge_drag() {
        let before = CropBox { x0: 0.0, y0: 100.0, x1: 160.0, y1: 190.0 };
        let after = CropBox { x0: 0.0, y0: 40.0, x1: 160.0, y1: 190.0 };
        let out = constrain(before, after, 16.0 / 9.0, 400.0, 300.0);
        assert_eq!((out.y0, out.y1), (40.0, 190.0));
        assert!((shape(out) - 16.0 / 9.0).abs() < 1e-4 && inside(out, 400.0, 300.0), "{out:?}");
    }
}
