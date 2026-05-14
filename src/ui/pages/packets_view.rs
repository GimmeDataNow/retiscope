#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::ui::components::packets::FormattedPacket;
use crate::ui::components::packets::StateModel;

use reticulum::iface::RxMessage;

use gpui::prelude::FluentBuilder; // compiler complains without this
use gpui::*;
use gpui_component::Theme;
use gpui_component::theme::ActiveTheme;

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
    label: &'static str,
    width: Pixels,
}

impl Render for DragColumn {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors;
        div()
            .w(self.width)
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .bg(colors.table_head)
            .border_1()
            .border_color(colors.border)
            .shadow_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_xs()
            .text_color(colors.table_head_foreground)
            .child(div().text_color(colors.muted_foreground).child("⠿"))
            .child(
                div()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.label),
            )
    }
}

/// Drag state for resizing the inspector panel.
#[derive(Clone)]
struct InspectorResize {}

impl Render for InspectorResize {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

pub struct ColumnState {
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

    /// The packet currently selected for inspection.
    packet: Option<RxMessage>,
    /// Height of the inspector panel in pixels.
    inspector_height: Pixels,
    /// Mouse Y captured at the start of a resize drag.
    resize_drag_start: Option<(Pixels, Pixels)>, // (mouse_y, height_at_start)
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
            packet: None,
            inspector_height: px(200.),
            resize_drag_start: None,
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

        let has_packet = self.packet.is_some();
        let weak = cx.weak_entity();

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
                        let result = range
                            .map(|ix| {
                                let item = items[ix].clone();
                                let weak = weak.clone();
                                let raw = item.raw_packet.clone();
                                Self::render_row(&item, &visible, ix, theme, move |_, _, cx| {
                                    weak.update(cx, |this, cx| {
                                        this.packet = Some(raw.clone());
                                        cx.notify();
                                    })
                                    .ok();
                                })
                                .into_any_element()
                            })
                            .collect::<Vec<_>>();

                        result
                    }
                })
                .track_scroll(self.scroll_handle.clone())
                .flex_1(),
            )
            .when(self.picker_open, |el| {
                el.child(self.render_column_picker(cx))
            })
            .when(has_packet, |el| el.child(self.render_inspector(cx)))
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
                        let col_label = col.col.label();

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
                                    label: col_label,
                                    width: col.width,
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
        item: &FormattedPacket,
        visible: &[(PacketColumn, Pixels)],
        row_index: usize,
        theme: &Theme,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
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
            .w_full()
            .id(("packet-row", row_index))
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(border_color)
            .bg(row_bg)
            .hover(move |s| s.bg(hover_bg))
            .cursor(CursorStyle::PointingHand)
            .on_click(on_click)
            .children(visible.iter().map(|(col, width)| {
                let content = match col {
                    PacketColumn::Hops => item.hops.clone(),
                    PacketColumn::Interface => item.interface.clone(),
                    PacketColumn::Destination => item.destination.clone(),
                    PacketColumn::Transport => item.transport.clone(),
                    PacketColumn::Context => item.context.clone(),
                    PacketColumn::DestinationType => item.destination_type.clone(),
                    PacketColumn::HeaderType => item.header_type.clone(),
                    PacketColumn::PropagationType => item.propagation_type.clone(),
                    PacketColumn::IfacFlag => item.ifac_flag.clone(),
                };

                let is_badge = col.is_badge();
                let is_hex = col.is_hex();

                div().w(*width).pr_3().child(if is_badge {
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
                    div()
                        .text_color(theme.muted_foreground)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(content)
                } else {
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

    fn render_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors;
        let height = self.inspector_height;
        let address_str = self
            .packet
            .as_ref()
            .map(|p| p.address.to_hex_string())
            .unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h(height)
            .border_t_1()
            .border_color(colors.border)
            .child(
                // resize handle
                div()
                    .id("inspector-resize-handle")
                    .w_full()
                    .h(px(4.))
                    .bg(colors.border)
                    .cursor(CursorStyle::ResizeRow)
                    .hover(|s| s.bg(colors.ring))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, e: &MouseDownEvent, _window, cx| {
                            this.resize_drag_start = Some((e.position.y, this.inspector_height));
                            cx.notify();
                        }),
                    )
                    .on_drag(InspectorResize {}, |drag, _point, _window, cx| {
                        cx.new(|_| drag.clone())
                    })
                    .on_drag_move(cx.listener(
                        |this, e: &DragMoveEvent<InspectorResize>, _window, cx| {
                            if let Some((start_y, start_height)) = this.resize_drag_start {
                                let delta = e.event.position.y - start_y;
                                let new_height = (start_height - delta).max(px(80.)).min(px(600.));
                                this.inspector_height = new_height;
                                cx.notify();
                            }
                        },
                    ))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _e, _window, cx| {
                            this.resize_drag_start = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                // title bar
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_1()
                    .bg(colors.title_bar)
                    .border_b_1()
                    .border_color(colors.title_bar_border)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.muted_foreground)
                            .child("PACKET INSPECTOR"),
                    )
                    .child(
                        div()
                            .id("inspector-close")
                            .text_xs()
                            .text_color(colors.muted_foreground)
                            .cursor(CursorStyle::PointingHand)
                            .hover(|s| s.text_color(colors.foreground))
                            .child("✕")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.packet = None;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // packet
                div()
                    .flex_1()
                    .overflow_hidden()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_family(&theme.mono_font_family)
                    .text_color(colors.muted_foreground)
                    .child(address_str),
            )
    }
}
