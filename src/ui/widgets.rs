//! Small hand-drawn controls, so every pixel follows Saber's theme.

use crate::assets::Icon;
use crate::theme::{MONO, theme};
use gpui_kit::prelude::*;
use gpui_kit::*;

pub type Handler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A quiet square button holding a single icon.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Icon,
    size: Pixels,
    active: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Handler>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: Icon) -> Self {
        Self {
            id: id.into(),
            icon,
            size: px(30.),
            active: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let c = theme(cx).colors;
        let tooltip = self.tooltip.clone();
        div()
            .id(self.id)
            .size(self.size)
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.))
            .cursor_pointer()
            .text_color(if self.active { c.accent } else { c.muted })
            .when(self.active, |d| d.bg(c.accent_a(0.12)))
            .hover(|d| d.bg(c.text.opacity(0.06)).text_color(c.text))
            .active(|d| d.bg(c.text.opacity(0.1)))
            .child(
                self.icon
                    .el()
                    .size(self.size * 0.5)
                    .text_color(if self.active { c.accent } else { c.muted }),
            )
            .when_some(tooltip, |d, text| {
                d.tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
                })
            })
            .when_some(self.on_click, |d, f| d.on_click(f))
    }
}

/// A keycap, e.g. `Ctrl K`.
pub fn kbd(label: &str, cx: &App) -> Div {
    let c = theme(cx).colors;
    div()
        .px(px(6.))
        .py(px(1.))
        .rounded(px(5.))
        .border_1()
        .border_color(c.line)
        .bg(c.surface_2)
        .font_family(MONO)
        .text_size(px(10.5))
        .text_color(c.muted)
        .child(label.to_string())
}

/// Tiny all-caps label used for section headings.
pub fn eyebrow(text: impl Into<SharedString>, cx: &App) -> Div {
    let c = theme(cx).colors;
    div()
        .font_family(MONO)
        .text_size(px(10.5))
        .text_color(c.faint)
        .child(text.into().to_uppercase())
}

/// An on/off switch with a soft accent glow when on.
pub fn switch(id: impl Into<ElementId>, on: bool, cx: &App) -> Stateful<Div> {
    let c = theme(cx).colors;
    let (w, h, knob) = (px(36.), px(20.), px(14.));
    div()
        .id(id)
        .w(w)
        .h(h)
        .flex_none()
        .rounded_full()
        .cursor_pointer()
        .p(px(3.))
        .flex()
        .items_center()
        .when(on, |d| {
            d.justify_end()
                .bg(c.accent)
                .shadow(vec![glow(c.accent, 12.)])
        })
        .when(!on, |d| d.justify_start().bg(c.text.opacity(0.12)))
        .child(div().size(knob).rounded_full().bg(if on {
            c.on_accent
        } else {
            c.text.opacity(0.7)
        }))
}

/// A row of mutually exclusive options.
pub fn segmented<T: Copy + PartialEq + 'static>(
    id: &'static str,
    options: &[(T, &'static str)],
    current: T,
    cx: &App,
    on_pick: impl Fn(T, &mut Window, &mut App) + Clone + 'static,
) -> Div {
    let c = theme(cx).colors;
    div()
        .flex()
        .p(px(3.))
        .gap(px(2.))
        .rounded(px(10.))
        .bg(c.text.opacity(0.05))
        .border_1()
        .border_color(c.line)
        .children(options.iter().enumerate().map(|(i, (value, label))| {
            let selected = *value == current;
            let value = *value;
            let on_pick = on_pick.clone();
            div()
                .id((id, i))
                .px(px(12.))
                .h(px(26.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .text_size(px(12.5))
                .cursor_pointer()
                .when(selected, |d| {
                    d.bg(c.surface)
                        .text_color(c.text)
                        .shadow(vec![soft_shadow(c.dark)])
                })
                .when(!selected, |d| {
                    d.text_color(c.muted).hover(|d| d.text_color(c.text))
                })
                .on_click(move |_, window, cx| on_pick(value, window, cx))
                .child(*label)
        }))
}

pub fn glow(color: Hsla, blur: f32) -> BoxShadow {
    BoxShadow {
        color: color.opacity(0.45),
        offset: point(px(0.), px(0.)),
        blur_radius: px(blur),
        spread_radius: px(0.),
        inset: false,
    }
}

pub fn soft_shadow(dark: bool) -> BoxShadow {
    BoxShadow {
        color: hsla(0., 0., 0., if dark { 0.45 } else { 0.12 }),
        offset: point(px(0.), px(1.)),
        blur_radius: px(3.),
        spread_radius: px(0.),
        inset: false,
    }
}

pub fn lift_shadow(dark: bool) -> BoxShadow {
    BoxShadow {
        color: hsla(0., 0., 0., if dark { 0.55 } else { 0.18 }),
        offset: point(px(0.), px(18.)),
        blur_radius: px(40.),
        spread_radius: px(-12.),
        inset: false,
    }
}
