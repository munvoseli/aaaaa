use std::fs::File;
use std::io::{Write, Read};
use rand::distributions::{Distribution, Uniform};
use rand::SeedableRng;
pub struct Chunk {
	pub x: i32,
	pub y: i32,
	pub tiles: [u8; 128*128],
	pub modified: bool
}


// https://stackoverflow.com/questions/664014/what-integer-hash-function-are-good-that-accepts-an-integer-hash-key
fn coord_hash(x: i32, y: i32, m: u32) -> u32 {
	#![allow(arithmetic_overflow)]
	let mut h: u32 = x as u32;
	h ^= (y as u32) << 16;
	h ^= (y as u32) >> 16;
	h ^= m;
	h = u32::wrapping_mul((h >> 16) ^ h, 0x45d9f3b);
	h = u32::wrapping_mul((h >> 16) ^ h, 0x45d9f3b);
	h = (h >> 16) ^ h;
	return h;
}


impl Chunk {
//	fn generate_rowcol(data: &mut [i32; 129*129], n: usize, iscol: bool, die: rand::distributions::Uniform<i32>) {
//		let mul = if iscol { 129 } else { 0 };
//		let off = if iscol { n } else { 129 * n };
//		let width = 128;
//		loop {
//			if width == 1 { break; }
//			let g = 0;
//			loop {
//				if g == 128 { break; }
//				let v0 = data[mul * g + off];
//				let v2 = data[mul * (g+wdith) + off];
//				data[mul * (g+width/2) + off] = (v0 + v2) / 2 + die.sample(&mut rng);
//				g += width;
//			}
//			width /= 2;
//		}
//	}
//	fn get_bq_int_recur(x: i32, y: i32, seed: u32, v: &mut Vec<(i32, i32, i32)>) -> i32 {
////		println!("getting at {} {}", x, y);
//		if (x & 15) == 0 && (y & 15) == 0 {
//			return (coord_hash(x, y, seed) as i32) / 16;
//		}
//		let mut i = 0;
//		loop {
//			if i == v.len() { break; }
//			let vv = v[i];
//			if vv.0 == x && vv.1 == y {
//				return vv.2;
//			}
//			i += 1;
//		}
//		let bx = (u32::wrapping_add(x as u32 ^ u32::wrapping_sub(x as u32, 1), 1) >> 1) as i32;
//		let by = (u32::wrapping_add(y as u32 ^ u32::wrapping_sub(y as u32, 1), 1) >> 1) as i32;
////		println!("{} {}", bx, by);
//		let v00: i32;
//		let v02: i32;
//		let v20: i32;
//		let v22: i32;
//		if bx == by { // x
//			v00 = Self::get_bq_int_recur(x - bx, y - bx, seed, v);
//			v02 = Self::get_bq_int_recur(x - bx, y + bx, seed, v);
//			v20 = Self::get_bq_int_recur(x + bx, y - bx, seed, v);
//			v22 = Self::get_bq_int_recur(x + bx, y + bx, seed, v);
//		} else { // +
//			let b = if bx == 0 { by } else if by == 0 { bx } else { std::cmp::min(bx, by) }; 
//			v00 = Self::get_bq_int_recur(x     , y - b , seed, v);
//			v02 = Self::get_bq_int_recur(x - b , y     , seed, v);
//			v20 = Self::get_bq_int_recur(x + b , y     , seed, v);
//			v22 = Self::get_bq_int_recur(x     , y + b , seed, v);
//		}
//		let v11 = v00 / 4 + v02 / 4 + v20 / 4 + v22 / 4;
//		v.push((x, y, v11));
//		v11
//	}
	fn generate_noise_bq(cx: i32, cy: i32, seed: u32) -> [i32; 128*128] {
		const cw: i32 = 128;
		const bw: i32 = 128 * 3 + 1;
		let mut data: [i32; bw as usize * bw as usize] = [0; bw as usize * bw as usize]; // represents points on edges of tiles
		{ // fill set points
			for y in (0..=3*cw).step_by(cw as usize) {
			for x in (0..=3*cw).step_by(cw as usize) {
				data[(y * bw + x) as usize] = coord_hash(x + cx, y + cy, seed) as i32 / 16; // offset does not matter now
			}}
		}
		// loop de loop
		let mut l = cw / 2;
		loop {
			if l == 0 { break; }
			// diagonal
			for y in (cw-l..=2*cw+l).step_by(2*l as usize) {
			for x in (cw-l..=2*cw+l).step_by(2*l as usize) {
				let v0 = data[((y - l) * bw + x - l) as usize];
				let v1 = data[((y + l) * bw + x - l) as usize];
				let v2 = data[((y - l) * bw + x + l) as usize];
				let v3 = data[((y + l) * bw + x + l) as usize];
				data[(y * bw + x) as usize] = v0 / 4 + v1 / 4 + v2 / 4 + v3 / 4;
			}}
			// orthogonal a
			for y in (cw-l..=2*cw+l).step_by(2*l as usize) {
			for x in (cw..=2*cw).step_by(2*l as usize) {
				let v0 = data[((y    ) * bw + x - l) as usize];
				let v1 = data[((y + l) * bw + x    ) as usize];
				let v2 = data[((y - l) * bw + x    ) as usize];
				let v3 = data[((y    ) * bw + x + l) as usize];
				data[(y * bw + x) as usize] = v0 / 4 + v1 / 4 + v2 / 4 + v3 / 4;
			}}
			// orthogonal b
			for y in (cw..=2*cw).step_by(2*l as usize) {
			for x in (cw-l..=2*cw+l).step_by(2*l as usize) {
				let v0 = data[((y    ) * bw + x - l) as usize];
				let v1 = data[((y + l) * bw + x    ) as usize];
				let v2 = data[((y - l) * bw + x    ) as usize];
				let v3 = data[((y    ) * bw + x + l) as usize];
				data[(y * bw + x) as usize] = v0 / 4 + v1 / 4 + v2 / 4 + v3 / 4;
			}}
			l /= 2;
		}
		let mut td: [i32; 128*128] = [0; 128*128];
		let mut i = 0;
		for y in 0..128 {
		for x in 0..128 {
			let g = ((y + cw) * bw + (x + cw)) as usize;
			td[i] = data[g] / 4 + data[g+1] / 4 + data[g+(bw as usize)] / 4 + data[g+(bw as usize)+1] / 4;
			i += 1;
		}}
		td
	}
//	fn generate_noise_deux(cx: i32, cy: i32, seed: u32) -> [i32; 129*129] {
//		let mut weights: [i32; 129*129] = [0; 129*129]; //unsafe { std::mem::MaybeUninit::uninit().assume_init() };
//		let mut i = 0;
//			let mut v: Vec<(i32, i32, i32)> = Vec::new();
//		weights[0] = Self::get_bq_int_recur(cx + 11, cy + 11, seed, &mut v);
//		let mut gencalls = 0;
////		println!("{}", v.len());
//		for y in 0..129 {
//		for x in 0..129 {
//			let mut v: Vec<(i32, i32, i32)> = Vec::new();
//			weights[i] = Self::get_bq_int_recur(cx + x, cy + y, seed, &mut v);
//			gencalls += v.len();
//			i += 1;
//		}
//		}
//		println!("tiles calculated during gencalls: {}", gencalls);
////		weights[0] = coord_hash(cx, cy, seed) as i32;
////		weights[128] = coord_hash(cx + 128, cy, seed) as i32;
////		weights[128 * 129] = coord_hash(cx, cy + 128, seed) as i32;
////		weights[128 * 130] = coord_hash(cx + 128, cy + 128, seed) as i32;
////		let mut j = 64;
////		loop {
////			if j == 1 { break; }
////			let mut y = 0;
////			loop {
////			let mut x = 0;
////			loop {
////			//weights[j * 129 + j] = weights[
////			x += 2 * j;
////			}
////			y += 2 * j;
////			}
////			j /= 2;
////		}
////		for i in 0..129*129 {
////			weights[i] = coord_hash(x, y + (i as i32), seed) as i32;
////		}
//		weights
//	}
	fn generate_noise(range: std::ops::RangeInclusive<i32>, seed: u64) -> [i32; 129*129] {
		let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
//		let die = Uniform::from(127..=129);
		let mut weights: [i32; 129*129] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
		weights[0] = 0;
		weights[128] = 0;
		weights[128 * 129] = 0;
		weights[128 * 130] = 0;
		let die = Uniform::from(range);
		let mut width = 128;
		loop {
			let mut y = 0;
			loop {
			let mut x = 0;
			loop {
				let v00 = weights[y * 129 + x];
				let v20 = weights[y * 129 + x + width];
				let v02 = weights[(y + width) * 129 + x];
				let v22 = weights[(y + width) * 129 + x + width];

				let v10 = (v00 + v20) / 2 + die.sample(&mut rng);
				let v21 = (v20 + v22) / 2 + die.sample(&mut rng);
				let v01 = (v00 + v02) / 2 + die.sample(&mut rng);
				let v12 = (v02 + v22) / 2 + die.sample(&mut rng);
				let v11 = (v00 + v02 + v20 + v22) / 4 + die.sample(&mut rng);
				weights[(y + width/2) * 129 + x]           = v10;
				weights[(y + width/2) * 129 + x + width]   = v12;
				weights[y * 129 + x + width/2]             = v01;
				weights[(y + width) * 129 + x + width/2]   = v21;
				weights[(y + width/2) * 129 + x + width/2] = v11;
				x += width;
				if x == 128 { break; }
			}
			y += width;
			if y == 128 { break; }
			}
			width /= 2;
			if width == 1 {
				break;
			}
		}
		for _ in 0..4 {
		for y in 0..128 {
		for x in 0..128 {
			weights[y * 129 + x] =
				weights[y * 129 + x] / 4 +
				weights[y * 129 + x + 1] / 4 +
				weights[y * 129 + x + 129] / 4 +
				weights[y * 129 + x + 130] / 4;
		}
		}
		}
		weights
	}
	fn generate_new(cells: &mut [u8; 128*128], wx: i32, wy: i32) {
		let weights = Self::generate_noise_bq(wx, wy, 0);
		let colors = Self::generate_noise_bq(wx, wy, 1);
		let mut i = 0;
		for y in 0..128 {
		for x in 0..128 {
//			cells[i] = if (x - 64) * (x - 64) + (y - 64) * (y - 64) > 32 * 64 { 0x80 } else { 0x81 };
			let blockval = weights[y * 128 + x];
			cells[i] = if blockval > 0 {
				let mut val = colors[y * 128 + x];
				val /= 30000000;
				if val < -4 {
					val = -4;
				} else if val > 4 {
					val = 4;
				}
				val += 4;
				(0xa0 | val) as u8
			} else if blockval == 0 {
				0x82
			} else { 0x80 };
			i += 1;
		}
		}
	}
	pub fn load(x: i32, y: i32) -> Self {
		let f = File::open(format!("data/{}_{}.dat", x, y));
		let mut tiles: [u8; 128*128] = [0; 128*128];
		// can't initialize the array without it being initialized first
		// it would be great if there were an assert initialized
		if let Ok(mut file) = f {
			let mut buf: [u8; 9 + 128 * 128] = [1; 128*128+9];
			file.read(&mut buf).unwrap();
			for i in 0..128*128 {
				tiles[i] = buf[i + 9];
			}
		} else {
			Self::generate_new(&mut tiles, x, y);
		}
		Self {
			x: x, y: y,
			tiles: tiles,
			modified: false
		}
	}
	pub fn save(&self) {
		let mut buf: [u8; 9 + 128 * 128] = [0; 128*128+9];
		buf[1] = ((self.x >> 24) & 255) as u8; // store x and y
		buf[2] = ((self.x >> 16) & 255) as u8; // big endian
		buf[3] = ((self.x >>  8) & 255) as u8;
		buf[4] = ((self.x >>  0) & 255) as u8;
		buf[5] = ((self.y >> 24) & 255) as u8;
		buf[6] = ((self.y >> 16) & 255) as u8;
		buf[7] = ((self.y >>  8) & 255) as u8;
		buf[8] = ((self.y >>  0) & 255) as u8;
		for i in 0..128*128 {
			buf[i + 9] = self.tiles[i];
		}
		let mut f = File::create(format!("data/{}_{}.dat", self.x, self.y)).unwrap();
		f.write(&buf).unwrap();
	}
}
