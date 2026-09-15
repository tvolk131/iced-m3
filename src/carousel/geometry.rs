//! Fitted Material keylines: large focal items, flexible medium items and small
//! edge items. Based on the published Android arrangement equations and keyline
//! shifting model; spacing is included in each cell, then removed from its mask.
use super::CarouselVariant;

#[derive(Clone, Copy, Debug)]
struct Keyline {
    loc: f32,
    offset: f32,
    size: f32,
}
#[derive(Clone)]
struct Frame {
    keys: Vec<Keyline>,
}
pub(super) struct Plan {
    pub pitch: f32,
    pub max: f32,
    spacing: f32,
    default: Frame,
    start: Vec<Frame>,
    end: Vec<Frame>,
    uncontained: bool,
}
impl Frame {
    fn new(cells: &[(f32, bool)], width: f32, pitch: f32) -> Self {
        let focal = cells.iter().position(|(_, focal)| *focal).unwrap_or(0);
        let focal_offset = cells[..focal].iter().map(|(size, _)| size).sum::<f32>() + pitch / 2.;
        let first_loc = focal_offset - focal as f32 * pitch;
        let mut keys = vec![Keyline {
            loc: first_loc - pitch,
            offset: -0.5,
            size: 1.,
        }];
        let mut edge = 0.;
        for (i, (size, _)) in cells.iter().enumerate() {
            keys.push(Keyline {
                loc: first_loc + i as f32 * pitch,
                offset: edge + size / 2.,
                size: *size,
            });
            edge += size;
        }
        keys.push(Keyline {
            loc: first_loc + cells.len() as f32 * pitch,
            offset: width + 0.5,
            size: 1.,
        });
        Self { keys }
    }
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            keys: self
                .keys
                .iter()
                .zip(&other.keys)
                .map(|(a, b)| Keyline {
                    loc: a.loc + (b.loc - a.loc) * t,
                    offset: a.offset + (b.offset - a.offset) * t,
                    size: a.size + (b.size - a.size) * t,
                })
                .collect(),
        }
    }
}
impl Plan {
    pub fn new(
        width: f32,
        height: f32,
        preferred: f32,
        spacing: f32,
        count: usize,
        variant: CarouselVariant,
    ) -> Self {
        let width = width.max(1.) + spacing;
        let preferred = (preferred + spacing).clamp(spacing + 1., width);
        let uncontained = variant == CarouselVariant::Uncontained;
        let centered = variant == CarouselVariant::HeroCenter;
        let hero = matches!(variant, CarouselVariant::Hero | CarouselVariant::HeroCenter);
        let target = if variant == CarouselVariant::FullScreen {
            width
        } else if hero {
            (height * 2. + spacing).min(width).max(spacing + 1.)
        } else {
            preferred
        };
        let cells = if uncontained {
            vec![(preferred, true)]
        } else if variant == CarouselVariant::FullScreen || count <= 1 || width < 80. + spacing * 2.
        {
            vec![(width, true)]
        } else {
            arrangement(width, target, spacing, count, hero, centered)
        };
        let pitch = cells
            .iter()
            .find(|(_, f)| *f)
            .map_or(target, |(size, _)| *size);
        let default = Frame::new(&cells, width, pitch);
        let mut start = vec![default.clone()];
        let mut shifted = cells.clone();
        while !shifted.first().unwrap().1 {
            let cell = shifted.remove(0);
            let at = shifted.iter().rposition(|(_, f)| *f).unwrap() + 1;
            shifted.insert(at, cell);
            start.push(Frame::new(&shifted, width, pitch));
        }
        let mut end = vec![default.clone()];
        let mut shifted = cells;
        while !shifted.last().unwrap().1 {
            let cell = shifted.pop().unwrap();
            let at = shifted
                .iter()
                .position(|(size, f)| *f || *size > cell.0)
                .unwrap_or(0);
            shifted.insert(at, cell);
            end.push(Frame::new(&shifted, width, pitch));
        }
        Self {
            pitch,
            max: ((count as f32 * pitch - width) / pitch).max(0.),
            spacing,
            default,
            start,
            end,
            uncontained,
        }
    }
    fn frame(&self, position: f32) -> Frame {
        let offset = position.clamp(0., self.max) * self.pitch;
        let first = |f: &Frame| f.keys[0].loc;
        let last = |f: &Frame| f.keys.last().unwrap().loc;
        let start_distance = first(self.start.last().unwrap()) - first(&self.default);
        let end_distance = last(&self.default) - last(self.end.last().unwrap());
        let (steps, amount, from_start) = if offset < start_distance {
            (&self.start, start_distance - offset, true)
        } else if offset > self.max * self.pitch - end_distance {
            (
                &self.end,
                offset - (self.max * self.pitch - end_distance),
                false,
            )
        } else {
            return self.default.clone();
        };
        let distance = |f: &Frame| {
            if from_start {
                first(f) - first(&self.default)
            } else {
                last(&self.default) - last(f)
            }
        };
        for pair in steps.windows(2) {
            let low = distance(&pair[0]);
            let high = distance(&pair[1]);
            if amount <= high {
                return pair[0].interpolate(
                    &pair[1],
                    ((amount - low) / (high - low).max(0.001)).clamp(0., 1.),
                );
            }
        }
        steps.last().unwrap().clone()
    }
    pub fn geometry(
        &self,
        position: f32,
        indices: impl IntoIterator<Item = usize>,
    ) -> Vec<(f32, f32)> {
        let offset = position.clamp(0., self.max) * self.pitch;
        let frame = self.frame(position);
        indices
            .into_iter()
            .map(|i| {
                let loc = (i as f32 + 0.5) * self.pitch - offset;
                if self.uncontained {
                    return (loc - self.pitch / 2., self.pitch - self.spacing);
                }
                let keys = &frame.keys;
                let j = keys
                    .partition_point(|k| k.loc < loc)
                    .clamp(1, keys.len() - 1);
                let (a, b) = (keys[j - 1], keys[j]);
                let t = ((loc - a.loc) / (b.loc - a.loc).max(0.001)).clamp(0., 1.);
                let size = (a.size + (b.size - a.size) * t - self.spacing).max(0.);
                let center = if loc < keys[0].loc {
                    keys[0].offset + loc - keys[0].loc
                } else if loc > keys.last().unwrap().loc {
                    keys.last().unwrap().offset + loc - keys.last().unwrap().loc
                } else {
                    a.offset + (b.offset - a.offset) * t
                };
                (center - size / 2. - self.spacing / 2., size)
            })
            .collect()
    }
    pub fn visible_range(&self, position: f32, count: usize) -> std::ops::Range<usize> {
        let frame = self.frame(position);
        let offset = position.clamp(0., self.max) * self.pitch;
        let start = ((offset + frame.keys[0].loc) / self.pitch - 2.)
            .floor()
            .max(0.) as usize;
        let end = ((offset + frame.keys.last().unwrap().loc) / self.pitch + 2.)
            .ceil()
            .max(0.) as usize;
        start.min(count)..end.min(count)
    }
}

fn arrangement(
    width: f32,
    target: f32,
    spacing: f32,
    count: usize,
    hero: bool,
    centered: bool,
) -> Vec<(f32, bool)> {
    let small_target = (target / 3.).clamp(40. + spacing, 56. + spacing);
    let small_count = if centered { 2 } else { 1 };
    let mut best = None;
    let mut priority = 0;
    let max_large = (width / target).ceil().max(1.) as usize;
    for large_count in (1..=max_large.min(count)).rev() {
        for medium_count in if hero { &[0][..] } else { &[1, 0][..] } {
            priority += 1;
            if large_count + medium_count + small_count > count {
                continue;
            }
            let n = large_count as f32;
            let m = *medium_count as f32;
            let s = small_count as f32;
            let desired = n * target + m * (target + small_target) / 2. + s * small_target;
            let small = (small_target + (width - desired) / s).clamp(40. + spacing, 56. + spacing);
            let mut large = (width - (s + m / 2.) * small) / (n + m / 2.);
            let mut medium = (large + small) / 2.;
            if m > 0. {
                let flex = ((target - large) * n).clamp(-medium * 0.1 * m, medium * 0.1 * m);
                large += flex / n;
                medium -= flex / m;
            }
            if large <= small || m > 0. && (medium <= small || medium >= large) {
                continue;
            }
            let cost = (target - large).abs() * priority as f32;
            if best.as_ref().is_none_or(|(c, _)| cost < *c) {
                let mut cells = Vec::new();
                if centered {
                    cells.push((small, false));
                }
                cells.extend(std::iter::repeat_n((large, true), large_count));
                cells.extend(std::iter::repeat_n((medium, false), *medium_count));
                cells.push((small, false));
                best = Some((cost, cells));
            }
        }
    }
    best.map_or_else(|| vec![(width, true)], |(_, cells)| cells)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fitted_keylines_fill_the_viewport_and_keep_both_ends_visible() {
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Hero,
            CarouselVariant::HeroCenter,
            CarouselVariant::FullScreen,
            CarouselVariant::Uncontained,
        ] {
            for width in [64., 320., 600., 1100.] {
                for count in [1, 2, 5, 10000] {
                    let p = Plan::new(width, 200., 280., 8., count, variant);
                    let first = p.geometry(0., [0])[0];
                    let last = p.geometry(p.max, [count - 1])[0];
                    assert!(
                        first.0 >= -0.01 && first.0 + first.1 <= width + 0.01,
                        "{variant:?} {width} {count}: {first:?}"
                    );
                    assert!(
                        last.0 >= -0.01 && last.0 + last.1 <= width + 0.01,
                        "{variant:?} {width} {count}: {last:?}"
                    );
                    assert!(p.visible_range(p.max / 2., count).len() < 40);
                }
            }
        }
    }
}
