use std::process;

fn main() {
    if let Err(e) = electric_field_visualizer::run() {
        eprintln!("Exited with fatal error: {e:?}");
        process::exit(1);
    }
}
