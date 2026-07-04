use self::gpui_ui;
use self::win_reactor_ui;
use self::winio_ui;

pub fn ui(decoded_text: String) {
    cfg_select! {
        cfg(feature = "win-reactor-ui") => {
            win_reactor_ui::ui(decoded_text);
        }
        cfg(feature = "winio-ui") => {
            winio_ui::ui(decoded_text);
        }
        cfg(feature = "gpui-ui") => {
            gpui_ui::ui(decoded_text);
        }
    }
}
