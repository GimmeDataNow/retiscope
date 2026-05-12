use crate::ui::components::packets::StateModel;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use reticulum::iface::RxMessage;

// ─── Column Definition ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PacketColumn {
    Hops,
    Destination,
    Interface,
    Transport,
    Context,
    DestinationType,
    HeaderType,
    PropagationType,
    IfacFlag,
}

impl PacketColumn {
    fn label(&self) -> &'static str {
        match self {
            Self::Hops => "Hops",
            Self::Destination => "Destination",
            Self::Interface => "Interface",
            Self::Transport => "Transport",
            Self::Context => "Context",
            Self::DestinationType => "Dest Type",
            Self::HeaderType => "Header",
            Self::PropagationType => "Propagation",
            Self::IfacFlag => "Ifac Flag",
        }
    }

    fn default_width(&self) -> Pixels {
        match self {
            Self::Hops => px(56.),
            Self::Destination => px(300.),
            Self::Interface => px(300.),
            Self::Transport => px(300.),
            Self::Context => px(130.),
            Self::DestinationType => px(110.),
            Self::HeaderType => px(100.),
            Self::PropagationType => px(110.),
            Self::IfacFlag => px(90.),
        }
    }

    /// Columns that show enum values get a subtle badge treatment.
    fn is_badge(&self) -> bool {
        matches!(
            self,
            Self::Context
                | Self::DestinationType
                | Self::HeaderType
                | Self::PropagationType
                | Self::IfacFlag
        )
    }

    /// Columns that show hex addresses use monospace.
    fn is_hex(&self) -> bool {
        matches!(self, Self::Destination | Self::Interface | Self::Transport)
    }
}

// ─── Drag payload ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct DragColumn {
    from_index: usize,
}

impl Render for DragColumn {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(rgb(0x2a2d3a))
            .border_1()
            .border_color(rgb(0x5865f2))
            .shadow_lg()
            .text_sm()
            .text_color(rgb(0xdde1f0))
            .child("Moving column…")
    }
}

// ─── Column State ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ColumnState {
    col: PacketColumn,
    width: Pixels,
    visible: bool,
}

impl ColumnState {
    fn new(col: PacketColumn) -> Self {
        Self {
            width: col.default_width(),
            visible: true,
            col,
        }
    }
}

// ─── Page ─────────────────────────────────────────────────────────────────────

pub struct PacketsPage {
    /// Ordered list of all columns (visible or not).
    columns: Vec<ColumnState>,
    scroll_handle: UniformListScrollHandle,
    /// Whether the column-picker popover is open.
    picker_open: bool,
}

impl PacketsPage {
    pub fn new() -> Self {
        Self {
            columns: vec![
                ColumnState::new(PacketColumn::Hops),
                ColumnState::new(PacketColumn::Destination),
                ColumnState::new(PacketColumn::Context),
                ColumnState::new(PacketColumn::DestinationType),
                ColumnState::new(PacketColumn::HeaderType),
                ColumnState::new(PacketColumn::PropagationType),
                ColumnState::new(PacketColumn::IfacFlag),
                ColumnState::new(PacketColumn::Interface),
                ColumnState::new(PacketColumn::Transport),
            ],
            scroll_handle: UniformListScrollHandle::new(),
            picker_open: false,
        }
    }

    /// Indices into `self.columns` that are currently visible, in order.
    fn visible_indices(&self) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.visible)
            .map(|(i, _)| i)
            .collect()
    }
}

// ─── Render ───────────────────────────────────────────────────────────────────

macro_rules! cell {
    ($width:expr, $child:expr) => {
        div().w($width).flex_shrink_0().pr_3().child(
            div()
                .w_full()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child($child),
        )
    };
}

impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = cx.global::<StateModel>().inner.clone();
        let count = state_handle.read(cx).items.len();

        // Snapshot visible column states for the row renderer (avoids lifetime issues).
        let visible: Vec<(PacketColumn, Pixels)> = self
            .visible_indices()
            .into_iter()
            .map(|i| (self.columns[i].col, self.columns[i].width))
            .collect();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x13151f))
            .text_color(rgb(0xc8cde0))
            .text_sm()
            .font_family("JetBrains Mono, Cascadia Code, Fira Code, monospace")
            // Header bar (column headers + toolbar)
            .child(self.render_toolbar(cx))
            .child(self.render_header(cx))
            // Packet rows
            .child(
                uniform_list("packet-list", count, {
                    let visible = visible.clone();
                    move |range, _window, cx| {
                        let state = state_handle.read(cx);
                        let items = &state.items;
                        let vis = visible.clone();
                        range
                            .map(|ix| Self::render_row(&items[ix], &vis, ix).into_any_element())
                            .collect::<Vec<_>>()
                    }
                })
                .track_scroll(self.scroll_handle.clone())
                .flex_1(),
            )
            // Column visibility picker (rendered on top when open)
            .when(self.picker_open, |el| {
                el.child(self.render_column_picker(cx))
            })
    }
}

impl PacketsPage {
    // ── Toolbar ───────────────────────────────────────────────────────────────

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let picker_open = self.picker_open;

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(rgb(0x1e2130))
            .bg(rgb(0x0f1119))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x5c6380))
                    .font_weight(FontWeight::MEDIUM)
                    .child("PACKET MONITOR"),
            )
            .child(
                // Columns toggle button
                div()
                    .id("col-picker-btn")
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor(CursorStyle::PointingHand)
                    .bg(if picker_open {
                        rgb(0x252840)
                    } else {
                        rgb(0x1a1d2e)
                    })
                    .border_1()
                    .border_color(if picker_open {
                        rgb(0x5865f2)
                    } else {
                        rgb(0x2a2d3e)
                    })
                    .hover(|s| s.bg(rgb(0x252840)).border_color(rgb(0x5865f2)))
                    .text_xs()
                    .text_color(rgb(0x9ba3c0))
                    .child("⊞ Columns")
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.picker_open = !this.picker_open;
                        cx.notify();
                    })),
            )
    }

    // ── Column Picker Popover ─────────────────────────────────────────────────

    fn render_column_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Floating panel – positioned top-right via absolute layout
        div()
            .absolute()
            .top(px(68.)) // below toolbar + header
            .right(px(8.))
            // .z_index(100)
            .w(px(200.))
            .rounded_lg()
            .bg(rgb(0x1a1d2e))
            .border_1()
            .border_color(rgb(0x2e3250))
            .shadow_xl()
            .p_2()
            .flex()
            .flex_col()
            .gap_px()
            // Title
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x5c6380))
                    .child("TOGGLE COLUMNS"),
            )
            .children(
                self.columns
                    .iter()
                    .enumerate()
                    .map(|(idx, col_state)| {
                        let visible = col_state.visible;
                        let label = col_state.col.label();

                        div()
                            .id(("picker-row", idx))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor(CursorStyle::PointingHand)
                            .hover(|s| s.bg(rgb(0x252840)))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                // Always keep at least one column visible
                                let visible_count =
                                    this.columns.iter().filter(|c| c.visible).count();
                                if this.columns[idx].visible && visible_count <= 1 {
                                    return;
                                }
                                this.columns[idx].visible = !this.columns[idx].visible;
                                cx.notify();
                            }))
                            // Checkbox indicator
                            .child(
                                div()
                                    .w(px(14.))
                                    .h(px(14.))
                                    .rounded_sm()
                                    .border_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .border_color(if visible {
                                        rgb(0x5865f2)
                                    } else {
                                        rgb(0x3a3f5c)
                                    })
                                    .bg(if visible {
                                        rgb(0x5865f2)
                                    } else {
                                        rgb(0x13151f)
                                    })
                                    .when(visible, |el| {
                                        el.text_color(rgb(0xffffff)).text_xs().child("✓")
                                    }),
                            )
                            // Label
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if visible {
                                        rgb(0xc8cde0)
                                    } else {
                                        rgb(0x4a4f6e)
                                    })
                                    .child(label),
                            )
                    })
                    .collect::<Vec<_>>(),
            )
            // Close button
            .child(
                div()
                    .mt_1()
                    .border_t_1()
                    .border_color(rgb(0x2e3250))
                    .pt_1()
                    .child(
                        div()
                            .id("picker-close")
                            .text_xs()
                            .text_color(rgb(0x5c6380))
                            .text_center()
                            .py_1()
                            .rounded_md()
                            .cursor(CursorStyle::PointingHand)
                            .hover(|s| s.text_color(rgb(0x9ba3c0)).bg(rgb(0x252840)))
                            .child("Close")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.picker_open = false;
                                cx.notify();
                            })),
                    ),
            )
    }

    // ── Header Row ────────────────────────────────────────────────────────────

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let visible_indices = self.visible_indices();

        div()
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .bg(rgb(0x0f1119))
            .border_b_1()
            .border_color(rgb(0x1e2130))
            .font_weight(FontWeight::SEMIBOLD)
            .text_xs()
            .text_color(rgb(0x5c6380))
            .children(
                visible_indices
                    .into_iter()
                    .enumerate()
                    .map(|(visual_pos, real_idx)| {
                        let col = &self.columns[real_idx];
                        let width = col.width;
                        let label = col.col.label();

                        div()
                            .id(("col-header", real_idx))
                            .w(width)
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_1()
                            .pr_3()
                            // Drag handle glyph
                            .child(
                                div()
                                    .text_color(rgb(0x3a3f5c))
                                    .hover(|s| s.text_color(rgb(0x5865f2)))
                                    .cursor(CursorStyle::ClosedHand)
                                    .child("⠿"),
                            )
                            .child(
                                div()
                                    // .uppercase()
                                    // .tracking_wide()
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(label),
                            )
                            .on_drag(
                                DragColumn {
                                    from_index: visual_pos,
                                },
                                |drag, _point, _window, cx| cx.new(|_| drag.clone()),
                            )
                            .on_drop(cx.listener(move |this, dragged: &DragColumn, _window, cx| {
                                let vis = this.visible_indices();
                                let from_vis = dragged.from_index;
                                let to_vis = visual_pos;
                                if from_vis == to_vis {
                                    return;
                                }
                                // Map visual positions back to real indices, then reorder
                                if from_vis < vis.len() && to_vis < vis.len() {
                                    let from_real = vis[from_vis];
                                    let to_real = vis[to_vis];
                                    let col = this.columns.remove(from_real);
                                    this.columns.insert(to_real, col);
                                    cx.notify();
                                }
                            }))
                    })
                    .collect::<Vec<_>>(),
            )
    }

    // ── Data Row ──────────────────────────────────────────────────────────────

    fn render_row(
        item: &RxMessage,
        visible: &[(PacketColumn, Pixels)],
        row_index: usize,
    ) -> impl IntoElement {
        let is_even = row_index % 2 == 0;

        div()
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(rgb(0x191c28))
            .bg(if is_even {
                rgb(0x13151f)
            } else {
                rgb(0x111320)
            })
            .hover(|s| s.bg(rgb(0x1e2235)))
            .children(visible.iter().map(|(col, width)| {
                let content = match col {
                    PacketColumn::Hops => format!("{}", item.packet.header.hops),
                    PacketColumn::Interface => item.address.to_hex_string(),
                    PacketColumn::Destination => item.packet.destination.to_hex_string(),
                    PacketColumn::Transport => item
                        .packet
                        .transport
                        .map_or("—".into(), |a| a.to_hex_string()),
                    PacketColumn::Context => format!("{:?}", item.packet.context),
                    PacketColumn::DestinationType => {
                        format!("{:?}", item.packet.header.destination_type)
                    }
                    PacketColumn::HeaderType => format!("{:?}", item.packet.header.header_type),
                    PacketColumn::PropagationType => {
                        format!("{:?}", item.packet.header.propagation_type)
                    }
                    PacketColumn::IfacFlag => format!("{:?}", item.packet.header.ifac_flag),
                };

                let is_badge = col.is_badge();
                let is_hex = col.is_hex();

                div()
                    .w(*width)
                    .flex_shrink_0()
                    .pr_3()
                    .child(if is_badge {
                        // Enum fields → subtle pill badge
                        div()
                            // .inline_flex()
                            .items_center()
                            .px_2()
                            .py_px()
                            .rounded_full()
                            .bg(rgb(0x1e2235))
                            .border_1()
                            .border_color(rgb(0x2e3250))
                            .text_xs()
                            .text_color(rgb(0x8b93b8))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(content)
                            .into_any_element()
                    } else if is_hex {
                        // Hex addresses → accent colour, monospace
                        div()
                            .text_color(rgb(0x7b8fe0))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(content)
                            .into_any_element()
                    } else {
                        // Plain value (hops, etc.)
                        div()
                            .text_color(rgb(0xdde1f0))
                            .font_weight(FontWeight::MEDIUM)
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(content)
                            .into_any_element()
                    })
                    .into_any_element()
            }))
    }
}
