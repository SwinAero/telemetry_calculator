#![feature(test)]
extern crate test;

extern crate nalgebra;

use nalgebra::*;

use std::{io, fs};
use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};

mod iter_calculus;

#[cfg(feature = "simd")]
mod simd;

#[derive(Debug, Default, Clone, Copy)]
pub struct RawTelemUnit {
	pub delta_t: f32,
	pub acc_x: f32,
	pub acc_y: f32,
	pub acc_z: f32,
	pub theta_x: f32,
	pub theta_y: f32,
	pub theta_z: f32,
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
			theta_x: values[4].parse::<f32>()?,
			theta_y: values[5].parse::<f32>()?,
			theta_z: values[6].parse::<f32>()?,
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
	let bufcsv = BufCSV::new("testdata/still.csv")?;

	let baseline = bufcsv.previous;

	let ((t, ax), (ay, az)): ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)) = {
		let into_radians = std::f32::consts::PI / 180.;

		let (a, b): (Vec<_>, Vec<_>) = bufcsv.into_iter()
			.skip(1000)
			.take(1000)
			.map(|mut tdb| {
				tdb.theta_x *= into_radians;
				tdb.theta_y *= into_radians;
				tdb.theta_z *= into_radians;

				let point = Point3::new(tdb.acc_x, tdb.acc_y, tdb.acc_z);
				let rot = Rotation3::from_euler_angles(tdb.theta_x, tdb.theta_y, tdb.theta_z);
				let norm_accel: Point3<f32> = rot.transform_point(&point);

				((tdb.delta_t, norm_accel[0]), (norm_accel[1], norm_accel[2]))
			}).unzip();

		(a.into_iter().unzip(), b.into_iter().unzip())
	};

	Ok(())
}
