use crate::ui::components::packets::StateModel;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use reticulum::iface::RxMessage;

pub struct PacketsPage {
    w: Vec<Pixels>,
    // size: Rc<Vec<Size<Pixels>>>,
    scroll_handle: UniformListScrollHandle,
}

impl PacketsPage {
    pub fn new() -> Self {
        //                      -Hops     -Dest     -Time    -Transport -Interface -etc
        let column_widths = vec![px(60.), px(330.), px(330.), px(330.), px(150.), px(150.)];
        Self {
            w: column_widths,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }
}

impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = cx.global::<StateModel>().inner.clone();
        let count = state_handle.read(cx).items.len();

        let widths = self.w.clone();

        // self.scroll_handle.scroll_to_item(ix, ScrollStrategy::Bottom);

        div()
            .size_full()
            .overflow_x_scrollbar()
            .child(PacketsPage::render_header(&self))
            .child(
                uniform_list("packet-list", count, move |range, _window, cx| {
                    let state = state_handle.read(cx);
                    let items = &state.items;

                    // let widths = self.w.clone();
                    range
                        .map(|ix| {
                            let item = &items[ix];
                            // item.packet.

                            // div().child(format!("hops: {}", item.packet.header.hops))
                            div().child(PacketsPage::render_row(item, &widths))
                        })
                        .collect::<Vec<_>>()
                })
                // .track_scroll(self.scroll_handle.clone())
                .size_full(),
            )
    }
}

macro_rules! row_element {
    ($width:expr, $item:expr) => {
        div().w($width).pr_4().child(
            div()
                .w_full()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child($item)
                .text_right(),
        )
    };
}

impl PacketsPage {
    #[rustfmt::skip]
    fn render_row(item: &RxMessage, w: &Vec<Pixels>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .border_b_1()
            .child(row_element!(w[0], format!("{}", item.packet.header.hops)                                        ))
            .child(row_element!(w[1], format!("{}", item.address.to_hex_string())                                   ))
            .child(row_element!(w[2], format!("{}", item.packet.destination.to_hex_string())                        ))
            .child(row_element!(w[3], format!("{}", item.packet.transport.map_or("-".into(), |a| a.to_hex_string()))))
            // data last
    }

    fn render_header(&self) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .font_weight(gpui::FontWeight::BOLD)
            // time - hops - dest - protocol - transport - interface
            .child(div().w(*&self.w[0]).child("Hops"))
            .child(div().w(*&self.w[1]).child("Destination"))
            .child(div().w(*&self.w[2]).child("Interface"))
            .child(div().w(*&self.w[3]).child("Transport"))
    }
}
