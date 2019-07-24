use piston_window::*;
use crate::smoothing::circbuf::CircBuf;
use std::time::{SystemTime, Duration};
use std::thread;

const SIZE: [f64; 2] = [1280., 720.];

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

	println!("Sample rate: approx. {}/s", frequency);

	let mut window: PistonWindow =
		WindowSettings::new("Velocity Diagram", SIZE)
			.exit_on_esc(true).build().unwrap();

	let circ_buf_size = 10 * frequency as usize;
	let mut circ_buf: CircBuf<(f32, f32, f32, f32)> = CircBuf::new(circ_buf_size);

	println!("Populating the visualizer's buffer...");

	for _ in 0..circ_buf_size {
		circ_buf.push(data.next().unwrap());
	}

	println!("Visualizer's buffer has been populated.");

	let mut last_vel = (0., 0., 0.);
	let x_factor = SIZE[0] / 11.;
	let x_offset = SIZE[0] / 22.;
	let y_factor = 7. * SIZE[1] / 16e-1;
	let y_offset = SIZE[1] / 2.;

	let mut timings = SystemTime::now();
	while let Some(e) = window.next() {
		window.draw_2d(&e, |ctx, g2d, _device| {
			clear([1.0; 4], g2d);

			let mut time = 0.;
			circ_buf.iter()
				.for_each(|(dt, vx, vy, vz)| {
					let (dt, vx, vy, vz) = (*dt as f64, *vx as f64, *vy as f64, *vz as f64);

					let xpair = [time * x_factor + x_offset, (time + dt) * x_factor + x_offset];

					let x = [xpair[0], last_vel.0 * y_factor + y_offset, xpair[1], vx * y_factor + y_offset];
					let y = [xpair[0], last_vel.1 * y_factor + y_offset, xpair[1], vy * y_factor + y_offset];
					let z = [xpair[0], last_vel.2 * y_factor + y_offset, xpair[1], vz * y_factor + y_offset];

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
					last_vel = (vx, vy, vz);
					time += dt;
				});

			if let Ok(elapsed) = timings.elapsed() {
				let fps = 1e6 / elapsed.as_micros() as f32;

				for _ in 0..(frequency / fps).ceil() as u8 {
					if let Some(item) = data.next() {
						circ_buf.push(item);
					} else {
						println!("Data stream exhausted!");
						loop {
							thread::sleep(Duration::from_secs(1));
						}
					}
				}

				println!("Framerate: {:?}", fps);
			}
			timings = SystemTime::now();
		});
	}
}
