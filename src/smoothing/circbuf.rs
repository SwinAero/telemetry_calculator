use std::mem;

pub struct CircBuf<T> {
	buf: Vec<Option<T>>,
	index: usize,
}

impl<T> CircBuf<T> {
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

impl<T: Copy> CircBuf<T> {
	pub fn fold<F, A>(&self, init: A, f: F) -> A
		where F: FnMut(A, T) -> A {
		self.buf.iter()
			.filter(|x| x.is_some())
			.map(|x| x.unwrap())
			.fold(init, f)
	}
}