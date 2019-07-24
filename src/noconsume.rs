use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

pub struct Unziperator<I, T> {
	inner: I,
	linked: Vec<Sender<Option<T>>>,
}

pub struct UnzippedIter<T> {
	inner: Receiver<Option<T>>
}

impl<T> Iterator for UnzippedIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.recv();
		unimplemented!()
	}
}

pub fn unzip7<I, O, T>(mut iter: I) -> (Unziperator<I, T>, O, O, O, O, O, O, O)
	where I: Iterator<Item=(T, T, T, T, T, T, T)>,
		  O: Iterator<Item=T> {
	let (rx, tx) = mpsc::channel::<Option<T>>();

	unimplemented!()
}
