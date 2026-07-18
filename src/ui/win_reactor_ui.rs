//! Sample for the `ScrollViewer` element.
#![cfg(feature = "win-reactor-ui")]
use windows_reactor::*;

fn app(_cx: &mut RenderCx) -> Element {
    let tall_body = vstack(
        (1..=30)
            .map(|i| text_block(format!("Line {i}")).font_size(13.0).into())
            .collect::<Vec<Element>>(),
    )
    .spacing(4.0);

    let wide_body = text_block(
        "This line is intentionally long so the ScrollViewer scrolls \
             horizontally to reveal the full content.",
    )
    .font_size(13.0);

    vstack((
        text_block("Default (vertical-only, auto)").bold(),
        scroll_viewer(tall_body).max_height(120.0),
        text_block("Both axes, always visible").bold(),
        scroll_viewer(wide_body)
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Visible)
            .vertical_scroll_bar_visibility(ScrollBarVisibility::Visible)
            .max_width(280.0)
            .max_height(80.0),
    ))
    .spacing(8.0)
    .into()
}

pub(crate) fn ui(decoded_text: String) {
    let application = Application::new().unwrap();
    let window = application
        .create_window()
        .title("ScrollViewer Sample")
        .size((400, 300))
        .build()
        .unwrap();

    // set the UI content built by the `app` function above
    let mut cx = RenderCx::new(std::rc::Rc::new(|| {}));
    window.set_content(app(&mut cx));
    window.show().unwrap();
}
