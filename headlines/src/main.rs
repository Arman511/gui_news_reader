mod headlines;
use std::{
    sync::mpsc::{channel, sync_channel},
    thread,
};

use eframe::{
    egui::{
        self, CentralPanel, CtxRef, Hyperlink, Label, ScrollArea, Separator, TextStyle,
        TopBottomPanel, Ui, Vec2,
    },
    epi::App,
    run_native, NativeOptions,
};
use headlines::{Headlines, Msg, NewsCardData, PADDING};
use newsapi::NewsAPI;

impl App for Headlines {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        let api_key = self.config.api_key.to_string();
        let _refresh_news_data = self.config.refresh_news_data;

        let (mut news_tx, news_rx) = channel();
        let (app_tx, app_rx) = sync_channel(1);
        self.app_tx = Some(app_tx);
        self.news_rx = Some(news_rx);
        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx)
            } else {
                loop {
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => fetch_news(&api_key, &mut news_tx),
                        Err(e) => tracing::error!("failed receiving api key: {}", e),
                    }
                }
            }
        });
        self.configure_font(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        ctx.request_repaint();
        if self.config.refresh_news_data{
            self.config.refresh_news_data = false;
            let (mut news_tx, _news_rx) = channel();
            fetch_news(&self.config.api_key.to_string(), &mut news_tx);
        }
        if self.config.dark_mode {
            ctx.set_visuals(egui::Visuals::dark())
        } else {
            ctx.set_visuals(egui::Visuals::light())
        }
        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.preload_articles();

            self.render_top_panel(ctx, frame);

            CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::auto_sized().show(ui, |ui| {
                    self.render_news_card(ui);
                });
                render_footer(ctx);
            });
        }
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn fetch_news(api_key: &str, news_tx: &mut std::sync::mpsc::Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch() {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.get_title().to_string(),
                link: a.get_link().to_string(),
                description: a
                    .get_description()
                    .map(|s| s.to_string())
                    .unwrap_or("...".to_string()),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error of sending data: {}", e)
            };
        }
    }
}

fn render_footer(ctx: &CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API Source: newsdata.io").monospace());
            ui.add(
                Hyperlink::new("https.github.com/emilk/equi")
                    .text("Made with equi")
                    .text_style(TextStyle::Monospace),
            );
            ui.add(
                Hyperlink::new(
                    "https://github.com/Arman511/LearningRust/tree/main/gui_news_reader/headlines",
                )
                .text("Arman511/headlines")
                .text_style(TextStyle::Monospace),
            );
            ui.add_space(5.);
        })
    });
}
fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Headlines");
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}

fn main() {
    tracing_subscriber::fmt::init();
    let app = Headlines::new();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(360., 640.));

    run_native(Box::new(app), win_option);
}
