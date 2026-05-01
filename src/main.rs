use rust_embed::RustEmbed;
// use ThemeMode;
use gpui::*;
use gpui_component::Icon;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::sidebar::*;
use gpui_component::theme::Theme;
use gpui_component::*;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| SharedString::from(p.to_string()))
            .collect())
    }
}

struct AppView {
    sidebar_collapsed: bool,
}
impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = self.sidebar_collapsed;

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
                                .icon(Icon::empty().path("icons/telescope.svg")),
                        )
                        .child(
                            SidebarMenuItem::new("Settings")
                                .icon(Icon::empty().path("icons/telescope.svg")),
                        ),
                ),
            )
            .footer(SidebarFooter::new().child(theme_button));

        h_flex().size_full().child(sidebar).child(
            div()
                .flex_1()
                .h_full()
                .bg(cx.theme().background)
                .child("Main Content Area"),
        )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| AppView {
                    sidebar_collapsed: false,
                });
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
