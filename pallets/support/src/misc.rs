use sp_std::ops::AddAssign;
use sp_std::vec::Vec;

/// Cumulative sums for array
/// 
/// Taken from https://github.com/vandenheuvel/cumsum/blob/master/src/lib.rs
pub fn cumsum_array_owned<T, const N: usize>(mut x: [T; N]) -> [T; N]
where
	for<'r> T: AddAssign<&'r T>,
{
	for i in 1..N {
		let (read, write) = x.split_at_mut(i);
		write[0] += &read[i - 1];
	}
	x
}

/// Cumulative sums for vec
pub fn cumsum_owned<T>(mut x: Vec<T>) -> Vec<T>
where
	for<'r> T: AddAssign<&'r T>,
{
	for i in 1..x.len() {
		let (read, write) = x.split_at_mut(i);
		write[0] += &read[i - 1];
}
	x
}
