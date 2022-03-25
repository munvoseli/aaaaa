

pub struct Entpos {
	pub x: i32,
	pub y: i32,
	pub subx: u8,
	pub suby: u8
}

impl Entpos {
	pub fn new() -> Self {
		Self {
			x: 0,
			y: 0,
			subx: 0,
			suby: 0
		}
	}
	pub fn addsub(&mut self, mut x: i32, mut y: i32) {
		x += self.subx as i32;
		y += self.suby as i32;
		self.subx = (x & 255) as u8;
		self.suby = (y & 255) as u8;
		self.x += x >> 8;
		self.y += y >> 8;
	}
}

pub struct Player {
	pub pos: Entpos
}

impl Player {
}
