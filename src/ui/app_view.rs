#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use gpui::*;
use gpui_component::Icon;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::sidebar::*;
use gpui_component::theme::Theme;
use gpui_component::*;

use std::collections::HashMap;

use crate::daemon::StreamBundle;
use crate::ui::components::packets::StateModel;
use crate::ui::pages::dashboard::Dashboard;
use crate::ui::pages::packets_view::PacketsPage;

use crate::ui::pages::settings::Settings;

// pages
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Dashboard,
    Settings,
    Packets,
}

// view
pub struct AppView {
    pub active_page: PageId,
    pub pages: HashMap<PageId, AnyView>,
    pub sidebar_collapsed: bool,
}

impl AppView {
    // build all pages
    pub fn build(cx: &mut Context<'_, Self>, stream: StreamBundle) -> Self {
        let mut pages: HashMap<PageId, AnyView> = HashMap::new();

        StateModel::init(cx, stream.raw_interface_packets);

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
        // pages.insert(PageId::Packets, PacketsPage.into());
        pages.insert(PageId::Packets, cx.new(|_cx| PacketsPage::new()).into());

        Self {
            sidebar_collapsed: false,
            active_page: PageId::Dashboard,
            pages,
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
            .icon(if cx.theme().is_dark() {
                Icon::empty().path("icons/sun.svg")
            } else {
                Icon::empty().path("icons/moon.svg")
            })
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
                                .icon(Icon::empty().path("icons/radar.svg"))
                                .active(active_id == PageId::Packets)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.active_page = PageId::Packets;
                                    cx.notify();
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("Settings")
                                .icon(Icon::empty().path("icons/settings.svg"))
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
                .child(current_page),
        )
    }
}
