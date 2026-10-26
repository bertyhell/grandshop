//! Decorative "9-slice" borders. Each border style is a folder in `assets/borders` with four
//! corner pieces and four edge pieces. Corners are scaled to the requested border width;
//! edges are repeated a whole number of times and then stretched to close the gap exactly.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result};
use image::RgbaImage;
use include_dir::{include_dir, Dir};

use crate::resample::resize_exact;

static BORDERS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/borders");

/// Names of the bundled border styles, sorted.
pub fn available_borders() -> Vec<String> {
    let mut names: Vec<String> = BORDERS
        .dirs()
        .filter_map(|dir| dir.path().file_name()?.to_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

struct BorderSet {
    corner_top_left: RgbaImage,
    corner_top_right: RgbaImage,
    corner_bottom_left: RgbaImage,
    corner_bottom_right: RgbaImage,
    edge_top: RgbaImage,
    edge_bottom: RgbaImage,
    edge_left: RgbaImage,
    edge_right: RgbaImage,
}

fn load_border(name: &str) -> Result<Arc<BorderSet>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<BorderSet>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap();
    if let Some(set) = cache.get(name) {
        return Ok(set.clone());
    }

    let dir = BORDERS.get_dir(name).with_context(|| format!("unknown border style {name}"))?;
    let piece = |file: &str| -> Result<RgbaImage> {
        let path = format!("{name}/{file}.png");
        let data = dir
            .get_file(&path)
            .with_context(|| format!("border {name} is missing {file}.png"))?;
        Ok(image::load_from_memory(data.contents())?.into_rgba8())
    };
    let set = Arc::new(BorderSet {
        corner_top_left: piece("corner-top-left")?,
        corner_top_right: piece("corner-top-right")?,
        corner_bottom_left: piece("corner-bottom-left")?,
        corner_bottom_right: piece("corner-bottom-right")?,
        edge_top: piece("edge-top")?,
        edge_bottom: piece("edge-bottom")?,
        edge_left: piece("edge-left")?,
        edge_right: piece("edge-right")?,
    });
    cache.insert(name.to_owned(), set.clone());
    Ok(set)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    CornerTopLeft,
    CornerTopRight,
    CornerBottomLeft,
    CornerBottomRight,
    EdgeTop,
    EdgeBottom,
    EdgeLeft,
    EdgeRight,
}

/// Where each piece goes on a `width` × `height` image. Every piece is scaled by `scale`.
/// Edges fill the exact gap between their corners, so there are no 1 px seams.
/// Edges are left out when the image is too small for the corners.
pub fn layout(
    width: u32,
    height: u32,
    scale: f32,
    piece_size: impl Fn(Piece) -> (u32, u32),
) -> Vec<(Piece, Rect)> {
    let scaled = |piece: Piece| {
        let (w, h) = piece_size(piece);
        (((w as f32 * scale).round() as u32).max(1), ((h as f32 * scale).round() as u32).max(1))
    };
    let (w, h) = (width as i64, height as i64);
    let (tl, tr) = (scaled(Piece::CornerTopLeft), scaled(Piece::CornerTopRight));
    let (bl, br) = (scaled(Piece::CornerBottomLeft), scaled(Piece::CornerBottomRight));

    let mut placements = Vec::with_capacity(8);
    let mut edge = |piece: Piece, x: i64, y: i64, w: i64, h: i64| {
        if w > 0 && h > 0 {
            placements.push((piece, Rect { x, y, width: w as u32, height: h as u32 }));
        }
    };
    let top_h = scaled(Piece::EdgeTop).1 as i64;
    let bottom_h = scaled(Piece::EdgeBottom).1 as i64;
    let left_w = scaled(Piece::EdgeLeft).0 as i64;
    let right_w = scaled(Piece::EdgeRight).0 as i64;
    edge(Piece::EdgeTop, tl.0 as i64, 0, w - tl.0 as i64 - tr.0 as i64, top_h);
    edge(Piece::EdgeBottom, bl.0 as i64, h - bottom_h, w - bl.0 as i64 - br.0 as i64, bottom_h);
    edge(Piece::EdgeLeft, 0, tl.1 as i64, left_w, h - tl.1 as i64 - bl.1 as i64);
    edge(Piece::EdgeRight, w - right_w, tr.1 as i64, right_w, h - tr.1 as i64 - br.1 as i64);

    let corner = |piece: Piece, (cw, ch): (u32, u32), x: i64, y: i64| {
        (piece, Rect { x, y, width: cw, height: ch })
    };
    placements.push(corner(Piece::CornerTopLeft, tl, 0, 0));
    placements.push(corner(Piece::CornerTopRight, tr, w - tr.0 as i64, 0));
    placements.push(corner(Piece::CornerBottomLeft, bl, 0, h - bl.1 as i64));
    placements.push(corner(Piece::CornerBottomRight, br, w - br.0 as i64, h - br.1 as i64));
    placements
}

/// Repeats `tile` to cover roughly `length` (in source pixels) along one axis.
fn repeat(tile: &RgbaImage, count: u32, horizontal: bool) -> RgbaImage {
    let (tw, th) = tile.dimensions();
    let mut strip = if horizontal { RgbaImage::new(tw * count, th) } else { RgbaImage::new(tw, th * count) };
    for i in 0..count {
        let (x, y) = if horizontal { (i * tw, 0) } else { (0, i * th) };
        image::imageops::replace(&mut strip, tile, x as i64, y as i64);
    }
    strip
}

/// Draws border `name` over the edges of the image. `border_width` is the width of the
/// top-left corner in output pixels; everything else scales with it.
pub fn add_image_border(image: &RgbaImage, name: &str, border_width: f32) -> Result<RgbaImage> {
    let set = load_border(name)?;
    let piece = |p: Piece| match p {
        Piece::CornerTopLeft => &set.corner_top_left,
        Piece::CornerTopRight => &set.corner_top_right,
        Piece::CornerBottomLeft => &set.corner_bottom_left,
        Piece::CornerBottomRight => &set.corner_bottom_right,
        Piece::EdgeTop => &set.edge_top,
        Piece::EdgeBottom => &set.edge_bottom,
        Piece::EdgeLeft => &set.edge_left,
        Piece::EdgeRight => &set.edge_right,
    };
    let scale = border_width / set.corner_top_left.width() as f32;

    let mut out = image.clone();
    for (p, rect) in layout(image.width(), image.height(), scale, |p| piece(p).dimensions()) {
        let source = piece(p);
        let rendered = match p {
            Piece::EdgeTop | Piece::EdgeBottom => {
                let tile_width = source.width() as f32 * scale;
                let count = ((rect.width as f32 / tile_width).round() as u32).max(1);
                resize_exact(&repeat(source, count, true), rect.width, rect.height)
            }
            Piece::EdgeLeft | Piece::EdgeRight => {
                let tile_height = source.height() as f32 * scale;
                let count = ((rect.height as f32 / tile_height).round() as u32).max(1);
                resize_exact(&repeat(source, count, false), rect.width, rect.height)
            }
            _ => resize_exact(source, rect.width, rect.height),
        };
        image::imageops::overlay(&mut out, &rendered, rect.x, rect.y);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn lists_bundled_borders() {
        assert_eq!(
            available_borders(),
            ["art_deco", "christmas_holly", "floral_curly", "gold_frame", "vintage_scrollwork"].map(str::to_owned)
        );
    }

    #[test]
    fn unknown_border_is_an_error() {
        assert!(add_image_border(&RgbaImage::new(100, 100), "nope", 10.0).is_err());
    }

    #[test]
    fn edges_close_the_gaps_between_corners_exactly() {
        // Uneven piece sizes like the real floral_curly set
        let sizes = |p: Piece| match p {
            Piece::CornerTopLeft | Piece::CornerBottomLeft => (187, 187),
            Piece::CornerTopRight | Piece::CornerBottomRight => (186, 187),
            Piece::EdgeTop => (314, 187),
            Piece::EdgeBottom => (315, 187),
            Piece::EdgeLeft => (187, 315),
            Piece::EdgeRight => (186, 315),
        };
        for (width, height, scale) in [(1920, 1080, 0.711), (1001, 733, 0.37), (640, 480, 1.0)] {
            let rects: HashMap<_, _> = layout(width, height, scale, sizes).into_iter().collect();
            let r = |p| rects[&p];
            let (tl, tr, bl, br) = (r(Piece::CornerTopLeft), r(Piece::CornerTopRight), r(Piece::CornerBottomLeft), r(Piece::CornerBottomRight));
            let (top, bottom, left, right) = (r(Piece::EdgeTop), r(Piece::EdgeBottom), r(Piece::EdgeLeft), r(Piece::EdgeRight));

            assert_eq!(tl.x + tl.width as i64, top.x);
            assert_eq!(top.x + top.width as i64, tr.x);
            assert_eq!(tr.x + tr.width as i64, width as i64);
            assert_eq!(bl.x + bl.width as i64, bottom.x);
            assert_eq!(bottom.x + bottom.width as i64, br.x);
            assert_eq!(bottom.y + bottom.height as i64, height as i64);
            assert_eq!(tl.y + tl.height as i64, left.y);
            assert_eq!(left.y + left.height as i64, bl.y);
            assert_eq!(tr.y + tr.height as i64, right.y);
            assert_eq!(right.y + right.height as i64, br.y);
            assert_eq!(right.x + right.width as i64, width as i64);
        }
    }

    #[test]
    fn small_images_skip_edges() {
        let rects = layout(100, 100, 1.0, |_| (80, 80));
        assert!(rects.iter().all(|(p, _)| !matches!(p, Piece::EdgeTop | Piece::EdgeBottom | Piece::EdgeLeft | Piece::EdgeRight)));
    }

    #[test]
    fn draws_over_the_edges_and_keeps_the_size() {
        let image = RgbaImage::from_pixel(800, 600, Rgba([255, 0, 255, 255]));
        let out = add_image_border(&image, "floral_curly", 100.0).unwrap();
        assert_eq!(out.dimensions(), (800, 600));
        assert_eq!(out.get_pixel(400, 300), &Rgba([255, 0, 255, 255]), "centre untouched");
        let changed_near_edge = (0..800).filter(|&x| out.get_pixel(x, 20) != image.get_pixel(x, 20)).count();
        assert!(changed_near_edge > 100, "border pattern along the top edge");
    }
}
