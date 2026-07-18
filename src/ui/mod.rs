cfg_select! {
    feature = "win-reactor-ui" => {
        mod win_reactor_ui;
        use crate::ui::win_reactor_ui;
    },
    feature = "winio-ui" => {
        mod winio_ui;
        use crate::ui::winio_ui;
    },
    feature = "gpui-ui" => {
        mod gpui_ui;
        use crate::ui::gpui_ui;
    }
}

pub fn ui(decoded_text: String) {
    cfg_select! {
        feature = "win-reactor-ui" => {
            let _ = win_reactor_ui::ui(decoded_text);
        },
        feature = "winio-ui" => {
            winio_ui::ui(decoded_text);
        },
        feature = "gpui-ui" => {
            gpui_ui::ui(decoded_text);
        }
    }
}
