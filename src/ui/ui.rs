pub fn ui(decoded_text: String) {
    #[cfg(feature = "win-reactor-ui")]
    {
        super::win_reactor_ui::ui(decoded_text);
    }
    #[cfg(feature = "winio-ui")]
    {
        winio_ui::ui(decoded_text);
    }
    #[cfg(feature = "gpui-ui")]
    {
        gpui_ui::ui(decoded_text);
    }
}
