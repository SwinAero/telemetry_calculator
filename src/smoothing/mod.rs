use crate::smoothing::circbuf::CircBuf;
use std::ops::{Mul, Add, Div};

pub mod circbuf;

pub type WeightedMovingAvgF32<I, F> = WeightedMovingAvg<I, F, f32>;

pub struct WeightedMovingAvg<I, F, T> {
	circbuf: CircBuf<(T, T)>,
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
		  T: Add + Mul + Div + Default + Copy,
		  <T as Add>::Output: Into<T>,
		  <T as Mul>::Output: Into<T>,
		  <T as Div>::Output: Into<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let (next, factor) = if let (Some(item), Some(factor)) = (self.inner.next(), self.factor.next()) {
			(item, factor)
		} else {
			return None;
		};

		self.circbuf.push((next, factor));

		let (a, d) = self.circbuf
			.fold((T::default(), T::default()), |(a, a_f), (item, factor)| {
				((a + (item * factor).into()).into(), (a_f + factor).into())
			});

		Some((a / d).into())
	}
}