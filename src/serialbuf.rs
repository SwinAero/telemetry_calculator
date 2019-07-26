use serial::core::SerialDevice;
use serial::{SerialPortSettings, BaudRate};
use serial::unix::TTYPort;

use std::io::{BufRead, Error as IOError, Read};
use std::path::Path;
use std::{thread, process};
use std::time::Duration;

const MAX_BYTES_PER_CALL: usize = 128;

pub struct Cereal {
	tty: TTYPort,
	buf: Vec<u8>,
	index: usize,
}

impl Cereal {
	pub fn new(path: &str) -> Cereal {
		let path = Path::new(path);
		if path.exists() {
			println!("Please unplug the cable before continuing");
			while path.exists() {
				thread::sleep(Duration::from_secs(1));
			}
		}
		if !path.exists() {
			println!("Please plug the cable in now");
			while !path.exists() {
				thread::sleep(Duration::from_secs(1));
			}
		}

		let mut cereal = TTYPort::open(&path).unwrap_or_else(|e| {
			println!("Please check your permissions for accessing the serial port. {:?}", e);
			process::exit(0);
		});

		let mut settings = cereal.read_settings().unwrap();

		settings.set_baud_rate(BaudRate::Baud115200).unwrap();

		cereal.write_settings(&settings).unwrap();

		let mut cereal = Cereal {
			tty: cereal,
			buf: vec![0u8; MAX_BYTES_PER_CALL],
			index: 0,
		};

		println!("Loading initial read buffer");
		while cereal.index < MAX_BYTES_PER_CALL {
			let _ = cereal.fill_buf();
		}
		println!("Ready!");

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

		Ok(&self.buf[..self.index])
	}

	fn consume(&mut self, amt: usize) {
		for _ in 0..amt {
			self.buf.remove(0);
		}

		self.index -= amt;
	}
}
