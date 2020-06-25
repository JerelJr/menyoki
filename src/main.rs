#![allow(clippy::tabs_in_doc_comments)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate log;
mod app;
mod args;
mod gif;
mod image;
mod record;
mod settings;
mod test;
mod util;
mod x11;
use self::app::App;
use self::args::Args;
use self::settings::AppSettings;
use self::x11::WindowSystem;
use std::fs::File;
use std::io::Error;

fn main() -> Result<(), Error> {
	let args = Args::parse();
	util::init_logger().expect("Failed to initialize the logger");

	println!("thank god it's friday");

	let settings = AppSettings::new(&args);
	let mut window_system =
		WindowSystem::init(&settings).expect("Failed to access the window system");
	App::new(
		&settings,
		window_system
			.get_window()
			.expect("Failed to get the window"),
	)
	.start(
		File::create(&settings.save.file.name).expect("Failed to create the file"),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	#[should_panic(expected = "No frames found to save")]
	fn test_main() {
		main().unwrap();
	}
}
