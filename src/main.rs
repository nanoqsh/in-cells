mod app;
mod sprite;

type App<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = dunge_winit::try_block_on(app::run) {
        eprintln!("error: {e}");
    }
}
