use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

pub struct Unziperator<I, T> {
	inner: I,
	children: Vec<Sender<Option<T>>>,
}

impl<I, T> Unziperator<I, T>
	where I: Iterator<Item=[T; 4]> {
	pub fn new(inner: I) -> Self {
		Unziperator {
			inner,
			children: vec![],
		}
	}

	pub fn subscribe(&mut self) -> NCIterChild<T> {
		let (tx, rx) = mpsc::channel();

		self.children.push(tx);

		NCIterChild {
			inner: rx
		}
	}
}

impl<'a, I, T> Iterator for Unziperator<I, T>
	where I: Iterator<Item=[T; 4]>,
		  T: Clone {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(inner) = self.inner.next() {
			self.children.iter()
				.enumerate()
				.for_each(|(i, child)| {
					let _ = child.send(Some(inner[i + 1].clone()));
				});

			Some(inner[0].clone())
		} else {
			self.children.iter()
				.for_each(|child| {
					let _ = child.send(None);
				});

			None
		}
	}
}

pub struct NCIterChild<T> {
	inner: Receiver<Option<T>>,
}

impl<T> Iterator for NCIterChild<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if let Ok(item) = self.inner.try_recv() {
			item
		} else {
			None
		}
	}
}

pub struct Teeterator<I, T> {
	inner: I,
	children: Vec<Sender<Option<T>>>,
}

impl<I, T> Teeterator<I, T> {
	pub fn new(inner: I) -> Self {
		Teeterator {
			inner,
			children: vec![],
		}
	}

	pub fn subscribe(&mut self) -> NCIterChild<T> {
		let (tx, rx) = mpsc::channel();

		self.children.push(tx);

		NCIterChild {
			inner: rx
		}
	}
}

impl<'a, I, T> Iterator for Teeterator<I, T>
	where I: Iterator<Item=T>,
		  T: Copy {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(item) = self.inner.next() {
			self.children.iter()
				.for_each(|child| {
					let _ = child.send(Some(item));
				});

			Some(item)
		} else {
			self.children.iter()
				.for_each(|child| {
					let _ = child.send(None);
				});

			None
		}
	}
}

