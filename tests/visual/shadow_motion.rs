use super::{reference::Image, theme};
use iced::advanced::{Renderer as _, renderer::Headless};
use iced::{Point, Rectangle, Renderer, Size, border::Radius};

#[test]
fn tiled_shadows_preserve_integer_and_fractional_gaussian_outlines() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
        iced::Font::DEFAULT,
        16.into(),
        Some(&backend),
    ))
    .unwrap();
    let viewport = Rectangle::with_size(Size::new(620., 430.));
    let mut failures = Vec::new();
    for (shape, size, radius) in [
        ("card", Size::new(520., 250.), Radius::from(12.)),
        ("shallow", Size::new(520., 32.), Radius::from(16.)),
        ("narrow", Size::new(32., 320.), Radius::from(12.)),
        (
            "asymmetric",
            Size::new(360., 72.),
            Radius {
                top_left: 4.,
                top_right: 24.,
                bottom_right: 16.,
                bottom_left: 0.,
            },
        ),
        ("small", Size::new(40., 40.), Radius::from(20.)),
    ] {
        let bounds = Rectangle::new(Point::new(40.25, 40.25), size);
        for dark in [false, true] {
            let theme = theme(dark);
            for scale in [1., 1.25, 2.] {
                let physical = Size::new(
                    (viewport.width * scale) as u32,
                    (viewport.height * scale) as u32,
                );
                for level in [0.5, 1., 1.25, 2., 2.75, 3., 4., 4.5, 5.] {
                    renderer.reset(viewport);
                    crate::elevation::draw(
                        &mut renderer,
                        &theme,
                        bounds,
                        radius,
                        level,
                        viewport,
                        &Default::default(),
                    );
                    let tiled = renderer.screenshot(physical, scale, theme.colors.surface);
                    renderer.reset(viewport);
                    crate::elevation::draw_reference(
                        &mut renderer,
                        &theme,
                        bounds,
                        radius,
                        level,
                        viewport,
                        &Default::default(),
                    );
                    let exact = renderer.screenshot(physical, scale, theme.colors.surface);
                    // Independently translated SVGs can round their origins one
                    // physical pixel apart. Integer endpoints retain the existing
                    // two-unit contract. Fractional blurs need four: 125% raster
                    // rounding produced a one-pixel outlier in shallow corners.
                    let tolerance = if level.fract() == 0. { 2 } else { 4 };
                    let mut count = 0;
                    let mut worst = 0;
                    for (index, (&actual, &expected)) in tiled.iter().zip(&exact).enumerate() {
                        if actual.abs_diff(expected) <= 2 {
                            continue;
                        }
                        let width = physical.width as usize;
                        let height = physical.height as usize;
                        let (x, y, c) = (index / 4 % width, index / 4 / width, index % 4);
                        let mut low = u8::MAX;
                        let mut high = u8::MIN;
                        for row in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                            for col in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                                let value = exact[(row * width + col) * 4 + c];
                                low = low.min(value);
                                high = high.max(value);
                            }
                        }
                        let delta = low.saturating_sub(actual).max(actual.saturating_sub(high));
                        if delta > tolerance {
                            count += 1;
                            worst = worst.max(delta);
                        }
                    }
                    if count > 0 {
                        failures.push(format!("{shape} level={level} dark={dark} scale={scale} {backend}: {count} channels, worst neighborhood delta {worst}"));
                        for (name, pixels) in [("actual", tiled), ("expected", exact)] {
                            Image { width: physical.width, height: physical.height, pixels }.write(&std::path::PathBuf::from(format!("target/visual-report/shadow-audit/{backend}-{shape}-{level}-{dark}-{scale}-{name}.png")));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn hover_shadow_fades_continuously_and_preserves_surface_and_idle_pixels() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
        iced::Font::DEFAULT,
        16.into(),
        Some(&backend),
    ))
    .unwrap();
    let viewport = Rectangle::with_size(Size::new(420., 180.));
    let bounds = Rectangle::new(Point::new(32., 32.), Size::new(350., 112.));
    for dark in [false, true] {
        let theme = theme(dark);
        for resting in [0, 1] {
            let cache = Default::default();
            let mut images = Vec::new();
            for (step, amount) in [0., 0.25, 0.5, 0.75, 1., 0.5, 0.].into_iter().enumerate() {
                renderer.reset(viewport);
                crate::elevation::draw_transition(
                    &mut renderer,
                    &theme,
                    bounds,
                    12.into(),
                    (resting, resting + 1),
                    amount,
                    viewport,
                    &cache,
                );
                let image = Image {
                    width: 840,
                    height: 360,
                    pixels: renderer.screenshot(Size::new(840, 360), 2., theme.colors.surface),
                };
                image.write(&std::path::PathBuf::from(format!(
                    "target/visual-report/hover-shadow-{backend}/{resting}-{dark}-{step}.png"
                )));
                images.push(image);
            }
            assert!(
                images.windows(2).all(|pair| pair[0] != pair[1]),
                "Each sampled transition must visibly advance"
            );
            assert!(
                images[2] == images[5] && images[0] == images[6],
                "Reversing the transition must not accumulate pixels or leave a shadow behind"
            );
            for image in &images[1..] {
                assert!(
                    images[0].crop(100, 100, 600, 120) == image.crop(100, 100, 600, 120),
                    "Outset shadows must never tint the opaque surface interior"
                );
            }
        }
    }
}
