use std::fs::File;
use std::io::{Write, Read};
use rand::distributions::{Distribution, Uniform};
pub struct Chunk {
	pub x: i32,
	pub y: i32,
	pub tiles: [u8; 128*128],
	pub modified: bool
}


impl Chunk {
	fn generate_new(cells: &mut [u8; 128*128]) {
		let mut rng = rand::thread_rng();
//		let die = Uniform::from(127..=129);
		let mut i = 0;
		let mut weights: [u8; 129*129] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
		weights[0] = 128;
		weights[128] = 128;
		weights[128 * 129] = 128;
		weights[128 * 130] = 128;
		let die = Uniform::from(-5..=5);
		let mut width = 128;
		loop {
			let mut y = 0;
			loop {
			let mut x = 0;
			loop {
				let v00 = weights[y * 129 + x] as i16;
				let v20 = weights[y * 129 + x + width] as i16;
				let v02 = weights[(y + width) * 129 + x] as i16;
				let v22 = weights[(y + width) * 129 + x + width] as i16;

				let v10 = (v00 + v20) / 2 + die.sample(&mut rng) as i16;
				let v21 = (v20 + v22) / 2 + die.sample(&mut rng) as i16;
				let v01 = (v00 + v02) / 2 + die.sample(&mut rng) as i16;
				let v12 = (v02 + v22) / 2 + die.sample(&mut rng) as i16;
				let v11 = (v00 + v02 + v20 + v22) / 4 + die.sample(&mut rng) as i16;
				weights[(y + width/2) * 129 + x]           = v10 as u8;
				weights[(y + width/2) * 129 + x + width]   = v12 as u8;
				weights[y * 129 + x + width/2]             = v01 as u8;
				weights[(y + width) * 129 + x + width/2]   = v21 as u8;
				weights[(y + width/2) * 129 + x + width/2] = v11 as u8;
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
		for _ in 0..3 {
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
		for y in 0..128 {
		for x in 0..128 {
//			cells[i] = if (x - 64) * (x - 64) + (y - 64) * (y - 64) > 32 * 64 { 0x80 } else { 0x81 };
			cells[i] = if weights[y * 129 + x] > 124 { 0x81 } else { 0x80 };
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
			Self::generate_new(&mut tiles);
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
