//! Mechanical forwarding for wrappers whose content uses child tree slot zero.
//!
//! Each method is opt-in and forwards the incoming layout and viewport unchanged.
//! Wrappers that introduce a layout node must keep `layout.child(0)` methods
//! explicit, along with any custom clipping, state or input logic.
//! Omitted methods retain the widget's own implementation or iced's default.

macro_rules! delegate {
    ($message:ty, $field:ident, [$($method:ident),+ $(,)?]) => {
        $($crate::widget_delegate::delegate!(@$method $message, $field);)+
    };
    (@children $message:ty, $field:ident) => {
        fn children(&self) -> Vec<iced::advanced::widget::Tree> {
            vec![iced::advanced::widget::Tree::new(&self.$field)]
        }
    };
    (@diff $message:ty, $field:ident) => {
        fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
            tree.diff_children(std::slice::from_ref(&self.$field));
        }
    };
    (@size $message:ty, $field:ident) => {
        fn size(&self) -> iced::Size<iced::Length> {
            self.$field.as_widget().size()
        }
    };
    (@layout $message:ty, $field:ident) => {
        fn layout(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            renderer: &iced::Renderer,
            limits: &iced::advanced::layout::Limits,
        ) -> iced::advanced::layout::Node {
            self.$field.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
        }
    };
    (@operate $message:ty, $field:ident) => {
        fn operate(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn iced::advanced::widget::Operation,
        ) {
            self.$field.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
        }
    };
    (@update $message:ty, $field:ident) => {
        fn update(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            event: &iced::Event,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::mouse::Cursor,
            renderer: &iced::Renderer,
            clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, $message>,
            viewport: &iced::Rectangle,
        ) {
            self.$field.as_widget_mut().update(
                &mut tree.children[0], event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    };
    (@draw $message:ty, $field:ident) => {
        fn draw(
            &self,
            tree: &iced::advanced::widget::Tree,
            renderer: &mut iced::Renderer,
            theme: &$crate::Theme,
            style: &iced::advanced::renderer::Style,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::mouse::Cursor,
            viewport: &iced::Rectangle,
        ) {
            self.$field.as_widget().draw(
                &tree.children[0], renderer, theme, style, layout, cursor, viewport,
            );
        }
    };
    (@mouse_interaction $message:ty, $field:ident) => {
        fn mouse_interaction(
            &self,
            tree: &iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::mouse::Cursor,
            viewport: &iced::Rectangle,
            renderer: &iced::Renderer,
        ) -> iced::mouse::Interaction {
            self.$field.as_widget().mouse_interaction(
                &tree.children[0], layout, cursor, viewport, renderer,
            )
        }
    };
    (@overlay $message:ty, $field:ident) => {
        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'b>,
            renderer: &iced::Renderer,
            viewport: &iced::Rectangle,
            translation: iced::Vector,
        ) -> Option<iced::advanced::overlay::Element<'b, $message, $crate::Theme, iced::Renderer>> {
            self.$field.as_widget_mut().overlay(
                &mut tree.children[0], layout, renderer, viewport, translation,
            )
        }
    };
}

pub(crate) use delegate;
