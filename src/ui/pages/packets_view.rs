use crate::ui::components::packets::StateModel;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use reticulum::iface::RxMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PacketColumn {
    Hops,
    Destination,
    Interface,
    Transport,
}

impl PacketColumn {
    fn label(&self) -> &'static str {
        match self {
            Self::Hops => "Hops",
            Self::Destination => "Destination",
            Self::Interface => "Interface",
            Self::Transport => "Transport",
        }
    }
}

/// The payload for the drag-and-drop operation
#[derive(Clone)]
struct DragColumn {
    from_index: usize,
}

impl Render for DragColumn {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x333333))
            .p_2()
            .border_1()
            .border_color(rgb(0xffffff))
            .child("Moving Column...")
    }
}

pub struct PacketsPage {
    column_order: Vec<PacketColumn>,
    column_widths: Vec<Pixels>,
    scroll_handle: UniformListScrollHandle,
}

impl PacketsPage {
    pub fn new() -> Self {
        Self {
            column_order: vec![
                PacketColumn::Hops,
                PacketColumn::Destination,
                PacketColumn::Interface,
                PacketColumn::Transport,
            ],
            column_widths: vec![px(60.), px(330.), px(330.), px(330.)],
            scroll_handle: UniformListScrollHandle::new(),
        }
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
// TODO: trim down on the use of the .clone()
impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = cx.global::<StateModel>().inner.clone();
        let count = state_handle.read(cx).items.len();

        let current_order = self.column_order.clone();
        let current_widths = self.column_widths.clone();

        div()
            .size_full()
            .overflow_x_scrollbar()
            .child(self.render_header(cx))
            .child(
                uniform_list("packet-list", count, move |range, _window, cx| {
                    let state = state_handle.read(cx);
                    let items = &state.items;

                    // Pre-clone once for the whole range to use inside the map
                    let order = current_order.clone();
                    let widths = current_widths.clone();

                    range
                        .map(|ix| {
                            let item = &items[ix];
                            // .into_any_element() is the "escape hatch" for lifetime errors (gemini did this)
                            Self::render_row(item, order.clone(), widths.clone()).into_any_element()
                        })
                        .collect::<Vec<_>>()
                })
                .track_scroll(self.scroll_handle.clone())
                .size_full(),
            )
    }
}

impl PacketsPage {
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .font_weight(gpui::FontWeight::BOLD)
            .children(self.column_order.iter().enumerate().map(|(idx, col)| {
                div()
                    .id(("col-header", idx))
                    .w(self.column_widths[idx])
                    // .cursor(CursorStyle::Pointer)
                    .child(col.label())
                    // FIX: Added the 4 arguments (drag, window, cx)
                    // Note: GPUI uses a specific signature for on_drag
                    .on_drag(
                        DragColumn { from_index: idx },
                        |drag, _point, _window, cx| cx.new(|_| drag.clone()),
                    )
                    .on_drop(cx.listener(move |this, dragged: &DragColumn, _window, cx| {
                        let from = dragged.from_index;
                        let to = idx;
                        if from != to {
                            let col = this.column_order.remove(from);
                            this.column_order.insert(to, col);

                            let width = this.column_widths.remove(from);
                            this.column_widths.insert(to, width);

                            cx.notify();
                        }
                    }))
            }))
    }

    fn render_row(
        item: &RxMessage,
        order: Vec<PacketColumn>,
        widths: Vec<Pixels>,
    ) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .border_b_1()
            .children(order.into_iter().enumerate().map(|(idx, col)| {
                let content = match col {
                    PacketColumn::Hops => format!("{}", item.packet.header.hops),
                    PacketColumn::Destination => item.address.to_hex_string(),
                    PacketColumn::Interface => item.packet.destination.to_hex_string(),
                    PacketColumn::Transport => item
                        .packet
                        .transport
                        .map_or("-".into(), |a| a.to_hex_string()),
                };
                row_element!(widths[idx], content)
            }))
    }
}
