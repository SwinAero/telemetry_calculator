use crate::TelemetryDataUnit;
use crate::simd::iter::{F32x16Iter, CalcMode};
use std::f32::consts::PI;
use std::iter;

pub fn radians(tdus: &mut Vec<TelemetryDataUnit>) {
	let (tx, ty, tz) = {
		let (tx, ty): (Vec<_>, Vec<_>) = tdus.iter()
			.map(|tdu| (tdu.theta_x, tdu.theta_y))
			.unzip();
		let tz: Vec<_> = tdus.iter().map(|tdu| tdu.theta_z).collect();

		let tx = F32x16Iter::from(tx.into_iter(), iter::repeat(PI / 180.), CalcMode::Multiply);
		let ty = F32x16Iter::from(ty.into_iter(), iter::repeat(PI / 180.), CalcMode::Multiply);
		let tz = F32x16Iter::from(tz.into_iter(), iter::repeat(PI / 180.), CalcMode::Multiply);

		(tx, ty, tz)
	};

	tdus.iter_mut()
		.zip(tx)
		.zip(ty
			.zip(tz)
		)
		.for_each(|((tdu, tx), (ty, tz))| {
			tdu.theta_x = tx;
			tdu.theta_y = ty;
			tdu.theta_z = tz;
		});
}

pub fn normalize(tdu: TelemetryDataUnit) -> TelemetryDataUnit {
	unimplemented!()
}
