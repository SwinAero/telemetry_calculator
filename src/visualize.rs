extern crate piston_window;

use piston_window::*;
use crate::circbuf::CircBuf;
use std::time::SystemTime;
// use graphics::glyph_cache::rusttype::GlyphCache;

const SIZE: [f64; 2] = [1280., 720.];
const BUFFERED_SECS: usize = 10;

pub fn run<I>(mut data: I)
	where I: Iterator<Item=(f32, f32, f32, f32)> {
	let frequency = {
		let mut frequency = 0.;
		let mut time = 0.;

		while time < 1. {
			time += data.next().unwrap().0;
			frequency += 1.;
		}

		frequency / time
	};

	println!("Guessing sample rate: {}/s?", frequency);

	let circ_buf_size = BUFFERED_SECS * frequency as usize;
	let mut circ_buf: CircBuf<(f32, f32, f32, f32)> = CircBuf::new(circ_buf_size);

	println!("Populating the visualizer's buffer... This may take a while...");

	for _ in 0..circ_buf_size {
		circ_buf.push(data.next().unwrap());
	}

	println!("Visualizer's buffer has been populated.");

	let mut window: PistonWindow =
		WindowSettings::new("Acceleration Demo", SIZE)
			.exit_on_esc(true).build().unwrap();

	let mut last_accel = (0., 0., 0., 0.);
	let x_factor = SIZE[0] / 11.;
	let x_offset = SIZE[0] / 22.;
	let y_factor = 7. * SIZE[1] / 16e-3; // 10m * 10^-3 for entire FoV
	let y_offset = SIZE[1] / 2.;

	// let gc = GlyphCache::new(Font::from_bytes(include_bytes!("../fonts/font.ttf")));

	while let Some(e) = window.next() {
		window.draw_2d(&e, |ctx, g2d, _device| {
			clear([1.0; 4], g2d);

			line(
				[0.0, 0.0, 0.0, 1.0], // black
				3.0,
				[x_offset, y_offset, SIZE[0] - x_offset, y_offset],
				ctx.transform,
				g2d,
			);

			for i in 0..=10 {
				let x = x_offset + i as f64 * x_factor;
				line(
					[0.0, 0.0, 0.0, 0.2], // grey
					1.0,
					[x, SIZE[1] / 16., x, SIZE[1] * 15. / 16.],
					ctx.transform,
					g2d,
				);

				// text([0.0, 0.0, 0.0, 1.0],24,&format!("{}", i), gc, ctx.transform,g2d);
			}

			let mut time = 0.;
			circ_buf.iter()
				.filter(|(dt, _, _, _)| *dt > 0.)
				.for_each(|(dt, ax, ay, az)| {
					let (dt, ax, ay, az) = (*dt as f64, -*ax as f64, -*ay as f64, -*az as f64);
					let vsum = (ax.powf(2.) + ay.powf(2.) + az.powf(2.)).sqrt();

					if last_accel == (0., 0., 0., 0.) {
						last_accel = (ax, ay, az, vsum);
						return;
					}

					let xpair = [time * x_factor + x_offset, (time - dt) * x_factor + x_offset];

					let x = [xpair[0], last_accel.0 * y_factor + y_offset, xpair[1], ax * y_factor + y_offset];
					let y = [xpair[0], last_accel.1 * y_factor + y_offset, xpair[1], ay * y_factor + y_offset];
					let z = [xpair[0], last_accel.2 * y_factor + y_offset, xpair[1], az * y_factor + y_offset];
					let sum = [xpair[0], last_accel.3 * y_factor + y_offset, xpair[1], vsum * y_factor + y_offset];

					line(
						[1.0, 0.0, 0.0, 1.0], // red
						1.0,
						x,
						ctx.transform,
						g2d,
					);
					line(
						[0.0, 1.0, 0.0, 1.0], // green
						1.0,
						y,
						ctx.transform,
						g2d,
					);
					line(
						[0.0, 0.0, 1.0, 1.0], // blue
						1.0,
						z,
						ctx.transform,
						g2d,
					);
					line(
						[0.0, 0.0, 0.0, 0.5], // grey
						1.0,
						sum,
						ctx.transform,
						g2d,
					);
					last_accel = (ax, ay, az, vsum);
					time += dt;
				});

			last_accel = (0., 0., 0., 0.);

			let start_of_refresh = SystemTime::now();

			while let Ok(delay) = start_of_refresh.elapsed() {
				// This will limit framerate to 2x refresh
				if delay.as_millis() > (2e3 / frequency).ceil() as u128 {
					break;
				}

				if let Some(item) = data.next() {
					circ_buf.push(item);
				} else {
					println!("Data stream exhausted!");
				}
			}

			while circ_buf.fold(0., |t, (dt, _, _, _)| t + dt) > BUFFERED_SECS as f32 {
				circ_buf.push((0., 0., 0., 0.));
			}
		});
	}
}
