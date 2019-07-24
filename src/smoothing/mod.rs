use crate::smoothing::circbuf::CircBuf;
use std::ops::{AddAssign, DivAssign, Mul, Add};

pub mod circbuf;

pub type WeightedMovingAvgF32<I, F> = WeightedMovingAvg<I, F, f32>;

pub struct WeightedMovingAvg<I, F, T> {
	circbuf: CircBuf<T>,
	inner: I,
	factor: F,
}

impl<I, F, T> WeightedMovingAvg<I, F, T> where
	I: Iterator<Item=T>,
	F: Iterator<Item=T>,
	T: Default {
	pub fn from(inner: I, factor: F) -> Self {
		WeightedMovingAvg {
			circbuf: CircBuf::new(20),
			inner,
			factor,
		}
	}
}

impl<I, F, T> Iterator for WeightedMovingAvg<I, F, T>
	where I: Iterator<Item=T>,
		  F: Iterator<Item=T>,
		  T: Add + Mul + Default + Copy,
		  <T as Add>::Output: Into<T>,
		  <T as Mul>::Output: Into<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let (next, factor) = if let (Some(next), Some(factor)) = (self.inner.next(), self.factor.next()) {
			(next, factor)
		} else {
			return None;
		};

		self.circbuf.push((next * factor).into());

		Some(self.circbuf.sum())
	}
}