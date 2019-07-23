extern crate packed_simd; // This program's speed can be improved by an order of magnitude if optimised using Single Instruction Multiple Data instructions

pub mod conv;

use packed_simd::{u32x16, f32x16};
use conv::*;
use crate::TelemetryDataUnit;
use std::vec::IntoIter;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Telemetryx16Iter {
	count: usize,
	delta_t: IntoIter<u32x16>,
	acc_x: IntoIter<f32x16>,
	acc_y: IntoIter<f32x16>,
	acc_z: IntoIter<f32x16>,
	theta_x: IntoIter<f32x16>,
	theta_y: IntoIter<f32x16>,
	theta_z: IntoIter<f32x16>,
}

macro_rules! poly_let_mut {
	($initial:expr, $($name:ident),+)=>{
		$(
			let mut $name = $initial;
		)+
	}
}

macro_rules! poly_push {
	($source:expr, $($key:ident),+)=>{
		$(
			$key.push($source.$key);
		)+
	}
}

macro_rules! poly_simdize {
	($simd_fn:ident, $($var:ident),+)=>{
		$(
			let $var = $simd_fn($var.into_iter()).into_iter();
		)+
	}
}

macro_rules! poly_iter {
	($source:expr, $($var:ident),+)=>{
		if let ($(Some($var)),+) = ($($source.$var.next()),+) {
			Some(($($var),+))
		} else {
			None
		}
	}
}

impl<I> From<I> for Telemetryx16Iter where I: Iterator<Item=TelemetryDataUnit> {
	fn from(units: I) -> Self {
		poly_let_mut!(vec![], delta_t, acc_x, acc_y, acc_z, theta_x, theta_y, theta_z);

		let mut count = 0;

		for tdu in units {
			poly_push!(tdu, delta_t, acc_x, acc_y, acc_z, theta_x, theta_y, theta_z);
			count += 1;
		}

		poly_simdize!(into_u32x16_blocks, delta_t);
		poly_simdize!(into_f32x16_blocks, acc_x, acc_y, acc_z, theta_x, theta_y, theta_z);

		return Telemetryx16Iter {
			count,
			delta_t,
			acc_x,
			acc_y,
			acc_z,
			theta_x,
			theta_y,
			theta_z,
		};
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryDataUnitx16 {
	pub delta_t: u32x16,
	pub acc_x: f32x16,
	pub acc_y: f32x16,
	pub acc_z: f32x16,
	pub theta_x: f32x16,
	pub theta_y: f32x16,
	pub theta_z: f32x16,
}

impl From<(u32x16, f32x16, f32x16, f32x16, f32x16, f32x16, f32x16)> for TelemetryDataUnitx16 {
	fn from(src: (u32x16, f32x16, f32x16, f32x16, f32x16, f32x16, f32x16)) -> Self {
		TelemetryDataUnitx16 {
			delta_t: src.0,
			acc_x: src.1,
			acc_y: src.2,
			acc_z: src.3,
			theta_x: src.4,
			theta_y: src.5,
			theta_z: src.6,
		}
	}
}

impl Iterator for Telemetryx16Iter {
	type Item = TelemetryDataUnitx16;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(block) = poly_iter!(self, delta_t, acc_x, acc_y, acc_z, theta_x, theta_y, theta_z) {
			Some(block.into())
		} else {
			None
		}
	}
}