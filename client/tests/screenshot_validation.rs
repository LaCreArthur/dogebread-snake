//! Structural validation of AUTO_TEST screenshots.
//!
//! Two-step workflow:
//!   1. `AUTO_TEST=1 cargo run -p client --release`  (capture — needs GPU)
//!   2. `cargo test -p client --test screenshot_validation`  (validate — pure image analysis)
//!
//! These are NOT pixel-perfect golden baselines. They check structural properties:
//! file exists, non-blank, expected color regions present.

use image::{GenericImageView, Rgba};
use std::path::Path;

const TEST_OUTPUT_DIR: &str = "test-output";

const EXPECTED_SCREENSHOTS: &[&str] = &[
    "01-countdown-3.png",
    "02-countdown-go.png",
    "03-gameplay-start.png",
    "04-first-death.png",
    "05-arena-shrink.png",
    "06-late-game.png",
    "07-gameover-title.png",
    "08-gameover-rankings.png",
    "09-gameover-complete.png",
];

// ---------------------------------------------------------------------------
// Reusable validation utilities
// ---------------------------------------------------------------------------

fn screenshot_path(name: &str) -> std::path::PathBuf {
    Path::new(TEST_OUTPUT_DIR).join(name)
}

fn load_screenshot(name: &str) -> image::DynamicImage {
    let path = screenshot_path(name);
    image::open(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to open screenshot '{}' at {}: {}. Run AUTO_TEST=1 cargo run -p client --release first.",
            name,
            path.display(),
            e
        )
    })
}

/// Assert screenshot exists and meets minimum dimensions.
fn assert_screenshot(name: &str, min_w: u32, min_h: u32) {
    let path = screenshot_path(name);
    assert!(path.exists(), "Screenshot '{}' not found at {}", name, path.display());
    let img = load_screenshot(name);
    let (w, h) = img.dimensions();
    assert!(
        w >= min_w && h >= min_h,
        "Screenshot '{}' too small: {}x{} (minimum {}x{})",
        name,
        w,
        h,
        min_w,
        min_h
    );
}

/// Assert an image is not a solid single color (blank screen).
fn assert_not_blank(img: &image::DynamicImage, name: &str) {
    let (w, h) = img.dimensions();
    let sample_pixel = img.get_pixel(0, 0);
    let mut all_same = true;

    // Sample every 10th pixel for speed
    'outer: for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            if img.get_pixel(x, y) != sample_pixel {
                all_same = false;
                break 'outer;
            }
        }
    }

    assert!(!all_same, "Screenshot '{}' appears to be a solid blank color", name);
}

/// Rect region within an image (fractions of dimensions).
struct Region {
    x_start: f32,
    y_start: f32,
    x_end: f32,
    y_end: f32,
}

/// Returns the fraction of pixels in a region that satisfy a color predicate.
fn region_color_fraction(img: &image::DynamicImage, region: &Region, check: impl Fn(Rgba<u8>) -> bool) -> f32 {
    let (w, h) = img.dimensions();
    let x0 = (region.x_start * w as f32) as u32;
    let y0 = (region.y_start * h as f32) as u32;
    let x1 = (region.x_end * w as f32) as u32;
    let y1 = (region.y_end * h as f32) as u32;

    let mut matching = 0u32;
    let mut total = 0u32;

    // Sample every 3rd pixel for speed
    for y in (y0..y1).step_by(3) {
        for x in (x0..x1).step_by(3) {
            total += 1;
            if check(img.get_pixel(x, y)) {
                matching += 1;
            }
        }
    }

    if total == 0 {
        return 0.0;
    }
    matching as f32 / total as f32
}

// ---------------------------------------------------------------------------
// Color predicates
// ---------------------------------------------------------------------------

/// Doge gold: warm yellow-orange tones (R>180, G>140, B<120)
fn is_gold(p: Rgba<u8>) -> bool {
    p[0] > 180 && p[1] > 140 && p[3] > 200 && p[2] < 120
}

/// Danger zone: orange-red tones (R>150, G<120, B<80)
fn is_danger_red(p: Rgba<u8>) -> bool {
    p[0] > 150 && p[1] < 120 && p[2] < 80 && p[3] > 200
}

/// Bright / non-dark pixel (not background)
fn is_bright(p: Rgba<u8>) -> bool {
    let luminance = p[0] as u32 + p[1] as u32 + p[2] as u32;
    luminance > 200 && p[3] > 200
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_all_screenshots_exist() {
    let dir = Path::new(TEST_OUTPUT_DIR);
    if !dir.exists() {
        eprintln!(
            "Skipping screenshot validation: {} not found. Run AUTO_TEST=1 cargo run -p client --release first.",
            TEST_OUTPUT_DIR
        );
        return;
    }

    for name in EXPECTED_SCREENSHOTS {
        assert_screenshot(name, 100, 100);
    }
}

#[test]
fn test_countdown_has_gold_text() {
    let path = screenshot_path("01-countdown-3.png");
    if !path.exists() {
        eprintln!("Skipping: 01-countdown-3.png not found");
        return;
    }

    let img = load_screenshot("01-countdown-3.png");
    assert_not_blank(&img, "01-countdown-3.png");

    // Gold text should appear in the center region
    let center = Region {
        x_start: 0.3,
        y_start: 0.3,
        x_end: 0.7,
        y_end: 0.7,
    };
    let gold_fraction = region_color_fraction(&img, &center, is_gold);
    assert!(
        gold_fraction > 0.005,
        "01-countdown-3.png: expected gold text in center region, found {:.1}% gold pixels",
        gold_fraction * 100.0
    );
}

#[test]
fn test_gameplay_start_has_arena_border() {
    let path = screenshot_path("03-gameplay-start.png");
    if !path.exists() {
        eprintln!("Skipping: 03-gameplay-start.png not found");
        return;
    }

    let img = load_screenshot("03-gameplay-start.png");
    assert_not_blank(&img, "03-gameplay-start.png");

    // Arena wall border should be visible around edges
    // Check top edge strip
    let top_edge = Region {
        x_start: 0.1,
        y_start: 0.0,
        x_end: 0.9,
        y_end: 0.15,
    };
    let bright_fraction = region_color_fraction(&img, &top_edge, is_bright);
    assert!(
        bright_fraction > 0.01,
        "03-gameplay-start.png: expected visible content in top region, found {:.1}%",
        bright_fraction * 100.0
    );
}

#[test]
fn test_arena_shrink_has_danger_colors() {
    let path = screenshot_path("05-arena-shrink.png");
    if !path.exists() {
        eprintln!("Skipping: 05-arena-shrink.png not found");
        return;
    }

    let img = load_screenshot("05-arena-shrink.png");
    assert_not_blank(&img, "05-arena-shrink.png");

    // Danger zone should show orange-red colors somewhere
    let full = Region {
        x_start: 0.0,
        y_start: 0.0,
        x_end: 1.0,
        y_end: 1.0,
    };
    let danger_fraction = region_color_fraction(&img, &full, is_danger_red);
    assert!(
        danger_fraction > 0.001,
        "05-arena-shrink.png: expected danger zone colors, found {:.2}%",
        danger_fraction * 100.0
    );
}

#[test]
fn test_gameover_has_overlay_text() {
    let path = screenshot_path("09-gameover-complete.png");
    if !path.exists() {
        eprintln!("Skipping: 09-gameover-complete.png not found");
        return;
    }

    let img = load_screenshot("09-gameover-complete.png");
    assert_not_blank(&img, "09-gameover-complete.png");

    // Gold text overlay should be present
    let center = Region {
        x_start: 0.2,
        y_start: 0.1,
        x_end: 0.8,
        y_end: 0.5,
    };
    let gold_fraction = region_color_fraction(&img, &center, is_gold);
    assert!(
        gold_fraction > 0.002,
        "09-gameover-complete.png: expected gold overlay text, found {:.2}%",
        gold_fraction * 100.0
    );
}

#[test]
fn test_all_screenshots_not_blank() {
    let dir = Path::new(TEST_OUTPUT_DIR);
    if !dir.exists() {
        eprintln!("Skipping: test-output/ not found");
        return;
    }

    for name in EXPECTED_SCREENSHOTS {
        let path = screenshot_path(name);
        if path.exists() {
            let img = load_screenshot(name);
            assert_not_blank(&img, name);
        }
    }
}
