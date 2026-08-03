#![cfg(feature = "winio-ui")]

use std::ops::Deref;
use winio::prelude::*;
use crate::text_io::LoadedFile;

pub struct TextViewerPage {
    window: Child<Window>,
    scroll: Child<ScrollView>,
    label: Child<Label>,
}

#[derive(Debug)]
pub enum TextViewerPageEvent {}

#[derive(Debug)]
pub enum TextViewerPageMessage {
    Noop,
}

impl Component for TextViewerPage {
    type Error = Error;
    type Event = TextViewerPageEvent;
    type Init<'a> = LoadedFile;
    type Message = TextViewerPageMessage;

    async fn init(loaded_file: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let win_title = match &loaded_file.path {
            Some(p) => format!("Shift_JIS Notepad - {}", p.display()),
            None => "Shift_JIS Notepad".to_string(),
        };

        init! {
            window: Window = (()) => {
                text: win_title,
                size: Size::new(800.0, 600.0),
            },
            scroll: ScrollView = (&window) => {
                vscroll: true,
                hscroll: true,
            },
            label: Label = (&scroll) => {
                text: loaded_file.content,
                halign: HAlign::Left,
            },
        }

        window.show()?;

        Ok(Self {
            window,
            scroll,
            label,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: TextViewerPageMessage::Noop,
            self.window => {
                WindowEvent::Close => TextViewerPageMessage::Noop,
                WindowEvent::Move => TextViewerPageMessage::Noop,
                WindowEvent::Resize => TextViewerPageMessage::Noop,
                WindowEvent::ThemeChanged => TextViewerPageMessage::Noop,
            },
            self.scroll => {},
            self.label => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.scroll, self.label)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TextViewerPageMessage::Noop => Ok(false),
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        self.scroll.set_size(csize)?;
        self.label.set_size(self.label.preferred_size()?)?;
        Ok(())
    }
}

impl Deref for TextViewerPage {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

pub fn ui(loaded_file: LoadedFile) {
    if let Ok(app) = App::builder().name("c.nanaisisi.nanai-shiftjis-notepad").build() {
        let _ = app.block_on(TextViewerPage::run(loaded_file));
    }
}
