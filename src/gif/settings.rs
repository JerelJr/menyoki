use crate::args::parser::ArgParser;
use crate::util::file::File;
use std::path::Path;

/* GIF and frame settings */
#[derive(Clone, Copy, Debug)]
pub struct GifSettings {
	pub repeat: i32,
	pub quality: u8,
	pub speed: f32,
	pub fast: bool,
}

/* Default initialization values for GifSettings */
impl Default for GifSettings {
	fn default() -> Self {
		Self {
			repeat: -1,
			quality: 75,
			speed: 1.,
			fast: false,
		}
	}
}

impl GifSettings {
	/**
	 * Create a new GifSettings object.
	 *
	 * @param  repeat
	 * @param  quality
	 * @param  speed
	 * @param  fast
	 * @return GifSettings
	 */
	pub fn new(repeat: i32, quality: u8, speed: f32, fast: bool) -> Self {
		Self {
			repeat,
			quality,
			speed,
			fast,
		}
	}

	/**
	 * Create a GifSettings object from parsed arguments.
	 *
	 * @param  parser
	 * @return GifSettings
	 */
	pub fn from_args(parser: ArgParser<'_>) -> Self {
		match parser.args {
			Some(matches) => Self::new(
				parser.parse("repeat", Self::default().repeat) - 1,
				parser.parse("quality", Self::default().quality),
				parser.parse("speed", Self::default().speed),
				matches.is_present("fast"),
			),
			None => Self::default(),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct SplitSettings<'a> {
	pub file: &'a Path,
	pub dir: &'a Path,
}

/* Default initialization values for GifSettings */
impl Default for SplitSettings<'_> {
	fn default() -> Self {
		Self {
			file: Path::new(""),
			dir: Path::new(""),
		}
	}
}

impl<'a> SplitSettings<'a> {
	/**
	 * Create a new SplitSettings object.
	 *
	 * @param  file
	 * @param  dir
	 * @return SplitSettings
	 */
	pub fn new(file: &'a Path, dir: &'a Path) -> Self {
		Self { file, dir }
	}

	/**
	 * Create a SplitSettings object from parsed arguments.
	 *
	 * @param  parser
	 * @return SplitSettings
	 */
	pub fn from_args(parser: ArgParser<'a>) -> Self {
		match parser.args {
			Some(matches) => {
				let file = Path::new(matches.value_of("file").unwrap_or_default());
				let dir = match matches.value_of("dir") {
					Some(dir) => Path::new(dir),
					None => Box::leak(
						File::get_default_path(&format!(
							"{}_frames",
							file.file_stem()
								.unwrap_or_default()
								.to_str()
								.unwrap_or_default(),
						))
						.into_boxed_path(),
					),
				};
				Self::new(file, dir)
			}
			None => Self::default(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::{App, Arg};
	#[test]
	fn test_gif_settings() {
		let args = App::new("test")
			.arg(Arg::with_name("repeat").long("repeat").takes_value(true))
			.arg(Arg::with_name("quality").long("quality").takes_value(true))
			.arg(Arg::with_name("speed").long("speed").takes_value(true))
			.arg(Arg::with_name("fast").long("fast"))
			.get_matches_from(vec![
				"test",
				"--repeat",
				"5",
				"--quality",
				"10",
				"--speed",
				"1.1",
				"--fast",
			]);
		let gif_settings = GifSettings::from_args(ArgParser::new(Some(&args)));
		assert_eq!(4, gif_settings.repeat);
		assert_eq!(10, gif_settings.quality);
		assert_eq!(1.1, gif_settings.speed);
		assert_eq!(true, gif_settings.fast);
		let gif_settings = GifSettings::from_args(ArgParser::new(None));
		assert_eq!(-1, gif_settings.repeat);
		assert_eq!(75, gif_settings.quality);
		assert_eq!(1.0, gif_settings.speed);
		assert_eq!(false, gif_settings.fast);
	}
}
