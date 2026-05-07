use std::rc::Rc;

use crate::ui::components::packets::StateModel;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, VirtualListScrollHandle};
use reticulum::iface::RxMessage;

pub struct PacketsPage {
    size: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
}

impl PacketsPage {
    pub fn new() -> Self {
        let items = (0..5000).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
        let size = Rc::new(items.iter().map(|_| size(px(200.), px(28.))).collect());

        Self {
            size,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = cx.global::<StateModel>().inner.clone();
        let count = state_handle.read(cx).items.len();

        div()
            .size_full()
            .overflow_x_scrollbar()
            .child(PacketsPage::render_header())
            .child(
                uniform_list("packet-list", count, move |range, _window, cx| {
                    let state = state_handle.read(cx);
                    let items = &state.items;

                    range
                        .map(|ix| {
                            let item = &items[ix];
                            // item.packet.

                            // div().child(format!("hops: {}", item.packet.header.hops))
                            div().child(PacketsPage::render_row(item))
                        })
                        .collect::<Vec<_>>()
                })
                .size_full(),
            )
    }
}

impl PacketsPage {
    fn render_row(item: &RxMessage) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .border_b_1()
            // .border_color(gpui::white())
            .child(
                div()
                    .w_1_4()
                    // .pl_2()
                    .pr_4()
                    .overflow_x_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(
                        div()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(item.address.to_hex_string()),
                    ),
            )
            .child(
                div()
                    .w_1_4()
                    // .pl_2()
                    .pr_4()
                    .overflow_x_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(item.packet.destination.to_hex_string()),
            )
            .child(
                div()
                    .w_1_4()
                    // .pl_2()
                    .pr_4()
                    .overflow_x_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(
                        item.packet
                            .transport
                            .map_or("NONE".into(), |a| a.to_hex_string()),
                    ),
            )
            .child(
                // div()
                //     .w_1_4()
                //     // .pl_2()
                //     .pr_4()
                //     .whitespace_nowrap()
                //     .overflow_x_hidden()
                //     .text_ellipsis()
                //     .child(format!("{}", item.packet.header.hops)),
                div()
                    .w_1_4()
                    .pr_4() // This creates the "safe zone"
                    .child(
                        div() // This child handles the text behavior
                            .w_full()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(format!("{}", item.packet.header.hops)),
                    ),
            )
    }

    fn render_header() -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .font_weight(gpui::FontWeight::BOLD)
            .child(div().w_1_4().child("Interface"))
            .child(div().w_1_4().child("Destination"))
            .child(div().w_1_4().child("Transport"))
            .child(div().w_1_4().child("Hops"))
    }
}
