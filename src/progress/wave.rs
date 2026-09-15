//! Wavy geometry follows the pinned Material Android drawing delegates.
//! Cubic paths are trimmed by arc length; no child rasterization or pixel readback.
//! See docs/LOADING_PROGRESS.md for source/profile details and numerical tolerance.
use super::{Kind, linear};
use iced::{
    Color, Rectangle, Renderer,
    advanced::svg::{Handle, Renderer as _, Svg},
};
use std::{cell::RefCell, fmt::Write, time::Duration};

pub(super) type Cache = RefCell<Option<(Vec<u32>, [Handle; 3])>>;
type Point = [f32; 2];
type Cubic = [Point; 4];
const TAU: f32 = std::f32::consts::TAU;
const SMOOTHNESS: f32 = 0.48;

fn mix(a: Point, b: Point, t: f32) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn split(c: Cubic, t: f32) -> (Cubic, Cubic) {
    let a = mix(c[0], c[1], t);
    let b = mix(c[1], c[2], t);
    let d = mix(c[2], c[3], t);
    let e = mix(a, b, t);
    let f = mix(b, d, t);
    let g = mix(e, f, t);
    ([c[0], a, e, g], [g, f, d, c[3]])
}
fn point(c: Cubic, t: f32) -> Point {
    split(c, t).0[3]
}
struct Measured {
    curve: Cubic,
    lengths: [f32; 33],
    start: f32,
}
struct Path {
    curves: Vec<Measured>,
    length: f32,
}
impl Path {
    fn new(curves: Vec<Cubic>) -> Self {
        let mut length = 0.;
        let curves = curves
            .into_iter()
            .map(|curve| {
                let mut lengths = [0.; 33];
                let mut previous = curve[0];
                for (i, distance) in lengths.iter_mut().enumerate().skip(1) {
                    let next = point(curve, i as f32 / 32.);
                    *distance = (next[0] - previous[0]).hypot(next[1] - previous[1]);
                    previous = next;
                }
                for i in 1..33 {
                    lengths[i] += lengths[i - 1];
                }
                let start = length;
                length += lengths[32];
                Measured {
                    curve,
                    lengths,
                    start,
                }
            })
            .collect();
        Self { curves, length }
    }
    fn part(&self, start: f32, end: f32, transform: impl Fn(Point) -> Point) -> String {
        let start = start.clamp(0., self.length);
        let end = end.clamp(start, self.length);
        let mut data = String::new();
        for c in &self.curves {
            let len = c.lengths[32];
            if c.start + len <= start || c.start >= end || len <= f32::EPSILON {
                continue;
            }
            let parameter = |distance: f32| {
                let distance = distance.clamp(0., len);
                let i = c.lengths.partition_point(|p| *p < distance).clamp(1, 32);
                ((i - 1) as f32
                    + (distance - c.lengths[i - 1])
                        / (c.lengths[i] - c.lengths[i - 1]).max(f32::EPSILON))
                    / 32.
            };
            let a = parameter(start - c.start);
            let b = parameter(end - c.start);
            let left = split(c.curve, b).0;
            let segment = if a > 0. && b > 0. {
                split(left, a / b).1
            } else {
                left
            };
            let p = segment.map(&transform);
            if data.is_empty() {
                write!(data, "M {} {} ", p[0][0], p[0][1]).unwrap();
            }
            write!(
                data,
                "C {} {} {} {} {} {} ",
                p[1][0], p[1][1], p[2][0], p[2][1], p[3][0], p[3][1]
            )
            .unwrap();
        }
        data
    }
}
fn stroke(svg: &mut String, path: &str, thickness: f32) {
    if !path.is_empty() && thickness > 0. {
        write!(svg,"<path d='{path}' fill='none' stroke='black' stroke-width='{thickness}' stroke-linecap='round'/>").unwrap();
    }
}
fn line(svg: &mut String, a: f32, b: f32, y: f32, thickness: f32) {
    if b <= a {
        return;
    }
    let thickness = thickness.min(b - a);
    stroke(
        svg,
        &format!(
            "M {} {y} H {}",
            a + thickness / 2.,
            (b - thickness / 2.).max(a + thickness / 2. + 0.0001)
        ),
        thickness,
    );
}
fn dot(svg: &mut String, x: f32, y: f32, r: f32) {
    if r > 0. {
        write!(svg, "<circle cx='{x}' cy='{y}' r='{r}'/>").unwrap();
    }
}

/// Android disjoint linear profile and its four independent endpoint curves.
pub(super) fn linear_segments(elapsed: Duration) -> [(f32, f32); 2] {
    let ms = (elapsed.as_secs_f64() * 1000. % 1800.) as f32;
    let ends = [
        (1267., 533., [0.2, 0., 0.8, 1.]),
        (1000., 567., [0.4, 0., 1., 1.]),
        (333., 850., [0., 0., 0.65, 1.]),
        (0., 750., [0.1, 0., 0.45, 1.]),
    ]
    .map(|(delay, duration, curve)| linear::bezier(((ms - delay) / duration).clamp(0., 1.), curve));
    [(ends[0], ends[1]), (ends[2], ends[3])]
}
/// Circular Expressive retreat: one section, 10% -> 87% -> 10% in six seconds.
pub(super) fn retreat(elapsed: Duration) -> (f32, f32) {
    let ms = (elapsed.as_secs_f64() * 1000. % 6000.) as f32;
    let ease = |t: f32| linear::bezier(t.clamp(0., 1.), [0.4, 0., 0.2, 1.]);
    let rotation = 1080. * ms / 6000.
        + [0., 1500., 3000., 4500.]
            .iter()
            .map(|d| 90. * ease((ms - d) / 500.))
            .sum::<f32>();
    let fraction = ease(ms / 3000.) - ease((ms - 3000.) / 3000.);
    (
        (rotation - 90.).rem_euclid(360.),
        360. * (0.1 + 0.77 * fraction),
    )
}
pub(super) fn color(colors: &[Color], fallback: Color, elapsed: Duration) -> Color {
    if colors.is_empty() {
        return fallback;
    }
    let cycles = elapsed.as_secs_f64() / 1.5;
    let index = cycles.floor() as usize % colors.len();
    let t = linear::bezier((cycles.fract() as f32 * 15.).min(1.), [0.4, 0., 0.2, 1.]);
    let a = colors[index];
    let b = colors[(index + 1) % colors.len()];
    let ch = |x: f32, y: f32| (x.powf(2.2) + (y.powf(2.2) - x.powf(2.2)) * t).powf(1. / 2.2);
    Color::from_rgba(
        ch(a.r, b.r),
        ch(a.g, b.g),
        ch(a.b, b.b),
        a.a + (b.a - a.a) * t,
    )
}

pub(super) fn draw(
    renderer: &mut Renderer,
    cache: &Cache,
    bounds: Rectangle,
    clip: Rectangle,
    kind: Kind,
    segments: &[(f32, f32)],
    start: f32,
    sweep: f32,
    thickness: f32,
    amplitude: f32,
    wavelength: f32,
    phase: f32,
    buffer: Option<f32>,
    indeterminate: bool,
    indicator: Color,
    track: Color,
) {
    let side = bounds.width.min(bounds.height);
    let width = if kind == Kind::Circular {
        side
    } else {
        bounds.width
    };
    let height = if kind == Kind::Circular {
        side
    } else {
        bounds.height
    };
    if width <= 0. || height <= 0. {
        return;
    }
    let mut key = vec![
        width.to_bits(),
        height.to_bits(),
        start.to_bits(),
        sweep.to_bits(),
        thickness.to_bits(),
        amplitude.to_bits(),
        wavelength.to_bits(),
        phase.to_bits(),
        buffer.unwrap_or(-1.).to_bits(),
        indeterminate as u32,
        (kind == Kind::Circular) as u32,
    ];
    for (a, b) in segments {
        key.extend([a.to_bits(), b.to_bits()]);
    }
    let mut cached = cache.borrow_mut();
    if cached.as_ref().is_none_or(|(old, _)| *old != key) {
        let mut paths = [String::new(), String::new(), String::new()];
        if kind == Kind::Linear {
            linear_paths(
                &mut paths,
                width,
                height,
                segments,
                thickness,
                amplitude,
                wavelength,
                phase,
                buffer,
                indeterminate,
            );
        } else {
            circular_paths(
                &mut paths, width, start, sweep, thickness, amplitude, wavelength, phase,
            );
        }
        let handles=paths.map(|p|Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}'>{p}</svg>").into_bytes()));
        *cached = Some((key, handles));
    }
    let rect = Rectangle {
        x: bounds.center_x() - width / 2.,
        y: bounds.center_y() - height / 2.,
        width,
        height,
    };
    let colors = [
        track,
        Color {
            a: indicator.a * 0.24,
            ..indicator
        },
        indicator,
    ];
    for (handle, color) in cached.as_ref().unwrap().1.iter().zip(colors) {
        if color.a > 0. {
            renderer.draw_svg(
                Svg::from(handle)
                    .color(Color { a: 1., ..color })
                    .opacity(color.a),
                rect,
                clip,
            );
        }
    }
}
fn linear_paths(
    paths: &mut [String; 3],
    width: f32,
    height: f32,
    segments: &[(f32, f32)],
    thickness: f32,
    amplitude: f32,
    wavelength: f32,
    phase: f32,
    buffer: Option<f32>,
    indeterminate: bool,
) {
    let thickness = thickness.min(height);
    let y = height / 2.;
    let mut position = 0.;
    for &(a, b) in segments {
        let end = a * width - 4. * ((1. - a) / 0.01).min(1.);
        line(&mut paths[0], position, end, y, thickness);
        if let Some(buffer) = buffer {
            line(
                &mut paths[1],
                position,
                end.min(buffer * width),
                y,
                thickness,
            );
        }
        position = position.max(b * width + 4. * (b / 0.01).min(1.));
    }
    line(&mut paths[0], position, width, y, thickness);
    if let Some(buffer) = buffer {
        line(
            &mut paths[1],
            position,
            width.min(buffer * width),
            y,
            thickness,
        );
    }
    let count = (width / wavelength).floor().clamp(1., 1024.) as usize;
    let wave = width / count as f32;
    // Measure at unit amplitude, like Android; scale vertically after trimming.
    let mut cubics = Vec::with_capacity((count + 1) * 2);
    for i in 0..=count {
        let x = i as f32 * wave;
        let half = wave / 2.;
        let h = half * SMOOTHNESS;
        cubics.push([[x, 1.], [x + h, 1.], [x + half - h, -1.], [x + half, -1.]]);
        cubics.push([
            [x + half, -1.],
            [x + half + h, -1.],
            [x + wave - h, 1.],
            [x + wave, 1.],
        ]);
    }
    let path = Path::new(cubics);
    let length = path.length * count as f32 / (count + 1) as f32;
    for &(a, b) in segments {
        if b <= a {
            continue;
        }
        if amplitude <= 0.0001 || (b - a) * width < thickness {
            line(&mut paths[2], a * width, b * width, y, thickness);
            continue;
        }
        let offset = thickness / 2. / width;
        let data = path.part(
            (a + offset + phase / count as f32) * length,
            (b - offset + phase / count as f32) * length,
            |p| [p[0] - phase * wave, y + p[1] * amplitude],
        );
        stroke(&mut paths[2], &data, thickness);
    }
    if !indeterminate {
        let progress = segments.last().map_or(0., |(_, b)| *b);
        let diameter = 4.0_f32.min(thickness).min(width * (1. - progress));
        dot(&mut paths[2], width - diameter / 2., y, diameter / 2.);
    }
}
fn circular_paths(
    paths: &mut [String; 3],
    size: f32,
    start: f32,
    sweep: f32,
    thickness: f32,
    amplitude: f32,
    wavelength: f32,
    phase: f32,
) {
    let thickness = thickness.min(size / 2.);
    let radius = (size - thickness) / 2.;
    if radius <= 0. {
        return;
    }
    let amplitude = amplitude.min(radius * 0.45);
    let sweep = sweep.clamp(0., 360.);
    let gap = (4. / radius).to_degrees() * (sweep / 3.6).min(1.);
    let center = size / 2.;
    let arc = |svg: &mut String, start: f32, span: f32| {
        if span <= 0. {
            return;
        }
        let length = radius * span.to_radians();
        let cap = thickness.min(length);
        if span >= 359.99 {
            write!(svg,"<circle cx='{center}' cy='{center}' r='{radius}' fill='none' stroke='black' stroke-width='{thickness}'/>").unwrap();
            return;
        }
        if length < thickness {
            let a = (start + span / 2.).to_radians();
            dot(
                svg,
                center + radius * a.cos(),
                center + radius * a.sin(),
                cap / 2.,
            );
            return;
        }
        let inset = (thickness / 2. / radius).to_degrees();
        let a = (start + inset).to_radians();
        let b = (start + span - inset).to_radians();
        stroke(
            svg,
            &format!(
                "M {} {} A {radius} {radius} 0 {} 1 {} {}",
                center + radius * a.cos(),
                center + radius * a.sin(),
                u8::from(span - 2. * inset > 180.),
                center + radius * b.cos(),
                center + radius * b.sin()
            ),
            thickness,
        );
    };
    arc(
        &mut paths[0],
        start + sweep + gap,
        (360. - sweep - 2. * gap).max(0.),
    );
    if sweep <= 0. {
        return;
    }
    if amplitude <= 0.0001 {
        arc(&mut paths[2], start, sweep);
        return;
    }
    let count = (TAU * radius / wavelength).floor().clamp(3., 512.) as usize;
    let control = TAU * radius / count as f32 / 2. * SMOOTHNESS;
    let mut cubics = Vec::with_capacity(count * 4);
    for i in 0..count * 4 {
        let a = i as f32 * std::f32::consts::PI / count as f32;
        let b = (i + 1) as f32 * std::f32::consts::PI / count as f32;
        let r1 = if i % 2 == 0 {
            radius
        } else {
            radius - 2. * amplitude
        };
        let r2 = if i % 2 == 0 {
            radius - 2. * amplitude
        } else {
            radius
        };
        let p = [r1 * a.cos(), r1 * a.sin()];
        let q = [r2 * b.cos(), r2 * b.sin()];
        cubics.push([
            p,
            [p[0] - control * a.sin(), p[1] + control * a.cos()],
            [q[0] + control * b.sin(), q[1] - control * b.cos()],
            q,
        ]);
    }
    let path = Path::new(cubics);
    let length = path.length / 2.;
    let inset = (thickness / 2. / radius) / TAU;
    let span = sweep / 360.;
    let phase = phase / count as f32;
    let rotation = (start - 360. * phase).to_radians();
    let transform = |p: Point| {
        [
            center + p[0] * rotation.cos() - p[1] * rotation.sin(),
            center + p[0] * rotation.sin() + p[1] * rotation.cos(),
        ]
    };
    if span <= 2. * inset {
        arc(&mut paths[2], start, sweep);
        return;
    }
    let (a, b) = if sweep >= 359.99 {
        (phase * length, (phase + 1.) * length)
    } else {
        ((phase + inset) * length, (phase + span - inset) * length)
    };
    let mut data = path.part(a, b, transform);
    if sweep >= 359.99 {
        data.push('Z');
    }
    stroke(&mut paths[2], &data, thickness);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linear_endpoints_expand_contract_and_meet_the_empty_boundary() {
        let at = |ms| linear_segments(Duration::from_millis(ms));
        assert_eq!(at(0), [(0., 0.), (0., 0.)]);
        assert_eq!(at(750)[1].1, 1.0);
        assert_eq!(at(1183)[1], (1., 1.));
        assert_eq!(at(1567)[0].1, 1.0);
        assert_eq!(at(2300), at(500));
        for index in 0..2 {
            let widths: Vec<_> = (0..1800)
                .step_by(10)
                .map(|t| {
                    let (a, b) = at(t)[index];
                    assert!(b >= a);
                    b - a
                })
                .collect();
            assert!(widths.iter().copied().fold(0., f32::max) > 0.4);
            assert!(widths[0] == 0. && *widths.last().unwrap() < 0.01);
        }
    }
    #[test]
    fn retreat_has_one_growing_contracting_section_and_continuous_cycles() {
        let at = |ms| retreat(Duration::from_millis(ms));
        assert!((at(0).1 - 36.).abs() < 0.001);
        assert!((at(3000).1 - 313.2).abs() < 0.001);
        assert!((at(5999).1 - 36.).abs() < 0.01);
        assert_eq!(at(6500), at(500));
        assert!(((at(5999).0 - at(6000).0 + 180.).rem_euclid(360.) - 180.).abs() < 0.3);
    }
    #[test]
    fn trimming_preserves_cubic_endpoints_and_handles_tiny_widths() {
        let p = Path::new(vec![[[0., 0.], [1., 0.], [2., 0.], [3., 0.]]]);
        assert!((p.length - 3.).abs() < 0.00001);
        assert!(p.part(0., 3., |p| p).starts_with("M 0 0 C"));
        for size in [0.1, 1., 4., 15., 48., 1200.] {
            let mut paths = [String::new(), String::new(), String::new()];
            linear_paths(
                &mut paths,
                size,
                10.,
                &[(0., 0.5)],
                4.,
                3.,
                40.,
                0.9,
                None,
                false,
            );
            circular_paths(&mut paths, size, -90., 180., 4., 1.6, 15., 0.9);
            assert!(
                paths
                    .iter()
                    .all(|p| !p.contains("NaN") && !p.contains("inf"))
            );
        }
    }
}
