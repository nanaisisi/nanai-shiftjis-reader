#[cfg(feature = "win-reactor-ui")]
mod win_reactor_ui;

#[cfg(feature = "gpui-ui")]
mod gpui_ui;

use crate::text_io::LoadedFile;

pub fn ui(loaded_file: LoadedFile) {
    #[cfg(feature = "win-reactor-ui")]
    {
        win_reactor_ui::ui(loaded_file);
    }

    #[cfg(all(not(feature = "win-reactor-ui"), feature = "gpui-ui"))]
    {
        gpui_ui::ui(loaded_file);
    }
}
