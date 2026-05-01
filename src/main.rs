use gpui::*;
use gpui_component::*;

mod core;
mod db;
mod errors;
mod paths;
mod ui;

use ui::app_view::AppView;
use ui::components::assets::Assets;

fn main() {
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
