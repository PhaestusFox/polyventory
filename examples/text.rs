use std::io::Write;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, print_console);
    app.add_systems(Last, sleep);
    app.run();
}

fn print_console(
    time: Res<Time>,
) {
    println!("\x1B[2J");
    _ = std::io::stdout().flush();
    print!("-------\n|{:5.2}|\n|{1:5}|\n|{1:5}|\n|{1:5}|\n-------\n", time.elapsed().as_secs_f32(), ' ');
    _ = std::io::stdout().flush();
}

fn sleep(
    time: Res<Time>,
) {
    let elapsed = time.elapsed().as_millis();
    let next = 100 - (elapsed % 100);
    std::thread::sleep(std::time::Duration::from_millis(next as u64));
}