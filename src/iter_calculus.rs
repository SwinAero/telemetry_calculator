use std::ops::{AddAssign, SubAssign};
use std::mem;
use std::fmt::Debug;

#[macro_export]
macro_rules! differentiate {
	($($iter:expr),+) => {
		($(
			Differentiate::from($iter)
		),+)
	};
}

#[macro_export]
macro_rules! integrate {
	($($iter:expr),+) => {
		($(
			Integrate::from($iter)
		),+)
	};
}

pub struct Differentiate<I, T> {
	last: Option<T>,
	inner: I,
}

impl<I: Iterator, T> From<I> for Differentiate<I, T> {
	fn from(inner: I) -> Self {
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
		let mut cur = if let Some(i) = self.inner.next() {
			i
		} else {
			return None;
		};

		if let Some(ref mut last) = self.last {
			let mut last = mem::replace(last,  cur);

			println!("{:?} - {:?}", last, cur);

			last -= cur;

			Some(last)
		} else {
			self.last = Some(cur);

			Some(T::default())
		}
	}
}

pub struct Integrate<I, T> {
	accumulator: T,
	inner: I,
}

impl<I: Iterator, T: Default> From<I> for Integrate<I, T> {
	fn from(inner: I) -> Self {
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
			println!("{:?} + {:?}", self.accumulator, i);
			self.accumulator += i;
			return Some(self.accumulator);
		} else {
			None
		}
	}
}
