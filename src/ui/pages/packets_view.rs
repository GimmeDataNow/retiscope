use std::rc::Rc;

use crate::ui::components::packets::{State, StateModel};
use gpui::*;
use gpui_component::VirtualListScrollHandle;
use gpui_component::scroll::ScrollableElement;

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
