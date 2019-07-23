use packed_simd::{f32x16, u32x16};

macro_rules! polymorph_into_simd_blocks {
	($name:ident, $ty:ty, $lanes:expr, $out_ty:ty) =>{
		pub fn $name<I: Iterator<Item=$ty>>(mut iter: I) -> Vec<$out_ty> {
			let mut blocks = vec![];

			'outer: loop {
				let mut a = [0 as $ty; $lanes];
				for i in 0..$lanes {
					if let Some(next) = iter.next() {
						a[i] = next;
					} else {
						if i != 0 {
							blocks.push(<$out_ty>::from_slice_unaligned(&a));
						}
						break 'outer;
					}
				}

				blocks.push(<$out_ty>::from_slice_unaligned(&a));
			}

			return blocks;
		}
	}
}

polymorph_into_simd_blocks!(into_f32x16_blocks, f32, 16, f32x16);
polymorph_into_simd_blocks!(into_u32x16_blocks, u32, 16, u32x16);
