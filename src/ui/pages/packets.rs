use gpui::*;
use gpui_component::Icon;
use gpui_component::VirtualList;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::sidebar::*;
use gpui_component::theme::Theme;
use gpui_component::*;

use std::rc::Rc;

use crate::ui::app_view::AppView;

pub struct PacketsPage {
    pub app_view: Entity<AppView>,
    scroll_handle: VirtualListScrollHandle,
}

impl PacketsPage {
    pub fn new(app_view: Entity<AppView>) -> Self {
        Self {
            app_view,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Render for PacketsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let packets = self.app_view.read(cx).packets.clone();
        let count = packets.len();

        // Item sizes must be Rc<Vec<...>> and kept in sync with packet count
        let item_sizes = Rc::new(
            // this is an issue TODO!!!!!!
            (0..count)
                .map(|_| gpui::size(px(200.), px(24.)))
                .collect::<Vec<_>>(),
        );

        v_virtual_list(
            cx.entity().clone(),
            "packet-list",
            item_sizes,
            move |_view, visible_range, _, _cx| {
                visible_range
                    .map(|ix| {
                        div()
                            .w_full()
                            .h(px(24.))
                            .px_3()
                            .text_sm()
                            .font_family("monospace")
                            .child(
                                // packets[ix].to_string()
                                format!(
                                    "{ix} destination {}",
                                    packets[ix].packet.destination.to_hex_string()
                                ),
                            )
                    })
                    .collect()
            },
        )
        .track_scroll(&self.scroll_handle)
    }
}

// impl Render for PacketsPage {
//     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//         let packets = &self.app_view.read(cx).packets;
//         // VirtualList::grid(self)
//         div()
//             .flex()
//             .flex_col()
//             .size_full()
//             .overflow_y_scrollbar()
//             .children(packets.iter().enumerate().map(|(i, pkt)| {
//                 div().px_3().py_1().text_sm().child(format!(
//                     "{i} destination {}",
//                     pkt.packet.destination.to_hex_string()
//                 ))
//             }))
//     }
// }
