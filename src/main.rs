#[cfg(feature = "normalize")]
extern crate nalgebra;
extern crate serial;

#[cfg(feature = "normalize")]
use nalgebra::*;

use std::fs::{File, OpenOptions};
use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader, Lines};

#[macro_use]
mod calculus;

use calculus::*;

mod circbuf;

#[macro_use]
mod noconsume;

use noconsume::*;

mod serialbuf;

use serialbuf::*;

#[cfg(all(feature = "hidegravity", feature = "smooth"))]
mod smoothing;

#[cfg(all(feature = "hidegravity", feature = "smooth"))]
use smoothing::*;

#[cfg(feature = "visualize")]
mod visualize;

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

pub struct BufCSV<R> {
	source: Lines<R>,
	line_index: usize,
}

impl<R: BufRead> BufCSV<R> {
	pub fn new(source: R) -> Self {
		BufCSV {
			source: source.lines(),
			line_index: 0,
		}
	}
}

impl BufCSV<BufReader<File>> {
	pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
		let source = OpenOptions::new()
			.read(true)
			.open(path)?;

		let br = BufReader::new(source);

		Ok(Self::new(br))
	}
}

impl<T: BufRead> Iterator for BufCSV<T> {
	type Item = RawTelemUnit;

	fn next(&mut self) -> Option<Self::Item> {
		self.line_index += 1;

		if let Some(Ok(line)) = self.source.next() {
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

	// /dev/ttyUSB0 is the port if the clone arduino uses the CH340 USB chip, for genuine arduinos using ATmega16u2, use /dev/ttyACM0
	let bufcsv = BufCSV::new(Cereal::new("/dev/ttyUSB0"));

	// let bufcsv = BufCSV::from_file("testdata/drop.csv")?;

	let data = bufcsv
		.map(|mut tdb| {
			#[cfg(feature = "normalize")] {
				tdb.roll *= into_radians;
				tdb.pitch *= into_radians;
				tdb.yaw *= into_radians;

				let point = Point3::new(tdb.acc_x, tdb.acc_y, tdb.acc_z);
				let rot = Rotation3::from_euler_angles(-tdb.roll, -tdb.pitch, -tdb.yaw);
				let norm_accel: Point3<f32> = rot.transform_point(&point);

				return [tdb.delta_t, norm_accel[0], norm_accel[1], norm_accel[2]];
			}
			#[cfg(not(feature = "normalize"))] {
				[tdb.delta_t, tdb.acc_x, tdb.acc_y, tdb.acc_z]
			}
		});

	let mut unziperator = Unziperator::new(data);

	let ax = unziperator.subscribe();
	let ay = unziperator.subscribe();
	let az = unziperator.subscribe();
	let mut dt = Teeterator::new(unziperator);

	#[cfg(feature = "hidegravity")]
		let (jx, jy, jz) = calculus!(dt, DifferentiateF32, ax, ay, az);
	#[cfg(all(feature = "hidegravity", feature = "smooth"))]
		let (jx, jy, jz) = calculus!(dt, WeightedMovingAvgF32, jx, jy, jz);
	#[cfg(feature = "hidegravity")]
		let (ax, ay, az) = calculus!(dt, IntegrateF32, jx, jy, jz);
	// TODO: Motion compensation so it doesn't fly off
	// let (vx, vy, vz) = calculus!(dt, IntegrateF32, ax, ay, az);

	let data_src = dt
		.zip(ax)
		.zip(ay
			.zip(az)
		)
		.map(|((t, x), (y, z))| {
			(t, x, y, z)
		});

	#[cfg(feature = "visualize")]
		visualize::run(data_src);

	Ok(())
}
