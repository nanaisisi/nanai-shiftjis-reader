#![cfg(feature = "gpui-ui")]
use std::path::PathBuf;
use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div,
    point, prelude::*, px, rgb, size,
};
use crate::text_io::{save_file_shiftjis, LoadedFile};

/// デコードされたテキストとファイルパスを保持するビューモデル
struct NotepadModel {
    path: Option<PathBuf>,
    text: String,
    status_message: Option<String>,
}

impl NotepadModel {
    fn save(&mut self) {
        if let Some(ref path) = self.path {
            match save_file_shiftjis(path, &self.text) {
                Ok(_) => self.status_message = Some("保存しました (Shift_JIS)".into()),
                Err(err) => self.status_message = Some(format!("保存エラー: {err}")),
            }
        } else {
            self.status_message = Some("ファイルパスが指定されていません".into());
        }
    }
}

impl Render for NotepadModel {
    /// メモ帳画面の描画
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = match &self.path {
            Some(p) => format!("Nanai Shift_JIS Notepad - {}", p.display()),
            None => "Nanai Shift_JIS Notepad (新規ファイル)".to_string(),
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p(px(16.0))
            .bg(rgb(0x282c34)) // ダークテーマ背景
            .size(px(600.0))
            .justify_start()
            .items_start()
            .shadow_lg()
            .text_color(rgb(0xabb2bf))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .w_full()
                    .child(div().font_weight(gpui::FontWeight::BOLD).text_xl().child(title))
                    .child(
                        div()
                            .p(px(6.0))
                            .bg(rgb(0x4b5263))
                            .text_color(rgb(0xffffff))
                            .rounded_md()
                            .cursor_pointer()
                            .child("保存 (Shift_JIS)")
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.save();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                if let Some(msg) = &self.status_message {
                    div().text_sm().text_color(rgb(0x98c379)).child(msg.clone())
                } else {
                    div()
                }
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .h_full()
                    .p(px(8.0))
                    .bg(rgb(0x1e2127))
                    .border_1()
                    .border_color(rgb(0xabb2bf))
                    .id("text-content")
                    .overflow_scroll()
                    .child(self.text.clone()),
            )
    }
}

/// デコードされたテキストをGUIウィンドウで表示・編集する。
pub fn ui(loaded_file: LoadedFile) {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::new(point(px(100.), px(100.)), size(px(600.), px(600.0)));
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                show: true,
                ..Default::default()
            },
            |window, cx| {
                let window_title = match &loaded_file.path {
                    Some(p) => format!("Shift_JIS Notepad - {}", p.display()),
                    None => "Shift_JIS Notepad".to_string(),
                };
                window.set_window_title(&window_title);
                cx.new(|_| NotepadModel {
                    path: loaded_file.path,
                    text: loaded_file.content,
                    status_message: None,
                })
            },
        )
        .unwrap();
    });
}
