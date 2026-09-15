//! Modal input suspension is distinct from real window activity. iced has no
//! custom widget event, so private, synchronous update scopes carry that context.
//! Native controls still receive cancellation; opt-in animation clocks can use
//! the actual window focus. Scopes nest and restore even during unwinding.
use crate::Element;
use iced::advanced::{Clipboard, Layout, Shell, widget::Tree};
use iced::{Event, Rectangle, Renderer, mouse, window};
use std::cell::Cell;

#[derive(Clone, Copy)]
enum Context {
    Normal,
    Interaction,
    Covered { focused: bool },
}
thread_local! {
    static CONTEXT: Cell<Context> = const { Cell::new(Context::Normal) };
}
fn scoped<T>(context: Context, f: impl FnOnce() -> T) -> T {
    struct Restore(Context);
    impl Drop for Restore {
        fn drop(&mut self) {
            CONTEXT.set(self.0);
        }
    }
    let _restore = Restore(CONTEXT.replace(context));
    f()
}
pub(crate) fn interaction<T>(f: impl FnOnce() -> T) -> T {
    let context = match CONTEXT.get() {
        context @ Context::Covered { .. } => context,
        _ => Context::Interaction,
    };
    scoped(context, f)
}
pub(crate) fn is_covered() -> bool {
    matches!(CONTEXT.get(), Context::Covered { .. })
}
/// Only modal hosts and independent progress clocks opt into this distinction.
/// Other widgets keep treating synthetic Unfocused as interaction suspension.
pub(crate) fn window_focus(event: &Event) -> Option<bool> {
    match CONTEXT.get() {
        Context::Covered { focused } => Some(focused),
        Context::Interaction => None,
        Context::Normal => match event {
            Event::Window(window::Event::Focused) => Some(true),
            Event::Window(window::Event::Unfocused) => Some(false),
            _ => None,
        },
    }
}

/// Update visible covered content without permitting actions, input capture or
/// IME. Actual focus changes reach animation clocks, while native inputs and
/// timeout-driven interaction widgets continue receiving Unfocused.
pub(crate) fn update_covered<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
    focused: bool,
) {
    let event = match event {
        Event::Window(window::Event::RedrawRequested(_)) => event,
        Event::Window(window::Event::Focused | window::Event::Unfocused) => {
            &Event::Window(window::Event::Unfocused)
        }
        _ => return,
    };
    // The outermost host knows the real window state. A nested host may have
    // missed focus events while hidden; it must inherit this value.
    let context = match CONTEXT.get() {
        context @ Context::Covered { .. } => context,
        _ => Context::Covered { focused },
    };
    let mut ignored = Vec::new();
    let mut local = Shell::new(&mut ignored);
    scoped(context, || {
        content.as_widget_mut().update(
            tree,
            event,
            layout,
            mouse::Cursor::Unavailable,
            renderer,
            clipboard,
            &mut local,
            viewport,
        );
    });
    shell.request_redraw_at(local.redraw_request());
    if local.is_layout_invalid() {
        shell.invalidate_layout();
    }
    if local.are_widgets_invalid() {
        shell.invalidate_widgets();
    }
}
