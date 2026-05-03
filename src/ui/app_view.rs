use gpui::*;
use gpui_component::Icon;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::sidebar::*;
use gpui_component::theme::Theme;
use gpui_component::*;

use reticulum::iface::RxMessage;

use std::collections::HashMap;

use crate::daemon::StreamBundle;
use crate::ui::pages::dashboard::Dashboard;
use crate::ui::pages::packets::PacketsPage;
use crate::ui::pages::settings::Settings;

// pages
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Dashboard,
    Settings,
    Packets,
}

// pub struct AppState

// view
pub struct AppView {
    pub active_page: PageId,
    pub pages: HashMap<PageId, AnyView>,
    pub sidebar_collapsed: bool,
    pub packets: Vec<RxMessage>,
}

impl AppView {
    // build all pages
    pub fn build(cx: &mut Context<'_, Self>, mut stream: StreamBundle) -> Self {
        let mut pages: HashMap<PageId, AnyView> = HashMap::new();
        // Dashboard
        pages.insert(
            PageId::Dashboard,
            cx.new(|_| Dashboard {
                title: "Dashboard".into(),
            })
            .into(),
        );

        // Settings
        pages.insert(PageId::Settings, cx.new(|_| Settings { count: 0 }).into());

        let weak = cx.weak_entity();
        pages.insert(
            PageId::Packets,
            cx.new(|_| PacketsPage::new(weak.upgrade().unwrap())).into(),
        );

        let mut rx = stream.raw_interface_packets.subscribe();

        cx.spawn(async move |this, cx| {
            // while let Ok(msg) = rx.recv().await {
            //     let update_result = this.update(cx, |view, cx| {
            //         view.packets.push(msg);
            //         cx.notify();
            //     });

            //     if update_result.is_err() {
            //         break;
            //     }
            // }
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        this.update(cx, |view, cx| {
                            view.packets.push(msg);
                            eprintln!("total packets: {}", view.packets.len());
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("LAGGED: dropped {} packets", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        eprintln!("CHANNEL CLOSED");
                        break;
                    }
                }
            }
        })
        .detach();

        Self {
            sidebar_collapsed: false,
            active_page: PageId::Dashboard,
            pages,
            packets: Vec::new(),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = self.sidebar_collapsed;
        let active_id = self.active_page;
        let current_page = self
            .pages
            .get(&active_id)
            .cloned()
            .expect("Page not found in HashMap");

        let toggle_button = Button::new("toggle")
            .ghost()
            .small()
            .icon(
                Icon::empty()
                    .path("icons/telescope.svg")
                    .text_color(cx.theme().foreground),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.sidebar_collapsed = !this.sidebar_collapsed;
                cx.notify();
            }));
        let theme_button = Button::new("theme")
            .ghost()
            .small()
            .icon(Icon::empty().path("icons/telescope.svg"))
            .on_click(cx.listener(move |_, _, _, cx| {
                let current = cx.theme().mode;
                let next = if current.is_dark() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                };
                Theme::change(next, None, cx);
                cx.notify();
            }));

        let sidebar = Sidebar::left()
            .collapsed(collapsed)
            .header(
                SidebarHeader::new()
                    .child(h_flex().gap_2().child(toggle_button).child("Retiscope")),
            )
            .child(
                SidebarGroup::new("Navigation").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new("Dashboard")
                                .icon(Icon::empty().path("icons/telescope.svg"))
                                .active(active_id == PageId::Dashboard)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.active_page = PageId::Dashboard;
                                    cx.notify();
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("Packets")
                                .icon(Icon::empty().path("icons/telescope.svg"))
                                .active(active_id == PageId::Packets)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.active_page = PageId::Packets;
                                    cx.notify();
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("Settings")
                                .icon(Icon::empty().path("icons/telescope.svg"))
                                .active(active_id == PageId::Settings)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.active_page = PageId::Settings;
                                    cx.notify();
                                })),
                        ),
                ),
            )
            .footer(SidebarFooter::new().child(theme_button));

        h_flex().size_full().child(sidebar).child(
            div()
                .flex_1()
                .h_full()
                .bg(cx.theme().background)
                // .child(
                //     div()
                //         .flex()
                //         .flex_col()
                //         .size_full()
                //         .overflow_y_scrollbar()
                //         .children(self.packets.iter().enumerate().map(|(i, pkt)| {
                //             div().px_3().py_1().text_sm().child(format!(
                //                 "destination {}",
                //                 pkt.packet.destination.to_hex_string()
                //             ))
                //         })),
                // )
                .child(current_page),
        )
    }
}
