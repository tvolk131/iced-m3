use crate::{Button, Theme};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree},
};
use iced::{Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse};

pub(super) struct Group<'a, Message> {
    pub buttons: Vec<Button<'a, Message>>,
    pub spacing: f32,
    pub ratio: f32,
}
impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Group<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn children(&self) -> Vec<Tree> {
        self.buttons
            .iter()
            .map(|b| Tree::new(b as &dyn Widget<Message, Theme, Renderer>))
            .collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(
            &self.buttons,
            |tree, button| tree.diff(button as &dyn Widget<Message, Theme, Renderer>),
            |button| Tree::new(button as &dyn Widget<Message, Theme, Renderer>),
        );
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let count = self.buttons.len();
        if count == 0 {
            return layout::Node::new(Size::ZERO);
        }
        let spacing = self.spacing.min(limits.max().width / count as f32);
        let gap = spacing * count.saturating_sub(1) as f32;
        let available = (limits.max().width - gap).max(0.0);
        let base = if available.is_finite() {
            available / count as f32
        } else {
            self.buttons
                .iter_mut()
                .zip(&mut tree.children)
                .map(|(b, t)| {
                    b.layout_width(Length::Shrink);
                    b.group_compression(0.0, 0.0);
                    let width = b
                        .layout(
                            t,
                            renderer,
                            &layout::Limits::new(
                                Size::ZERO,
                                Size::new(f32::INFINITY, limits.max().height),
                            ),
                        )
                        .size()
                        .width;
                    b.layout_width(Length::Fill);
                    width
                })
                .fold(0.0, f32::max)
        };
        let mut widths = vec![base; count];
        let mut compressed = vec![(0.0, 0.0); count];
        // Each pressed child takes space only from its immediate neighbors.
        // Capacity accounts for simultaneous/reversing animations as well.
        let mut capacity: Vec<f32> = self
            .buttons
            .iter()
            .map(|b| b.compression_limit().min(base * 0.5))
            .collect();
        for i in 0..count {
            let amount = Button::<Message>::press_amount(&tree.children[i]).clamp(0.0, 1.25);
            let neighbors = usize::from(i > 0) + usize::from(i + 1 < count);
            if neighbors == 0 {
                continue;
            }
            let requested = base * self.ratio * amount / neighbors as f32;
            for j in [i.checked_sub(1), (i + 1 < count).then_some(i + 1)]
                .into_iter()
                .flatten()
            {
                let growth = requested.min(capacity[j]);
                widths[j] -= growth;
                widths[i] += growth;
                capacity[j] -= growth;
                if j < i {
                    compressed[j].1 += growth;
                } else {
                    compressed[j].0 += growth;
                }
            }
        }
        let mut x = 0.0;
        let mut height: f32 = 0.0;
        let mut nodes = Vec::with_capacity(count);
        for (((button, state), width), compression) in self
            .buttons
            .iter_mut()
            .zip(&mut tree.children)
            .zip(widths)
            .zip(compressed)
        {
            button.group_compression(compression.0, compression.1);
            let node = button
                .layout(
                    state,
                    renderer,
                    &layout::Limits::new(
                        Size::new(width, 0.0),
                        Size::new(width, limits.max().height),
                    ),
                )
                .move_to(Point::new(x, 0.0));
            height = height.max(node.size().height);
            nodes.push(node);
            x += width + spacing;
        }
        layout::Node::with_children(Size::new((x - spacing).max(0.0), height), nodes)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        op: &mut dyn Operation,
    ) {
        op.container(None, layout.bounds());
        op.traverse(&mut |op| {
            for ((b, t), l) in self
                .buttons
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                b.operate(t, l, renderer, op);
            }
        });
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut resized = false;
        for ((b, t), l) in self
            .buttons
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            let before = Button::<Message>::press_amount(t);
            b.update(t, event, l, cursor, renderer, clipboard, shell, viewport);
            resized |= before != Button::<Message>::press_amount(t);
        }
        if resized && self.ratio > 0.0 {
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((b, t), l) in self
            .buttons
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            b.draw(t, renderer, theme, style, l, cursor, viewport);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.buttons
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((b, t), l)| b.mouse_interaction(t, l, cursor, viewport, renderer))
            .max()
            .unwrap_or_default()
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let overlays: Vec<_> = self
            .buttons
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((b, t), l)| b.overlay(t, l, renderer, viewport, translation))
            .collect();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
