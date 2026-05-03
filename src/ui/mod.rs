pub mod app_view;
pub mod components;
pub mod pages;

use gpui::*;
use gpui_component::*;

use crate::core::AnnounceData;
use crate::daemon::StreamBundle;
use crate::ui::app_view::AppView;
use crate::ui::components::assets::Assets;

use tokio::sync::broadcast;

#[allow(dead_code)]
pub fn run(stream: StreamBundle) {
    let app = Application::new().with_assets(Assets);
    // app.run(move |cx| {
    //     gpui_component::init(cx);

    //     cx.spawn(|mut cx: &mut App| {
    //         async move {
    //             cx.open_window(WindowOptions::default(), |window, cx| {
    //                 // Pass it down to the view builder
    //                 let view = cx.new(|cx| AppView::build(cx, live_tx));
    //                 cx.new(|cx| Root::new(view, window, cx))
    //             })?;
    //             Ok::<_, anyhow::Error>(())
    //         }
    //     })
    //     .detach();
    // });

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
