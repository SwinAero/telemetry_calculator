use std::mem;
use std::ops::Add;

pub struct CircBuf<T> {
	buf: Vec<Option<T>>,
	index: usize,
}

impl<T: Default> CircBuf<T> {
	pub fn new(size: usize) -> Self {
		CircBuf {
			buf: Vec::with_capacity(size),
			index: 0,
		}
	}

	pub fn push(&mut self, t: T) -> Option<T> {
		if self.buf.len() < self.buf.capacity() {
			self.buf.push(Some(t));

			return None;
		}

		let old = mem::replace(&mut self.buf[self.index], Some(t));

		return old;
	}
}

impl<T> CircBuf<T> where
	T: Add + Default + Copy,
	<T as Add>::Output: Into<T> {
	pub fn sum(&self) -> T {
		self.buf.iter()
			.filter(|x| x.is_some())
			.map(|x| x.unwrap())
			.fold(T::default(), |acc, item| (acc + item).into())
	}
}
