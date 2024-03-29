use eframe::egui::{
    self, Button, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Key,
    Label, Layout, Separator, TextStyle, TopBottomPanel, Window,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    sync::mpsc::{Receiver, SyncSender},
};

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const RED: Color32 = Color32::from_rgb(255, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
    pub refresh_news_data: bool,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
            refresh_news_data: Default::default(),
        }
    }
}
pub struct NewsCardData {
    pub title: String,
    pub description: String,
    pub link: String,
}
pub enum Msg {
    ApiKeySet(String),
}
pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>,
}

impl Headlines {
    pub fn new() -> Headlines {
        let config: HeadlinesConfig = confy::load("headlines").unwrap_or_default();
        Headlines {
            articles: vec![],
            api_key_initialized: !config.api_key.is_empty(),
            config,
            news_rx: None,
            app_tx: None,
        }
    }

    pub fn configure_font(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            Cow::Borrowed(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );
        font_def
            .family_and_size
            .insert(TextStyle::Heading, (FontFamily::Proportional, 35.));
        font_def
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 20.));
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());

        ctx.set_fonts(font_def);
    }

    pub fn render_news_card(&self, ui: &mut eframe::egui::Ui) {
        if self.config.refresh_news_data{
            ui.add_space(PADDING);
            if self.config.dark_mode {
                ui.colored_label(WHITE, String::from("Loading..."));
            } else {
                ui.colored_label(BLACK, String::from("Loading..."));
            }
            return 
        }


        for a in &self.articles {
            // render title
            ui.add_space(PADDING);
            let title = format!("▶ {}", a.title);

            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }

            // render desc
            ui.add_space(PADDING);
            let desc = Label::new(&a.description).text_style(TextStyle::Button);
            ui.add(desc);

            // render link
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }
            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add(Hyperlink::new(&a.link).text("Read More"))
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(egui::TextStyle::Heading));
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("❌").text_style(egui::TextStyle::Body));
                    if close_btn.clicked() {
                        frame.quit();
                    }
                    let refresh_btn = ui.add(Button::new("🔄").text_style(egui::TextStyle::Body));
                    if refresh_btn.clicked(){
                        self.config.refresh_news_data=true;
                    }
                    let theme_btn = ui.add(
                        Button::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    );
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });
            });
            ui.add_space(5.);
        });
    }
    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    if self.articles.len() < 10 {
                        self.articles.push(news_data);
                    }
                }
    
                Err(e) => tracing::warn!("Error receiving msg: {}", e),
            }
        }
    }
    pub fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("Configuration").show(ctx, |ui| {
            ui.label("Enter your API_KEY for newsdata.io");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);
            if text_input.lost_focus() && ui.input().key_down(Key::Enter) {
                if let Err(e) = confy::store(
                    "headlines",
                    HeadlinesConfig {
                        dark_mode: self.config.dark_mode,
                        api_key: self.config.api_key.to_string(),
                        refresh_news_data: self.config.refresh_news_data,
                    },
                ) {
                    tracing::error!("Failed saving app state: {}", e)
                }
                tracing::error!("api key set");
                self.api_key_initialized = true;
                if let Some(tx) = &self.app_tx {
                    tx.send(Msg::ApiKeySet(self.config.api_key.to_string()))
                        .expect("Failed sending ApiKeySet event");
                }
            }
            tracing::error!("{}", &self.config.api_key);
            ui.label("If you haven't registered for the API_KEY, head of to ");
            ui.add(Hyperlink::new("https://newsdata.io"));
        });
    }

}
