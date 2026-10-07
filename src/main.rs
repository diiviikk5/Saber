use gpui_kit::*;

struct Saber;

impl Render for Saber {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(0x0b0b0d)).child("Saber")
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Saber))
            .expect("failed to open window");
    });
}
