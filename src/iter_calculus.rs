use std::ops::{AddAssign, Sub};
use std::mem;

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
	where T: Sub + Copy,
		  <T as Sub>::Output: Default {
	type Item = <T as Sub>::Output;

	fn next(&mut self) -> Option<Self::Item> {
		let mut cur = if let Some(i) = self.inner.next() {
			i
		} else {
			return None;
		};

		if let Some(ref mut last) = self.last {
			mem::swap(last, &mut cur);

			Some(*last - cur)
		} else {
			self.last = Some(cur);

			Some(<T as Sub>::Output::default())
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
	where T: AddAssign + Copy {
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
