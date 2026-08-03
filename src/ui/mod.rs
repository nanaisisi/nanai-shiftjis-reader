#[cfg(any(
    all(feature = "win-reactor-ui", feature = "winio-ui"),
    all(feature = "win-reactor-ui", feature = "gpui-ui"),
    all(feature = "winio-ui", feature = "gpui-ui")
))]
compile_error!(
    "Choose only one UI feature at a time. Use --no-default-features with winio-ui or gpui-ui when you want a non-default UI."
);

#[cfg(feature = "win-reactor-ui")]
mod win_reactor_ui;

#[cfg(feature = "winio-ui")]
mod winio_ui;

#[cfg(feature = "gpui-ui")]
mod gpui_ui;

use crate::text_io::LoadedFile;

pub fn ui(loaded_file: LoadedFile) {
    #[cfg(feature = "win-reactor-ui")]
    {
        let _ = win_reactor_ui::ui(loaded_file);
    }

    #[cfg(all(not(feature = "win-reactor-ui"), feature = "winio-ui"))]
    {
        winio_ui::ui(loaded_file);
    }

    #[cfg(all(
        not(feature = "win-reactor-ui"),
        not(feature = "winio-ui"),
        feature = "gpui-ui"
    ))]
    {
        gpui_ui::ui(loaded_file);
    }
}
