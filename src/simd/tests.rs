use crate::simd::*;
use crate::BufCSV;

use std::error::Error;

#[bench]
fn simd_instructions(b: &mut test::Bencher) -> Result<(), Box<dyn Error>> {
	let bufcsv = BufCSV::new("testdata/drop.csv")?;
	let blocks = Telemetryx16Iter::from(bufcsv.into_iter()).collect::<Vec<_>>();

	let into_radians = f32x16::splat(std::f32::consts::PI / 180.);

	b.iter(|| {
		let _ = test::black_box(
			blocks.clone().into_iter()
				.map(|mut block| {
					block.theta_x *= into_radians;
					block.theta_y *= into_radians;
					block.theta_z *= into_radians;

					block
				})
		);
	});

	Ok(())
}

#[bench]
fn standard_optimization(b: &mut test::Bencher) -> Result<(), Box<dyn Error>> {
	let bufcsv = BufCSV::new("testdata/drop.csv")?;
	let units = bufcsv.into_iter().collect::<Vec<_>>();

	let into_radians = std::f32::consts::PI / 180.;

	b.iter(|| {
		let _ = test::black_box(
			units.clone().into_iter()
				.map(|mut unit| {
					unit.theta_x *= into_radians;
					unit.theta_y *= into_radians;
					unit.theta_z *= into_radians;

					unit
				})
		);
	});

	Ok(())
}
