//! Material key and ambient shadows, cached and clipped on both renderers.
use crate::{Element, Theme};
use iced::advanced::svg::{Handle, Renderer as _, Svg};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Color, Event, Length, Rectangle, Renderer, Size, Vector, mouse};
use std::cell::RefCell;

/// One Material box-shadow layer. `blur` is CSS blur radius, not Gaussian sigma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub opacity: f32,
}
/// Key and ambient measurements for elevation levels 0 through 5. Fractional
/// levels interpolate adjacent measurements for continuous hover transitions.
pub fn layers(level: f32) -> [ShadowLayer; 2] {
    const KEY: [(f32, f32); 6] = [(0., 0.), (1., 2.), (1., 2.), (1., 3.), (2., 3.), (4., 4.)];
    const AMBIENT: [(f32, f32, f32); 6] = [
        (0., 0., 0.),
        (1., 3., 1.),
        (2., 6., 2.),
        (4., 8., 3.),
        (6., 10., 4.),
        (8., 12., 6.),
    ];
    let level = if level.is_finite() {
        level.clamp(0., 5.)
    } else {
        0.
    };
    let a = level.floor() as usize;
    let b = (a + 1).min(5);
    let t = level - a as f32;
    let mix = |a, b| a + (b - a) * t;
    let opacity = level.min(1.);
    [
        ShadowLayer {
            y: mix(KEY[a].0, KEY[b].0),
            blur: mix(KEY[a].1, KEY[b].1),
            spread: 0.,
            opacity: 0.3 * opacity,
        },
        ShadowLayer {
            y: mix(AMBIENT[a].0, AMBIENT[b].0),
            blur: mix(AMBIENT[a].1, AMBIENT[b].1),
            spread: mix(AMBIENT[a].2, AMBIENT[b].2),
            opacity: 0.15 * opacity,
        },
    ]
}
pub(crate) type Cache = RefCell<Option<([f32; 7], Handle)>>;
fn outline(w: f32, h: f32, r: [f32; 4], x: f32, y: f32) -> String {
    let [tl, tr, br, bl] = r;
    format!(
        "M {} {y} H {} A {tr} {tr} 0 0 1 {} {} V {} A {br} {br} 0 0 1 {} {} H {} A {bl} {bl} 0 0 1 {x} {} V {} A {tl} {tl} 0 0 1 {} {y} Z",
        x + tl,
        x + w - tr,
        x + w,
        y + tr,
        y + h - br,
        x + w - br,
        y + h,
        x + bl,
        y + h - bl,
        y + tl,
        x + tl
    )
}
pub(crate) fn draw(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    radius: iced::border::Radius,
    level: f32,
    viewport: Rectangle,
    cache: &Cache,
) {
    if !theme.shadows
        || !level.is_finite()
        || level <= 0.
        || bounds.width <= 0.
        || bounds.height <= 0.
    {
        return;
    }
    let limit = bounds.width.min(bounds.height) / 2.;
    let radii = [
        radius.top_left,
        radius.top_right,
        radius.bottom_right,
        radius.bottom_left,
    ]
    .map(|r| r.clamp(0., limit));
    let key = [
        bounds.width,
        bounds.height,
        radii[0],
        radii[1],
        radii[2],
        radii[3],
        level,
    ];
    let layers = layers(level);
    let pad = layers
        .iter()
        .map(|l| l.y.abs() + l.spread + 1.5 * l.blur)
        .fold(0., f32::max)
        .ceil()
        + 1.;
    let (w, h) = (bounds.width + 2. * pad, bounds.height + 2. * pad);
    let mut cached = cache.borrow_mut();
    if cached.as_ref().is_none_or(|(old, _)| *old != key) {
        let hole = outline(bounds.width, bounds.height, radii, pad, pad);
        let mut svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}'><defs><clipPath id='outside'><path clip-rule='evenodd' d='M0 0 H{w} V{h} H0 Z {hole}'/></clipPath>"
        );
        for (i, l) in layers.iter().enumerate() {
            svg += &format!(
                "<filter id='s{i}' filterUnits='userSpaceOnUse' x='0' y='0' width='{w}' height='{h}' color-interpolation-filters='sRGB'><feGaussianBlur stdDeviation='{}'/></filter>",
                l.blur / 2.
            );
        }
        svg += "</defs><g clip-path='url(#outside)' fill='black'>";
        // CSS key shadow is painted above ambient.
        for i in [1, 0] {
            let l = layers[i];
            let s = l.spread;
            let path = outline(
                bounds.width + 2. * s,
                bounds.height + 2. * s,
                radii.map(|r| r + s),
                pad - s,
                pad - s + l.y,
            );
            svg += &format!(
                "<path d='{path}' opacity='{}' filter='url(#s{i})'/>",
                l.opacity
            );
        }
        svg += "</g></svg>";
        *cached = Some((key, Handle::from_memory(svg.into_bytes())));
    }
    renderer.draw_svg(
        Svg {
            handle: cached.as_ref().unwrap().1.clone(),
            color: Some(Color {
                a: 1.,
                ..theme.colors.shadow
            }),
            rotation: iced::Radians(0.),
            opacity: theme.colors.shadow.a,
        },
        Rectangle {
            x: bounds.x - pad,
            y: bounds.y - pad,
            width: w,
            height: h,
        },
        viewport,
    );
}

/// Paint a resizing surface with unscaled, clipped patches of one small shadow.
/// SVG filters run on the CPU even with wgpu. Keeping both the handle and raster
/// dimensions stable avoids large Gaussian blurs on each animation frame, while
/// tiling only straight edges preserves the corner shape and blur radius.
pub(crate) fn draw_resizing(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    full_height: f32,
    radius: f32,
    level: f32,
    viewport: Rectangle,
    cache: &Cache,
) {
    use iced::advanced::Renderer as _;
    let extent = layers(level)
        .iter()
        .map(|l| l.y.abs() + l.spread + 1.5 * l.blur)
        .fold(0., f32::max)
        .ceil()
        + 1.;
    let edge = radius + extent;
    // Small surfaces have interacting corner blurs and need the exact outline.
    if bounds.height < 2. * edge || bounds.width < 2. * edge {
        draw(
            renderer,
            theme,
            bounds,
            radius.into(),
            level,
            viewport,
            cache,
        );
        return;
    }
    let width = bounds.width.min(2. * edge + 64.);
    let height = full_height.min(2. * edge + 64.);
    let xs = shadow_patches(bounds.width, width, edge, extent);
    let ys = shadow_patches(bounds.height, height, edge, extent);
    for (xi, &(x, sx, w)) in xs.iter().enumerate() {
        for (yi, &(y, sy, h)) in ys.iter().enumerate() {
            // The middle is the transparent hole; only the perimeter is painted.
            if xi > 0 && xi + 1 < xs.len() && yi > 0 && yi + 1 < ys.len() {
                continue;
            }
            // Adjacent fractional layer edges get antialiased independently on
            // wgpu, leaving a faint seam. Share snapped boundaries between tiles;
            // the shadow itself keeps its original fractional translation.
            let left = (bounds.x + x).round();
            let top = (bounds.y + y).round();
            let Some(clip) = (Rectangle {
                x: left,
                y: top,
                width: (bounds.x + x + w).round() - left,
                height: (bounds.y + y + h).round() - top,
            })
            .intersection(&viewport) else {
                continue;
            };
            renderer.with_layer(clip, |renderer| {
                draw(
                    renderer,
                    theme,
                    Rectangle {
                        x: bounds.x + x - sx,
                        y: bounds.y + y - sy,
                        width,
                        height,
                    },
                    radius.into(),
                    level,
                    clip,
                    cache,
                );
            });
        }
    }
}
// (destination offset, source offset, length). Source regions never scale.
fn shadow_patches(length: f32, template: f32, edge: f32, extent: f32) -> Vec<(f32, f32, f32)> {
    let mut patches = vec![(-extent, -extent, edge + extent)];
    let mut start = edge;
    let step = (template - 2. * edge).max(1.);
    while start < length - edge {
        let size = step.min(length - edge - start);
        patches.push((start, edge, size));
        start += size;
    }
    patches.push((length - edge, template - edge, edge + extent));
    patches
}
/// Add Material elevation around an opaque surface without changing layout or
/// hit testing. The content supplies the matching surface fill and corner radius.
pub fn elevated<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    level: f32,
    radius: impl Into<iced::border::Radius>,
) -> Element<'a, Message> {
    Element::new(Elevated {
        content: content.into(),
        level,
        radius: radius.into(),
    })
}
struct Elevated<'a, Message> {
    content: Element<'a, Message>,
    level: f32,
    radius: iced::border::Radius,
}
impl<Message> Widget<Message, Theme, Renderer> for Elevated<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Cache>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Cache::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l)
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o);
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, Message>,
        v: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        th: &Theme,
        s: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        draw(
            r,
            th,
            l.bounds(),
            self.radius,
            self.level,
            *v,
            t.state.downcast_ref::<Cache>(),
        );
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, tr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_shadow_measurements_interpolate_and_invalid_levels_are_inert() {
        let level3 = layers(3.0);
        assert_eq!(
            level3[0],
            ShadowLayer {
                y: 1.0,
                blur: 3.0,
                spread: 0.0,
                opacity: 0.3
            }
        );
        assert_eq!(
            level3[1],
            ShadowLayer {
                y: 4.0,
                blur: 8.0,
                spread: 3.0,
                opacity: 0.15
            }
        );
        assert_eq!(
            layers(5.0)[1],
            ShadowLayer {
                y: 8.0,
                blur: 12.0,
                spread: 6.0,
                opacity: 0.15
            }
        );
        assert_eq!(layers(3.5)[1].y, 5.0);
        assert_eq!(layers(3.5)[1].blur, 9.0);
        for value in [f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(layers(value), layers(0.0));
        }
        assert_eq!(layers(100.0), layers(5.0));
    }
}
