use std::fs::File;
use std::io::{Write, Read};

pub struct Chunk {
	pub x: i32,
	pub y: i32,
	pub tiles: [u8; 128*128],
	pub modified: bool
}

impl Chunk {
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
		//	tiles = [0; 128 * 128];
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
