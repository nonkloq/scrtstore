use gui::app::App;

fn main() {
    ratatui::run(|terminal| App::default().run(terminal))
        .unwrap_or_else(|e| eprintln!("Error in app: {e}"));
}
