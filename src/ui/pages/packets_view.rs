use crate::ui::components::packets::StateModel;
use gpui::prelude::FluentBuilder; // compiler complains without this
use gpui::*;
use gpui_component::Theme;
use gpui_component::theme::ActiveTheme;
use reticulum::iface::RxMessage;

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

    /// Enum fields get a subtle badge treatment.
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

    /// Hex address columns use the accent colour.
    fn is_hex(&self) -> bool {
        matches!(self, Self::Destination | Self::Interface | Self::Transport)
    }
}

#[derive(Clone)]
struct DragColumn {
    from_index: usize,
}

impl Render for DragColumn {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .px_3()
            .py_1()
            .rounded(theme.radius)
            .bg(theme.popover)
            .border_1()
            .border_color(theme.drag_border)
            .shadow_lg()
            .text_sm()
            .text_color(theme.popover_foreground)
            .child("Moving column…")
    }
}

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

impl Render for PacketsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = cx.global::<StateModel>().inner.clone();
        let count = state_handle.read(cx).items.len();

        // map entries into content + width
        let visible: Vec<(PacketColumn, Pixels)> = self
            .visible_indices()
            .into_iter()
            .map(|i| (self.columns[i].col, self.columns[i].width))
            .collect();

        let theme = cx.theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .text_sm()
            .font_family(&theme.mono_font_family)
            .child(self.render_toolbar(cx))
            .child(self.render_header(cx))
            .child(
                uniform_list("packet-list", count, {
                    move |range, _window, cx| {
                        let items = &state_handle.read(cx).items;
                        let theme = cx.theme();
                        range
                            .map(|ix| {
                                Self::render_row(&items[ix], &visible, ix, theme).into_any_element()
                            })
                            .collect::<Vec<_>>()
                    }
                })
                .track_scroll(self.scroll_handle.clone())
                .flex_1(),
            )
            .when(self.picker_open, |el| {
                el.child(self.render_column_picker(cx))
            })
    }
}

impl PacketsPage {
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.colors.title_bar_border)
            .bg(theme.colors.title_bar)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.colors.muted_foreground)
                    .child("PACKET MONITOR"),
            )
            // .child() // packet viewer here. if you take the child from below and just use .absolute() then you can observe some of the desired behaviour
            .child(
                div()
                    .id("col-picker-btn")
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .rounded(theme.radius)
                    .cursor(CursorStyle::PointingHand)
                    .bg(if self.picker_open {
                        theme.colors.accent
                    } else {
                        theme.colors.secondary
                    })
                    .border_1()
                    .border_color(if self.picker_open {
                        theme.colors.ring
                    } else {
                        theme.colors.border
                    })
                    .hover(move |s| s.bg(theme.colors.secondary_hover))
                    .text_xs()
                    .text_color(if self.picker_open {
                        theme.colors.accent_foreground
                    } else {
                        theme.colors.secondary_foreground
                    })
                    .child("⊞ Columns")
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.picker_open = !this.picker_open;
                        cx.notify();
                    })),
            )
    }

    fn render_column_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let radius = cx.theme().radius;

        div()
            .absolute()
            .top(px(68.))
            .right(px(8.))
            .w(px(200.))
            .rounded(radius)
            .bg(colors.popover)
            .border_1()
            .border_color(colors.border)
            .shadow_xl()
            .p_2()
            .flex()
            .flex_col()
            .gap_px()
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.muted_foreground)
                    .child("TOGGLE COLUMNS"),
            )
            .children(self.columns.iter().enumerate().map(|(idx, col_state)| {
                let visible = col_state.visible;
                let label = col_state.col.label();

                let check_bg = if visible {
                    colors.primary
                } else {
                    colors.muted
                };
                let check_border = if visible {
                    colors.primary
                } else {
                    colors.border
                };
                let check_fg = colors.primary_foreground;
                let label_color = if visible {
                    colors.popover_foreground
                } else {
                    colors.muted_foreground
                };
                let hover_bg = colors.list_hover;

                div()
                    .id(("picker-row", idx))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded(radius)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        let visible_count = this.columns.iter().filter(|c| c.visible).count();
                        if this.columns[idx].visible && visible_count <= 1 {
                            return;
                        }
                        this.columns[idx].visible = !&this.columns[idx].visible;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(14.))
                            .h(px(14.))
                            .rounded_sm()
                            .border_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_color(check_border)
                            .bg(check_bg)
                            .when(visible, |el| el.text_color(check_fg).text_xs().child("✓")),
                    )
                    .child(div().text_sm().text_color(label_color).child(label))
            }))
            .child(
                div()
                    .mt_1()
                    .border_t_1()
                    .border_color(colors.border)
                    .pt_1()
                    .child(
                        div()
                            .id("picker-close")
                            .text_xs()
                            .text_color(colors.muted_foreground)
                            .text_center()
                            .py_1()
                            .rounded(radius)
                            .cursor(CursorStyle::PointingHand)
                            .hover(move |s| s.text_color(colors.foreground).bg(colors.list_hover))
                            .child("Close")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.picker_open = false;
                                cx.notify();
                            })),
                    ),
            )
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors;
        let visible_indices = self.visible_indices();

        div()
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .bg(colors.table_head)
            .border_b_1()
            .border_color(colors.border)
            .font_weight(FontWeight::SEMIBOLD)
            .text_xs()
            .text_color(colors.table_head_foreground)
            .children(
                visible_indices
                    .into_iter()
                    .enumerate()
                    .map(|(visual_pos, real_idx)| {
                        let col = &self.columns[real_idx];

                        div()
                            .id(("col-header", real_idx))
                            .w(col.width)
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_1()
                            .pr_3()
                            .child(
                                div()
                                    .text_color(colors.muted_foreground)
                                    .hover(move |s| s.text_color(colors.accent))
                                    .cursor(CursorStyle::ClosedHand)
                                    .child("⠿"),
                            )
                            .child(
                                div()
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(col.col.label()),
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
                                // let visual_pos = visual_pos;
                                if from_vis == visual_pos {
                                    return;
                                }
                                if dragged.from_index < vis.len() && visual_pos < vis.len() {
                                    let from_real = vis[dragged.from_index];
                                    let to_real = vis[visual_pos];
                                    let col = this.columns.remove(from_real);
                                    this.columns.insert(to_real, col);
                                    cx.notify();
                                }
                            }))
                    }),
            )
    }

    fn render_row(
        item: &RxMessage,
        visible: &[(PacketColumn, Pixels)],
        row_index: usize,
        theme: &Theme,
    ) -> impl IntoElement {
        let is_even = row_index % 2 == 0;
        let row_bg = if is_even {
            theme.background
        } else {
            theme.table_even
        };
        let hover_bg = theme.table_hover;
        let border_color = theme.table_row_border;

        div()
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(border_color)
            .bg(row_bg)
            .hover(move |s| s.bg(hover_bg))
            .children(visible.iter().map(|(col, width)| {
                // it might be good to extract this into a function (maybe)
                let content = match col {
                    PacketColumn::Hops => format!("{}", item.packet.header.hops),
                    PacketColumn::Interface => item.address.to_hex_string(),
                    PacketColumn::Destination => item.packet.destination.to_hex_string(),
                    PacketColumn::Transport => item
                        .packet
                        .transport
                        .map_or("—".into(), |a| a.to_hex_string()), // em dash looks much better than the minus
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

                div().w(*width).flex_shrink_0().pr_3().child(if is_badge {
                    // Enum fields → pill badge
                    div()
                        .items_center()
                        .px_2()
                        .py_px()
                        .rounded(theme.radius)
                        .bg(theme.muted)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(content)
                } else if is_hex {
                    // Hex addresses → accent colour (maybe muted_fg is better)
                    div()
                        .text_color(theme.muted_foreground)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(content)
                } else {
                    // Plain values
                    div()
                        .text_color(theme.muted_foreground)
                        .font_weight(FontWeight::MEDIUM)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(content)
                        .text_right()
                })
            }))
    }
}
