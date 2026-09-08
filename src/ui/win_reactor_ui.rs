use crate::text_io::LoadedFile;
use windows_reactor::*;

struct Notepad;

impl Component for Notepad {
    type Input = LoadedFile;
    type Message = ();

    fn create(_input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn update(&mut self, _message: Self::Message, _context: &ComponentContext<Self>) {}

    fn view(&self, input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        let title = input.path.as_ref().map_or_else(
            || "Nanai Shift_JIS Notepad".to_string(),
            |path| format!("Nanai Shift_JIS Notepad - {}", path.display()),
        );
        context.window_title(title);

        StackPanel::new().spacing(8.0).children((
            TextBlock::new()
                .text("Nanai Shift_JIS Notepad")
                .font_size(20.0),
            ScrollViewer::new()
                .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                .content(TextBlock::new().text(input.content.clone()).font_size(14.0)),
        ))
    }
}

pub(crate) fn ui(loaded_file: LoadedFile) {
    App::run_component::<Notepad>(loaded_file).expect("failed to start Windows Reactor UI");
}
