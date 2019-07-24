extern crate nalgebra;
#[cfg(feature = "visualize")]
extern crate piston_window;

use nalgebra::*;
use piston_window::*;

use std::{io, fs};
use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};

#[macro_use]
mod calculus;

use calculus::*;

#[macro_use]
mod noconsume;

use noconsume::*;

#[cfg(feature = "smooth")]
mod smoothing;

#[cfg(feature = "smooth")]
use smoothing::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct RawTelemUnit {
	pub delta_t: f32,
	pub acc_x: f32,
	pub acc_y: f32,
	pub acc_z: f32,
	pub roll: f32,
	pub pitch: f32,
	pub yaw: f32,
}

impl FromStr for RawTelemUnit {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let values = s.trim().split(", ").collect::<Vec<&str>>();

		if values.len() != 7 {
			return Err(Box::new(CSVDeErr("Incorrectly sized telemetry data unit detected, dropping entire row.".to_owned())));
		}

		Ok(RawTelemUnit {
			delta_t: values[0].parse::<u32>()? as f32 * 1e-6,// Input is in microseconds
			acc_x: values[1].parse::<f32>()?,
			acc_y: values[2].parse::<f32>()?,
			acc_z: values[3].parse::<f32>()?,
			roll: values[4].parse::<f32>()?,
			pitch: values[5].parse::<f32>()?,
			yaw: values[6].parse::<f32>()?,
		})
	}
}

#[derive(Debug)]
pub struct CSVDeErr(String);

impl fmt::Display for CSVDeErr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Error for CSVDeErr {}

pub struct BufCSV {
	source: io::BufReader<fs::File>,
	previous: RawTelemUnit,
	line_index: usize,
}

impl BufCSV {
	pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
		let source = fs::OpenOptions::new()
			.read(true)
			.open(path)?;

		let mut bufcsv = BufCSV {
			source: BufReader::new(source),
			previous: RawTelemUnit::default(),
			line_index: 0,
		};
		// First data point is needed for baseline
		bufcsv.previous = if let Some(mut rtu) = bufcsv.next() {
			rtu.delta_t = 0.; // By definition the first data unit has no temporal baseline, and the unstable nature of this value from hardware shifts the integral, therefore this is zeroed to prevent any problem.
			rtu
		} else {
			return Err(Box::new(CSVDeErr("Failed to find a baseline for data.".to_string())));
		};

		Ok(bufcsv)
	}
}

impl Iterator for BufCSV {
	type Item = RawTelemUnit;

	fn next(&mut self) -> Option<Self::Item> {
		self.line_index += 1;

		let mut line = String::new();

		if let Ok(count) = self.source.read_line(&mut line) {
			if count == 0 {
				return None;
			}

			match line.parse() {
				Ok(rtu) => Some(rtu),
				Err(err) => {
					eprintln!("Deserializing line {} failed: {:?}", self.line_index, err);
					self.next()
				}
			}
		} else {
			None
		}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let into_radians = std::f32::consts::PI / 180.;

	let buf = BufCSV::new("testdata/drop.csv")?
		.map(|mut tdb| {
			tdb.roll *= into_radians;
			tdb.pitch *= into_radians;
			tdb.yaw *= into_radians;

			let point = Point3::new(tdb.acc_x, tdb.acc_y, tdb.acc_z);
			let rot = Rotation3::from_euler_angles(tdb.roll, tdb.pitch, tdb.yaw);
			let norm_accel: Point3<f32> = rot.transform_point(&point);

			[tdb.delta_t, norm_accel[0], norm_accel[1], norm_accel[2]]
		});

	let mut unziperator = Unziperator::new(buf);

	let ax = unziperator.subscribe();
	let ay = unziperator.subscribe();
	let az = unziperator.subscribe();
	let mut dt = Teeterator::new(unziperator);

	let (jx, jy, jz) = calculus!(dt, DifferentiateF32, ax, ay, az);
	#[cfg(feature = "smooth")]
		let (jx, jy, jz) = calculus!(dt, WeightedMovingAvgF32, jx, jy, jz);
	let (ax, ay, az) = calculus!(dt, IntegrateF32, jx, jy, jz);
	let (vx, vy, vz) = calculus!(dt, IntegrateF32, ax, ay, az);
	let (dx, dy, dz) = calculus!(dt, IntegrateF32, vx, vy, vz);

	let fd = dt
		.zip(dx)
		.zip(dy
			.zip(dz)
		)
		.map(|((t, x), (y, z))| {
			(t, x, y, z)
		})
		.fold((0f32, 0f32, 0f32, 0f32), |mut accum, this| {
			accum.0 += this.0;
			accum.1 += this.0 * this.1;
			accum.2 += this.0 * this.2;
			accum.3 += this.0 * this.3;

			accum
		});

	println!("{:?}", fd);

	Ok(())
}
