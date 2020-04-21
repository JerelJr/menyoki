pub mod fps;
use crate::image::gif::Frame;
use crate::image::Image;
use crate::record::fps::{FpsClock, TimeUnit};
use std::sync::mpsc;
use std::thread;

pub struct RecordResult {
	pub thread: thread::JoinHandle<Vec<Frame>>,
	pub sender: mpsc::Sender<()>,
}

pub struct Recorder {
	clock: FpsClock,
	channel: (mpsc::Sender<()>, mpsc::Receiver<()>),
}

impl Recorder {
	pub fn new(clock: FpsClock) -> Self {
		Self {
			clock,
			channel: mpsc::channel(),
		}
	}

	pub fn record(
		self,
		get_image: impl Fn() -> Option<Image> + Sync + Send + 'static,
	) -> RecordResult {
		let recorder = Box::leak(Box::new(self));
		let mut tick = 0.0;
		let mut frames = Vec::new();
		RecordResult {
			sender: recorder.channel.0.clone(),
			thread: thread::spawn(move || {
				while recorder.channel.1.try_recv().is_err() {
					tick = if tick >= 0. {
						recorder.clock.get_fps(TimeUnit::Millisecond)
							- ((tick / 1e6) as f64).round() as f32
					} else {
						recorder.clock.get_fps(TimeUnit::Millisecond)
					};
					println!("{}", tick);
					frames
						.push(Frame::new(get_image().unwrap(), (tick / 10.) as u16));
					tick = recorder.clock.tick();
				}
				frames
			}),
		}
	}
}
