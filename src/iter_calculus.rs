use std::ops::{AddAssign, SubAssign};
use std::mem;
use std::fmt::Debug;

#[macro_export]
macro_rules! calculus {
	($variant:ident, $($iter:expr),+) => {
		($(
			$variant::from($iter)
		),+)
	};
}

pub type DifferentiateF32<I> = Differentiate<I, f32>;

pub struct Differentiate<I, T> {
	last: Option<T>,
	inner: I,
}

impl<I: Iterator, T> Differentiate<I, T> {
	pub fn from(inner: I) -> Self {
		Differentiate {
			last: None,
			inner,
		}
	}
}

impl<I: Iterator<Item=T>, T> Iterator for Differentiate<I, T>
	where T: SubAssign + Copy + Debug + Default {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let cur = if let Some(i) = self.inner.next() {
			i
		} else {
			return None;
		};

		if let Some(ref mut last) = self.last {
			let mut last = mem::replace(last, cur);

			last -= cur;

			Some(last)
		} else {
			self.last = Some(cur);

			Some(T::default())
		}
	}
}

pub type IntegrateF32<I> = Integrate<I, f32>;

pub struct Integrate<I, T> {
	accumulator: T,
	inner: I,
}

impl<I: Iterator, T: Default> Integrate<I, T> {
	pub fn from(inner: I) -> Self {
		Integrate {
			accumulator: <T as Default>::default(),
			inner,
		}
	}
}

impl<I: Iterator<Item=T>, T> Iterator for Integrate<I, T>
	where T: AddAssign + Copy + Debug {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(i) = self.inner.next() {
			self.accumulator += i;
			return Some(self.accumulator);
		} else {
			None
		}
	}
}
