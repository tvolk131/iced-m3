//! Compare the optimized gradient against the former circle/clip rendering.
use super::{reference::Image, theme};
use iced::advanced::{
    Renderer as _,
    renderer::Headless,
    svg::{Handle, Renderer as _, Svg},
};
use iced::{Color, Point, Rectangle, Renderer, Size, border::Radius};

// Keep the pre-optimization construction independent of the production helper.
fn original(size: Size, corners: Radius, origin: Point, progress: f32, alpha: f32) -> Handle {
    let (w, h) = (size.width, size.height);
    let limit = w.min(h) / 2.;
    let [tl, tr, br, bl] = [
        corners.top_left,
        corners.top_right,
        corners.bottom_right,
        corners.bottom_left,
    ]
    .map(|r| r.min(limit));
    let cx = w * (origin.x + (0.5 - origin.x) * progress);
    let cy = h * (origin.y + (0.5 - origin.y) * progress);
    let initial = w.max(h) * 0.1;
    let radius = initial + ((w * w + h * h).sqrt() / 2. + 10. - initial) * progress;
    let path = format!(
        "M{tl} 0H{}Q{w} 0 {w} {tr}V{}Q{w} {h} {} {h}H{bl}Q0 {h} 0 {}V{tl}Q0 0 {tl} 0Z",
        w - tr,
        h - br,
        w - br,
        h - bl
    );
    Handle::from_memory(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><defs><clipPath id="clip"><path d="{path}"/></clipPath><radialGradient id="edge"><stop offset="80%" stop-opacity="1"/><stop offset="100%" stop-opacity="0"/></radialGradient></defs><g opacity="{alpha}" clip-path="url(#clip)"><circle cx="{cx}" cy="{cy}" r="{radius}" fill="url(#edge)"/></g></svg>"#).into_bytes())
}

#[test]
fn bounded_ripple_preserves_gradient_shape_and_clipping() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
        iced::Font::DEFAULT,
        16.into(),
        Some(&backend),
    ))
    .unwrap();
    let viewport = Rectangle::with_size(Size::new(600., 210.));
    // Removing the 8-bit intermediate opacity improves alpha precision. Metal
    // composites in linear light, magnifying that rounding difference on dark
    // surfaces; allow five sRGB channel units there (two in software).
    let tolerance = if backend == "wgpu" { 5 } else { 2 };
    let mut failures = Vec::new();
    for (name, size, radius, origin) in [
        (
            "card",
            Size::new(520., 140.),
            Radius::from(12.),
            Point::new(0.15, 0.7),
        ),
        (
            "pill",
            Size::new(120., 40.),
            Radius::from(20.),
            Point::new(0.5, 0.5),
        ),
        (
            "asymmetric",
            Size::new(300., 90.),
            Radius {
                top_left: 0.,
                top_right: 24.,
                bottom_right: 16.,
                bottom_left: 4.,
            },
            Point::new(0.95, 0.05),
        ),
    ] {
        let bounds = Rectangle::new(Point::new(32.25, 32.25), size);
        for dark in [false, true] {
            let theme = theme(dark);
            for scale in [1., 1.25, 2.] {
                let physical = Size::new(
                    (viewport.width * scale) as u32,
                    (viewport.height * scale) as u32,
                );
                for progress in [0., 0.25, 0.6, 1.] {
                    for alpha in [0.12, 0.045] {
                        let color = Color {
                            a: alpha,
                            ..theme.colors.primary
                        };
                        renderer.reset(viewport);
                        crate::ripple::draw_bounded(
                            &mut renderer,
                            bounds,
                            viewport,
                            radius,
                            origin,
                            progress,
                            color,
                            &Default::default(),
                        );
                        let actual = renderer.screenshot(physical, scale, theme.colors.surface);
                        renderer.reset(viewport);
                        renderer.draw_svg(
                            Svg::from(&original(size, radius, origin, progress, alpha))
                                .color(Color { a: 1., ..color }),
                            bounds,
                            viewport,
                        );
                        let expected = renderer.screenshot(physical, scale, theme.colors.surface);
                        let delta = actual
                            .iter()
                            .zip(&expected)
                            .map(|(a, b)| a.abs_diff(*b))
                            .max()
                            .unwrap();
                        // Alpha is now multiplied during composition instead of
                        // being quantized into the SVG's 8-bit intermediate image.
                        if delta > tolerance {
                            failures.push(format!("{backend} {name} dark={dark} scale={scale} progress={progress} alpha={alpha}: max delta {delta}"));
                        }
                        if scale == 2. && alpha == 0.12 {
                            Image { width: physical.width, height: physical.height, pixels: actual }.write(&std::path::PathBuf::from(format!("target/visual-report/ripple-rendering-{backend}/{name}-{dark}-{progress}.png")));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
