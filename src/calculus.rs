use std::ops::{AddAssign, SubAssign, Mul};
use std::mem;
use std::fmt::Debug;

#[macro_export]
macro_rules! calculus {
	($no_consume_iter:expr, $variant:ident, $($iter:expr),+) => {
		($(
			$variant::from($iter, $no_consume_iter.subscribe())
		),+)
	};
}

pub type DifferentiateF32<I, F> = Differentiate<I, F, f32>;

pub struct Differentiate<I, F, T> {
	last: Option<T>,
	inner: I,
	factor: F,
}

impl<'a, I: Iterator, F: Iterator, T> Differentiate<I, F, T> {
	pub fn from(inner: I, factor: F) -> Self {
		Differentiate {
			last: None,
			inner,
			factor,
		}
	}
}

impl<I, F, T> Iterator for Differentiate<I, F, T>
	where
		I: Iterator<Item=T>,
		F: Iterator<Item=T>,
		T: SubAssign + Copy + Debug + Default + Mul,
		<T as Mul>::Output: Into<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let (cur, factor) = if let (Some(i), Some(f)) = (self.inner.next(), self.factor.next()) {
			(i, f)
		} else {
			return None;
		};

		if let Some(ref mut last) = self.last {
			let mut last = mem::replace(last, cur);

			last -= cur;

			Some((last * factor).into())
		} else {
			self.last = Some(cur);

			Some(T::default())
		}
	}
}

pub type IntegrateF32<I, F> = Integrate<I, F, f32>;

pub struct Integrate<I, F, T> {
	accumulator: T,
	inner: I,
	factor: F,
}

impl<I, F, T: Default> Integrate<I, F, T> {
	pub fn from(inner: I, factor: F) -> Self {
		Integrate {
			accumulator: <T as Default>::default(),
			inner,
			factor,
		}
	}
}

impl<I, F, T> Iterator for Integrate<I, F, T>
	where
		I: Iterator<Item=T>,
		F: Iterator<Item=T>,
		T: AddAssign + Mul + Copy + Debug,
		<T as Mul>::Output: Into<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if let (Some(i), Some(f)) = (self.inner.next(), self.factor.next()) {
			self.accumulator += (i * f).into();
			return Some(self.accumulator);
		} else {
			None
		}
	}
}
