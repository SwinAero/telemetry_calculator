#![feature(test)]
extern crate test;

extern crate packed_simd; // This program's speed can be improved by an order of magnitude if optimised using Single Instruction Multiple Data instructions

use std::{io, fs};

use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};
use crate::simd::Telemetryx16Iter;
use packed_simd::f32x16;

mod simd;

#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryDataUnit {
	pub delta_t: u32,
	pub acc_x: f32,
	pub acc_y: f32,
	pub acc_z: f32,
	pub theta_x: f32,
	pub theta_y: f32,
	pub theta_z: f32,
}

impl FromStr for TelemetryDataUnit {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let values = s.trim().split(", ").collect::<Vec<&str>>();

		if values.len() != 7 {
			return Err(Box::new(CSVDeErr("Incorrectly sized telemetry data unit detected, dropping entire row.".to_owned())));
		}

		Ok(TelemetryDataUnit {
			delta_t: values[0].parse::<u32>()?,
			acc_x: values[0].parse::<f32>()?,
			acc_y: values[0].parse::<f32>()?,
			acc_z: values[0].parse::<f32>()?,
			theta_x: values[0].parse::<f32>()?,
			theta_y: values[0].parse::<f32>()?,
			theta_z: values[0].parse::<f32>()?,
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
	previous: TelemetryDataUnit,
	line_index: usize,
}

impl BufCSV {
	pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
		let source = fs::OpenOptions::new()
			.read(true)
			.open(path)?;

		let mut bufcsv = BufCSV {
			source: BufReader::new(source),
			previous: TelemetryDataUnit::default(),
			line_index: 0,
		};
		// First data point is needed for baseline
		bufcsv.previous = if let Some(mut tdu) = bufcsv.next() {
			tdu.delta_t = 0; // By definition the first data unit has no temporal baseline, and the unstable nature of this value from hardware shifts the integral, therefore this is zeroed to prevent any problem.
			tdu
		} else {
			return Err(Box::new(CSVDeErr("Failed to find a baseline for data.".to_string())));
		};

		Ok(bufcsv)
	}
}

impl Iterator for BufCSV {
	type Item = TelemetryDataUnit;

	fn next(&mut self) -> Option<Self::Item> {
		self.line_index += 1;

		let mut line = String::new();

		if let Ok(count) = self.source.read_line(&mut line) {
			if count == 0 {
				return None;
			}

			match line.parse() {
				Ok(tdu) => Some(tdu),
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

#[bench]
fn simd_instructions(b: &mut test::Bencher) -> Result<(), Box<dyn Error>> {
	let bufcsv = BufCSV::new("TELEM_0.CSV")?;
	let blocks = Telemetryx16Iter::from(bufcsv.into_iter()).collect::<Vec<_>>();

	let into_radians = f32x16::splat(std::f32::consts::PI / 180.);
	b.iter(|| {
		let _ = test::black_box(
			blocks.clone().into_iter()
				.map(|mut block| {
					block.theta_x *= into_radians;
					block.theta_y *= into_radians;
					block.theta_z *= into_radians;

					block
				})
		);
	});

	Ok(())
}

#[bench]
fn standard_optimization(b: &mut test::Bencher) -> Result<(), Box<dyn Error>> {
	let bufcsv = BufCSV::new("TELEM_0.CSV")?;
	let units = bufcsv.into_iter().collect::<Vec<_>>();

	let into_radians = std::f32::consts::PI / 180.;
	b.iter(|| {
		let _ = test::black_box(
			units.clone().into_iter()
				.map(|mut unit| {
					unit.theta_x *= into_radians;
					unit.theta_y *= into_radians;
					unit.theta_z *= into_radians;

					unit
				})
		);
	});

	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let bufcsv = BufCSV::new("TELEM_0.CSV")?;

	let baseline = bufcsv.previous;

	let into_radians = f32x16::splat(std::f32::consts::PI / 180.);
	let _ = Telemetryx16Iter::from(bufcsv.into_iter())
		.map(|mut tdb| {
			tdb.theta_x *= into_radians;
			tdb.theta_y *= into_radians;
			tdb.theta_z *= into_radians;

			tdb
		});

	Ok(())
}
