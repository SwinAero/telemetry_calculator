use packed_simd::{f32x16, u32x16};
use std::vec::IntoIter;

pub enum CalcMode {
	Multiply,
	Divide,
	Add,
	Subtract,
}

impl CalcMode {
	pub fn op(&self, a: f32x16, b: f32x16) -> f32x16 {
		match self {
			&CalcMode::Multiply => a * b,
			&CalcMode::Divide => a / b,
			&CalcMode::Add => a + b,
			&CalcMode::Subtract => a - b,
		}
	}
}

pub struct F32x16Iter<I, J> {
	a: I,
	b: J,
	out_buf: IntoIter<f32>,
	last_buf: bool,
	mode: CalcMode,
}

impl<I, J> F32x16Iter<I, J>
	where I: Iterator<Item=f32>,
		  J: Iterator<Item=f32> {
	pub fn from(a: I, b: J, mode: CalcMode) -> Self {
		F32x16Iter {
			a,
			b,
			out_buf: vec![].into_iter(),
			last_buf: false,
			mode,
		}
	}

	fn next_buf(&mut self) {
		let mut a_buf = [0f32; 16];
		let mut b_buf = [0f32; 16];

		let mut lanes = 16;
		for i in 0..16 {
			if let (Some(a), Some(b)) = (self.a.next(), self.b.next()) {
				a_buf[i] = a;
				b_buf[i] = b;
			} else {
				lanes = i;
				break;
			}
		}

		if lanes < 16 {
			self.last_buf = true;
			return;
		}

		let mut out = vec![0f32; 16];
		self.mode.op(f32x16::from_slice_unaligned(&a_buf), f32x16::from_slice_unaligned(&b_buf)).write_to_slice_unaligned(&mut out[..]);

		self.out_buf = out.into_iter();
	}
}

impl<I, J> Iterator for F32x16Iter<I, J>
	where I: Iterator<Item=f32>,
		  J: Iterator<Item=f32> {
	type Item = f32;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.out_buf.next();

		if next.is_none() && !self.last_buf {
			self.next_buf();
			self.next()
		} else {
			next
		}
	}
}
