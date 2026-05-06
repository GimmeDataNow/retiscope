use std::rc::Rc;

use crate::ui::components::packets::{State, StateModel};
use gpui::*;
use gpui_component::VirtualListScrollHandle;
use gpui_component::scroll::ScrollableElement;
use gpui_component::v_virtual_list;
// use gpui_component::scroll::ScrollableElement;

pub struct PacketsPage {
    size: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
}

// impl PacketsPage {
//     pub fn new(_cx: &mut ViewContext<Self>) -> Self {
//         Self
//     }
// }

impl PacketsPage {
    pub fn new() -> Self {
        let items = (0..5000).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
        let size = Rc::new(items.iter().map(|_| size(px(200.), px(28.))).collect());

        // cx.observe(&state_model.inner, |_, _, cx| {
        //     // 3. Tell THIS view to re-render itself
        //     cx.notify();
        // })
        // .detach();

        Self {
            size,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. Access the global StateModel
        // let state_model = cx.global::<StateModel>();

        // 2. Read the inner state (the Entity<State>)
        // let state = state_model.inner.read(cx);

        // let packet_count = state.items.len();

        // let new_size: Rc<Vec<Size<Pixels>>> = Rc::new(
        //     state
        //         .items
        //         .iter()
        //         .map(|_| size(px(200.), px(28.)))
        //         .collect(),
        // );
        // v_virtual_list(
        //     cx.entity().clone(),
        //     "my-list",
        //     packet_count,
        //     |view, visible_range, _, cx| {
        //         visible_range
        //             .map(|ix| {
        //                 div()
        //                     .h(px(30.))
        //                     .w_full()
        //                     // .bg(cx.theme().sarcondary)
        //                     .child(format!("Item {}", ix))
        //             })
        //             .collect()
        //     },
        // )
        // .track_scroll(&self.scroll_handle)
        let state_handle = cx.global::<StateModel>().inner.clone();
        // let count = s
        let count = state_handle.read(cx).items.len();
        div().size_full().overflow_x_scrollbar().child(
            uniform_list("packet-list", count, move |range, _window, _app| {
                range
                    .map(|ix| div().child(format!("Packet #{}", ix)))
                    .collect()
            })
            .size_full(),
        )
    }
}
