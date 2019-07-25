use serial::core::SerialDevice;
use serial::{SerialPortSettings, BaudRate};
use serial::unix::TTYPort;

use std::io::{BufRead, Error as IOError, Read};
use std::path::Path;

const MAX_BYTES_PER_CALL: usize = 256;

pub struct Cereal {
	tty: TTYPort,
	buf: Vec<u8>,
	index: usize,
}

impl Cereal {
	pub fn new(path: &str) -> Cereal {
		let mut cereal = TTYPort::open(Path::new(path)).unwrap();

		let mut settings = cereal.read_settings().unwrap();

		settings.set_baud_rate(BaudRate::Baud115200).unwrap();

		cereal.write_settings(&settings).unwrap();

		let mut cereal = Cereal {
			tty: cereal,
			buf: vec![0u8; MAX_BYTES_PER_CALL],
			index: 0,
		};

		println!("Waiting for connection...");
		while cereal.index < MAX_BYTES_PER_CALL {
			let _ = cereal.fill_buf();
		}
		println!("Connected!");

		return cereal;
	}
}

impl Read for Cereal {
	fn read(&mut self, _buf: &mut [u8]) -> Result<usize, IOError> {
		panic!("use the BufRead impl please")
	}
}

impl BufRead for Cereal {
	fn fill_buf(&mut self) -> Result<&[u8], IOError> {
		let read = self.tty.read(&mut self.buf[self.index..self.index + MAX_BYTES_PER_CALL])?;

		self.index += read;
		self.buf.extend((0..read).map(|_| 0u8));

		Ok(&self.buf[..self.buf.len() - MAX_BYTES_PER_CALL])
	}

	fn consume(&mut self, amt: usize) {
		for _ in 0..amt {
			self.buf.remove(0);
		}

		self.index -= amt;
	}
}
