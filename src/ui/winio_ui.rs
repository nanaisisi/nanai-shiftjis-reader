#![cfg(feature = "winio-ui")]

use std::ops::Deref;

use winio::prelude::*;

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
    type Init<'a> = String;
    type Message = TextViewerPageMessage;

    async fn init(decoded_text: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Shift_JIS Viewer",
                size: Size::new(800.0, 600.0),
            },
            scroll: ScrollView = (&window) => {
                vscroll: true,
                hscroll: true,
            },
            label: Label = (&scroll) => {
                text: decoded_text,
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

pub fn ui(decoded_text: String) {
    let _ = App::new("c.nanaisisi.nanai-shiftjis-notepad")
        .and_then(|app| app.run::<TextViewerPage>(decoded_text));
}
