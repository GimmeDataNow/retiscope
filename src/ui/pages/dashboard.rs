use gpui::*;

pub struct Dashboard {
    pub title: String,
}

impl Render for Dashboard {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(format!("{} Page", self.title))
    }
}
