use gpui::*;
use gpui_component::button::Button;

pub struct Settings {
    pub count: usize,
}

impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(format!("Settings Counter: {}", self.count))
            .child(
                Button::new("increment")
                    .child("Click to persist state")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    })),
            )
    }
}
