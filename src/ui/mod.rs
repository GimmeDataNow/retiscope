pub mod app_view;
pub mod components;
pub mod pages;

use gpui::*;
use gpui_component::*;

use crate::ui::app_view::AppView;
use crate::ui::components::assets::Assets;

#[allow(dead_code)]
pub fn run() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| AppView::build(cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
