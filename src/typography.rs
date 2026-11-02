//! Theme-owned Material type recipes, resolved before iced lays out native text.
use crate::{Theme, TypeScale};
use iced::{Font, font, widget};
use std::{
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
    sync::{Arc, OnceLock},
};

/// One text role in logical pixels. Weight and family are part of `font`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    pub font: Font,
    pub size: f32,
    pub line_height: f32,
    /// Recorded for custom renderers; iced 0.14 cannot apply letter spacing.
    pub tracking: f32,
}

impl TypeStyle {
    pub(crate) fn baseline(role: TypeScale) -> Self {
        let (size, line_height) = role.metrics();
        Self {
            font: role.font(),
            size,
            line_height,
            tracking: role.tracking(),
        }
    }

    pub(crate) fn resolved(self, fallback: Self) -> Self {
        Self {
            size: if self.size.is_finite() && self.size > 0.0 {
                self.size
            } else {
                fallback.size
            },
            line_height: if self.line_height.is_finite() && self.line_height > 0.0 {
                self.line_height
            } else {
                fallback.line_height
            },
            tracking: if self.tracking.is_finite() {
                self.tracking
            } else {
                fallback.tracking
            },
            ..self
        }
    }

    /// Ordinary iced text: explicit native builders still take precedence.
    pub fn text<'a>(self, content: impl widget::text::IntoFragment<'a>) -> widget::Text<'a, Theme> {
        crate::fonts::ensure_loaded();
        let style = self.resolved(Self::baseline(TypeScale::BodyLarge));
        widget::text(content)
            .size(style.size)
            .line_height(widget::text::LineHeight::Absolute(style.line_height.into()))
            .font(style.font)
            .shaping(widget::text::Shaping::Advanced)
            .wrapping(widget::text::Wrapping::WordOrGlyph)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Styles {
    baseline: [TypeStyle; 15],
    emphasized: [TypeStyle; 15],
}

/// Fifteen baseline roles and their independently customizable emphasized roles.
/// Clones share immutable recipes; indexing mutably uses copy-on-write.
#[derive(Debug, Clone, PartialEq)]
pub struct Typography(Arc<Styles>);

impl Default for Typography {
    fn default() -> Self {
        static DEFAULT: OnceLock<Arc<Styles>> = OnceLock::new();
        Self(Arc::clone(DEFAULT.get_or_init(|| {
            let baseline = TypeScale::ALL.map(TypeStyle::baseline);
            let emphasized = TypeScale::ALL.map(|role| {
                let mut style = TypeStyle::baseline(role);
                style.font.weight = match role {
                    TypeScale::TitleMedium
                    | TypeScale::TitleSmall
                    | TypeScale::LabelLarge
                    | TypeScale::LabelMedium
                    | TypeScale::LabelSmall => font::Weight::Bold,
                    _ => font::Weight::Medium,
                };
                style
            });
            Arc::new(Styles {
                baseline,
                emphasized,
            })
        })))
    }
}

impl Typography {
    pub(crate) fn widgets(&self) -> crate::Widgets {
        crate::Widgets {
            typography: self.clone(),
        }
    }
    /// Resolve invalid numeric overrides to the unchanged baseline recipe.
    pub fn style(&self, role: TypeScale) -> TypeStyle {
        self[role].resolved(TypeStyle::baseline(role))
    }

    pub fn emphasized(&self, role: TypeScale) -> TypeStyle {
        self.0.emphasized[role as usize].resolved(Self::default().0.emphasized[role as usize])
    }

    pub fn emphasized_mut(&mut self, role: TypeScale) -> &mut TypeStyle {
        &mut Arc::make_mut(&mut self.0).emphasized[role as usize]
    }

    /// Change the family of every role without discarding its weight/style.
    pub fn family(mut self, family: font::Family) -> Self {
        let styles = Arc::make_mut(&mut self.0);
        for style in styles
            .baseline
            .iter_mut()
            .chain(styles.emphasized.iter_mut())
        {
            style.font.family = family;
        }
        self
    }

    /// Scale text metrics, not component density or artwork. Invalid factors are ignored.
    pub fn scaled(mut self, factor: f32) -> Self {
        if factor.is_finite() && factor > 0.0 {
            let styles = Arc::make_mut(&mut self.0);
            for style in styles
                .baseline
                .iter_mut()
                .chain(styles.emphasized.iter_mut())
            {
                if (style.size * factor).is_finite() && (style.line_height * factor).is_finite() {
                    style.size *= factor;
                    style.line_height *= factor;
                    style.tracking *= factor;
                }
            }
        }
        self
    }

    pub fn text<'a>(
        &self,
        content: impl widget::text::IntoFragment<'a>,
        role: TypeScale,
    ) -> widget::Text<'a, Theme> {
        self.style(role).text(content)
    }
}

impl Index<TypeScale> for Typography {
    type Output = TypeStyle;
    fn index(&self, role: TypeScale) -> &Self::Output {
        &self.0.baseline[role as usize]
    }
}

impl IndexMut<TypeScale> for Typography {
    fn index_mut(&mut self, role: TypeScale) -> &mut Self::Output {
        &mut Arc::make_mut(&mut self.0).baseline[role as usize]
    }
}

impl Hash for Typography {
    fn hash<State: Hasher>(&self, state: &mut State) {
        for style in self.0.baseline.iter().chain(self.0.emphasized.iter()) {
            style.font.hash(state);
            for metric in [style.size, style.line_height, style.tracking] {
                (if metric == 0.0 { 0 } else { metric.to_bits() }).hash(state);
            }
        }
    }
}
