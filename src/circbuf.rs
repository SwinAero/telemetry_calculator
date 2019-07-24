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

		self.index += 1;
		self.index %= self.buf.len();

		return old;
	}

	pub fn iter(&self) -> Iter<T> {
		Iter { buf: &self.buf, offset: self.index, i: 0 }
	}
}

#[cfg(feature = "smooth")]
impl<T: Copy> CircBuf<T> {
	pub fn fold<F, A>(&self, init: A, f: F) -> A
		where F: FnMut(A, T) -> A {
		self.buf.iter()
			.filter(|x| x.is_some())
			.map(|x| x.unwrap())
			.fold(init, f)
	}
}

pub struct Iter<'a, T> {
	buf: &'a Vec<Option<T>>,
	offset: usize,
	i: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		let len = self.buf.len();

		if self.i == len {
			return None;
		}

		let item = &self.buf[(self.i + self.offset) % len];

		self.i += 1;

		if let Some(ref inner) = item {
			return Some(inner);
		} else {
			return None;
		}
	}
}
