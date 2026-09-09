use crate::text_io::{LoadedFile, save_file_shiftjis};
use std::path::PathBuf;
use windows_reactor::*;

struct Notepad {
    path: Option<PathBuf>,
    text: String,
    status: Option<String>,
}

#[derive(Clone)]
enum Message {
    TextChanged(String),
    Save,
}

impl Component for Notepad {
    type Input = LoadedFile;
    type Message = Message;

    fn create(input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self {
            path: input.path.clone(),
            text: input.content.clone(),
            status: None,
        }
    }

    fn update(&mut self, message: Self::Message, _context: &ComponentContext<Self>) {
        match message {
            Message::TextChanged(text) => {
                self.text = text;
                self.status = Some("未保存の変更".to_string());
            }
            Message::Save => {
                self.status = Some(match self.path.as_deref() {
                    Some(path) => match save_file_shiftjis(path, &self.text) {
                        Ok(()) => "保存しました (Shift_JIS)".to_string(),
                        Err(error) => format!("保存エラー: {error}"),
                    },
                    None => "保存先がありません。ファイルを指定して起動してください".to_string(),
                });
            }
        }
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        let sender = context.sender();
        let title = self.path.as_ref().map_or_else(
            || "Nanai Shift_JIS Notepad".to_string(),
            |path| format!("Nanai Shift_JIS Notepad - {}", path.display()),
        );
        context.window_title(title);

        StackPanel::new().spacing(8.0).children((
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(8.0)
                .children((
                    TextBlock::new()
                        .text("Nanai Shift_JIS Notepad")
                        .font_size(20.0),
                    Button::new()
                        .on_click(sender.message(Message::Save))
                        .content("保存 (Shift_JIS)"),
                )),
            self.status
                .as_deref()
                .map(|status| TextBlock::new().text(status))
                .unwrap_or_else(|| TextBlock::new().text("Shift_JISとして編集・保存します")),
            RichEditBox::new()
                .text(self.text.clone())
                .on_text_changed(sender.callback(Message::TextChanged))
                .height(520.0)
                .vertical_alignment(VerticalAlignment::Stretch),
        ))
    }
}

pub(crate) fn ui(loaded_file: LoadedFile) {
    App::run_component::<Notepad>(loaded_file).expect("failed to start Windows Reactor UI");
}
