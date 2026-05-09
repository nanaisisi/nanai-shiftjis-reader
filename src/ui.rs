use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    point, prelude::*, px, rgb, size,
};

/// デコードされたテキストを保持するビューモデル
struct ReadText {
    text: SharedString,
}

impl Render for ReadText {
    /// テキストをスクロール可能な縦並びレイアウトで描画する
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p(px(16.0))
            .bg(rgb(0x505050))          // 背景色：ダークグレー
            .size(px(500.0))
            .justify_start()
            .items_start()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff)) // 枠線の色：青
            .text_xl()
            .text_color(rgb(0xffffff))   // 文字色：白
            .child(format!("{}!", self.text))
            .child(div().flex().gap_2())
            .id("vertical")
            .overflow_scroll()
    }
}

/// デコードされたテキストをGUIウィンドウで表示する。
/// ウィンドウは座標 (100, 100) に 500×500 ピクセルで表示される。
pub fn ui(decoded_text: String) {
    Application::new().run(|cx: &mut App| {
        // ウィンドウの初期位置とサイズを設定する
        let bounds = Bounds::new(point(px(100.), px(100.)), size(px(500.), px(500.0)));
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| ReadText {
                    text: decoded_text.into(),
                })
            },
        )
        .unwrap();
    });
}
