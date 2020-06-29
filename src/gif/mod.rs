pub mod settings;

use crate::gif::settings::GifSettings;
use crate::image::geometry::Geometry;
use crate::image::Image;
use crate::util;
use crate::util::state::InputState;
use gif::{Encoder as GifEncoder, Frame, Repeat, SetParameter};
use image::ColorType;
use std::convert::TryInto;
use std::io::{Error, Write};

/* Required encoding methods */
pub trait Encoder<Output: Write> {
	fn new(
		geometry: Geometry,
		output: Output,
		fps: u32,
		settings: GifSettings,
	) -> Result<Self, Error>
	where
		Self: Sized;
	fn save(
		self,
		images: Vec<Image>,
		input_state: &'static InputState,
	) -> Result<(), Error>;
}

/* GIF encoder and settings */
pub struct Gif<Output: Write> {
	fps: u32,
	encoder: GifEncoder<Output>,
	settings: GifSettings,
}

impl<Output: Write> Encoder<Output> for Gif<Output> {
	/**
	 * Create a new Gif object.
	 *
	 * @param  geometry
	 * @param  output
	 * @param  fps
	 * @param  settings
	 * @return Result (Gif)
	 */
	fn new(
		geometry: Geometry,
		output: Output,
		fps: u32,
		settings: GifSettings,
	) -> Result<Self, Error> {
		let mut encoder = GifEncoder::new(
			output,
			geometry.width.try_into().unwrap_or_default(),
			geometry.height.try_into().unwrap_or_default(),
			&[],
		)?;
		encoder.set(match settings.repeat {
			n if n >= 0 => Repeat::Finite(n.try_into().unwrap_or_default()),
			_ => Repeat::Infinite,
		})?;
		Ok(Self {
			fps,
			encoder,
			settings,
		})
	}

	/**
	 * Encode images as frame and write to the GIF file.
	 *
	 * @param  images
	 * @param  input_state
	 * @return Result
	 */
	fn save(
		mut self,
		images: Vec<Image>,
		input_state: &'static InputState,
	) -> Result<(), Error> {
		for image in images {
			if input_state.check_cancel_keys() {
				warn!("User interrupt detected.");
				break;
			}
			let mut frame = Frame::from_rgba_speed(
				image.geometry.width.try_into().unwrap_or_default(),
				image.geometry.height.try_into().unwrap_or_default(),
				&mut image.get_data(ColorType::Rgba8),
				30 - util::map_range(
					self.settings.quality.into(),
					(1., 100.),
					(0., 29.),
				) as i32,
			);
			frame.delay = ((1. / self.fps as f32) * 1e2) as u16;
			self.encoder.write_frame(&frame)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::Bgra;
	#[test]
	fn test_gif_mod() -> Result<(), Error> {
		let geometry = Geometry::new(0, 0, 1, 2, None);
		let settings = GifSettings::new(-1, 10);
		let data = vec![Bgra::from([0, 0, 0, 0]), Bgra::from([255, 255, 255, 0])];
		let frames = vec![
			Frame::new(Image::new(data.clone(), false, geometry), 10),
			Frame::new(
				Image::new(data.into_iter().rev().collect(), false, geometry),
				10,
			),
		];
		let mut gif = Gif::new(geometry, Vec::new(), settings)?;
		gif.save(frames, &InputState::new())?;
		Ok(())
	}
}
