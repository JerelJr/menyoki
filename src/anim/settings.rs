use crate::args::matches::ArgMatches;
use crate::args::parser::ArgParser;
use crate::file::format::FileFormat;
use crate::file::File;
use std::fs;
use std::path::PathBuf;

/* Animation and frame settings */
#[derive(Debug)]
pub struct AnimSettings {
	pub fps: u32,
	pub repeat: i32,
	pub quality: u8,
	pub speed: f32,
	pub cut: (f32, f32),
	pub frames: Vec<PathBuf>,
	pub gifski: (bool, bool),
}

/* Default initialization values for AnimSettings */
impl Default for AnimSettings {
	fn default() -> Self {
		Self {
			fps: 20,
			repeat: -1,
			quality: 75,
			speed: 1.,
			cut: (0., 0.),
			frames: Vec::new(),
			gifski: (false, false),
		}
	}
}

impl AnimSettings {
	/**
	 * Create a new AnimSettings object.
	 *
	 * @param  fps
	 * @param  repeat
	 * @param  quality
	 * @param  speed
	 * @param  cut
	 * @param  frames
	 * @param  gifski
	 * @return AnimSettings
	 */
	pub fn new(
		fps: u32,
		repeat: i32,
		quality: u8,
		speed: f32,
		cut: (f32, f32),
		frames: Vec<PathBuf>,
		gifski: (bool, bool),
	) -> Self {
		Self {
			fps,
			repeat,
			quality,
			speed,
			cut,
			frames,
			gifski,
		}
	}

	/**
	 * Create a new AnimSettings object from arguments.
	 *
	 * @param  matches
	 * @param  file_format
	 * @return AnimSettings
	 */
	pub fn from_args(matches: &ArgMatches<'_>, file_format: &FileFormat) -> Self {
		let format = &file_format.to_string().to_lowercase();
		Self::from_parser(ArgParser::from_subcommand(
			matches,
			if matches.is_present("make") {
				"make"
			} else {
				format
			},
		))
	}

	/**
	 * Create a AnimSettings object from an argument parser.
	 *
	 * @param  parser
	 * @return AnimSettings
	 */
	fn from_parser(parser: ArgParser<'_>) -> Self {
		match parser.args {
			Some(ref matches) => Self::new(
				match parser.parse("fps", Self::default().fps) {
					fps if fps > 0 => fps,
					_ => Self::default().fps,
				},
				parser.parse("repeat", Self::default().repeat) - 1,
				parser.parse("quality", Self::default().quality),
				parser.parse("speed", Self::default().speed),
				(
					parser.parse("cut-beginning", Self::default().cut.0) * 1000.,
					parser.parse("cut-end", Self::default().cut.1) * 1000.,
				),
				Self::get_frames(matches),
				(
					matches.is_present("gifski") || matches.is_present("fast"),
					matches.is_present("fast"),
				),
			),
			None => Self::default(),
		}
	}

	/**
	 * Get the frame files from parsed arguments.
	 *
	 * @param  args
	 * @return Vector of PathBuf
	 */
	fn get_frames(args: &ArgMatches<'_>) -> Vec<PathBuf> {
		let mut values = if let Some(dir) = args.value_of("dir") {
			let dir = shellexpand::full(dir)
				.map(|s| s.to_string())
				.unwrap_or(dir.to_string());
			fs::read_dir(dir)
				.expect("Could not read files from directory")
				.map(|entry| {
					entry
						.expect("Failed to get directory entry")
						.path()
						.into_os_string()
						.into_string()
						.unwrap_or_default()
				})
				.collect()
		} else if let Some(values) = args.values_of("frames") {
			values.map(String::from).collect()
		} else {
			Vec::new()
		};
		if !args.is_present("no-sort") {
			values.sort_by(|a, b| natord::compare(a, b));
		}
		values.into_iter().map(PathBuf::from).collect()
	}

	/**
	 * Map the given number from a range to another range.
	 *
	 * @param  value
	 * @param  from_range
	 * @param  to_range
	 * @return f64
	 */
	pub fn map_range(
		&self,
		value: f64,
		from_range: (f64, f64),
		to_range: (f64, f64),
	) -> f64 {
		to_range.0
			+ (value - from_range.0) * (to_range.1 - to_range.0)
				/ (from_range.1 - from_range.0)
	}
}

/* Animation split settings */
#[derive(Debug)]
pub struct SplitSettings {
	pub file: PathBuf,
	pub dir: PathBuf,
}

/* Default initialization values for SplitSettings */
impl Default for SplitSettings {
	fn default() -> Self {
		Self {
			file: PathBuf::new(),
			dir: PathBuf::new(),
		}
	}
}

impl SplitSettings {
	/**
	 * Create a new SplitSettings object.
	 *
	 * @param  file
	 * @param  dir
	 * @return SplitSettings
	 */
	pub fn new(file: PathBuf, dir: PathBuf) -> Self {
		Self { file, dir }
	}

	/**
	 * Create a new SplitSettings object from arguments.
	 *
	 * @param  matches
	 * @return SplitSettings
	 */
	pub fn from_args(matches: &ArgMatches<'_>) -> Self {
		Self::from_parser(ArgParser::from_subcommand(matches, "split"))
	}

	/**
	 * Create a SplitSettings object from an argument parser.
	 *
	 * @param  parser
	 * @return SplitSettings
	 */
	fn from_parser(parser: ArgParser<'_>) -> Self {
		match parser.args {
			Some(matches) => {
				let file = matches.value_of("file").unwrap_or_default();
				let file = shellexpand::full(file)
					.map(|s| s.to_string())
					.unwrap_or(file.to_string());
				let file = PathBuf::from(file);
				let dir = match matches.value_of("dir") {
					Some(dir) => {
						let dir = shellexpand::full(dir)
							.map(|s| s.to_string())
							.unwrap_or(dir.to_string());
						PathBuf::from(dir)
					}
					None => File::get_default_path(&format!(
						"{}_frames",
						file.file_stem()
							.unwrap_or_default()
							.to_str()
							.unwrap_or_default(),
					)),
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
	use pretty_assertions::assert_eq;
	use std::ffi::OsStr;
	#[test]
	fn test_anim_settings() {
		let args = App::new("test")
			.arg(Arg::with_name("fps").long("fps").takes_value(true))
			.arg(Arg::with_name("repeat").long("repeat").takes_value(true))
			.arg(Arg::with_name("quality").long("quality").takes_value(true))
			.arg(Arg::with_name("gifski").long("gifski"))
			.arg(Arg::with_name("fast").long("fast"))
			.arg(Arg::with_name("speed").long("speed").takes_value(true))
			.arg(
				Arg::with_name("cut-beginning")
					.long("cut-beginning")
					.takes_value(true),
			)
			.arg(Arg::with_name("cut-end").long("cut-end").takes_value(true))
			.get_matches_from(vec![
				"test",
				"--fps",
				"15",
				"--repeat",
				"5",
				"--quality",
				"10",
				"--gifski",
				"--fast",
				"--speed",
				"1.1",
				"--cut-beginning",
				"0.9",
				"--cut-end",
				"0.8",
			]);
		let anim_settings = AnimSettings::from_parser(ArgParser::from_args(&args));
		assert_eq!(15, anim_settings.fps);
		assert_eq!(4, anim_settings.repeat);
		assert_eq!(10, anim_settings.quality);
		assert_eq!(true, anim_settings.gifski.0);
		assert_eq!(true, anim_settings.gifski.1);
		assert_eq!(1.1, anim_settings.speed);
		assert_eq!((900., 800.), anim_settings.cut);
		let anim_settings = AnimSettings::from_parser(ArgParser::new(None));
		assert_eq!(-1, anim_settings.repeat);
		assert_eq!(75, anim_settings.quality);
		assert_eq!(false, anim_settings.gifski.0);
		assert_eq!(false, anim_settings.gifski.1);
		assert_eq!(1.0, anim_settings.speed);
		assert_eq!((0., 0.), anim_settings.cut);
	}
	#[test]
	fn test_split_settings() {
		let args = App::new("test")
			.arg(Arg::with_name("file").required(true))
			.get_matches_from(vec!["test", "x"]);
		let split_settings = SplitSettings::from_parser(ArgParser::from_args(&args));
		assert_eq!(PathBuf::from("x"), split_settings.file);
		assert_eq!(Some(OsStr::new("x_frames")), split_settings.dir.file_name());
		let args = App::new("test")
			.arg(Arg::with_name("dir").long("dir").takes_value(true))
			.get_matches_from(vec!["test", "--dir", "~/"]);
		let split_settings = SplitSettings::from_parser(ArgParser::from_args(&args));
		assert_eq!(dirs::home_dir().unwrap(), split_settings.dir)
	}
}
