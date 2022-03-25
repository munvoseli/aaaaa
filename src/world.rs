
use crate::chunk::Chunk;
use crate::player::Player;

pub struct World {
	chunks: Vec<Chunk>,
	pub players: Vec<Player>
}

impl World {
	pub fn new() -> Self {
		Self {
			chunks: Vec::new(),
			players: Vec::new()
		}
	}
	fn get_chunk(&mut self, x: i32, y: i32) -> usize {
		for i in 0..self.chunks.len() {
			if self.chunks[i].x == x && self.chunks[i].y == y {
				return i;
			}
		}
		self.chunks.push(Chunk::load(x, y));
		return self.chunks.len() - 1;
	}
	pub fn set_tile(&mut self, x: i32, y: i32, tile: u8) {
		let incx = x & 127;
		let incy = y & 127;
		let cid = self.get_chunk(x ^ incy, y ^ incy);
		self.chunks[cid].modified = true;
		self.chunks[cid].tiles[(incx + incy * 128) as usize] = tile;
	}
	pub fn get_tile(&mut self, x: i32, y: i32) -> u8 {
		let incx = x & 127;
		let incy = y & 127;
		let cid = self.get_chunk(x ^ incy, y ^ incy);
		self.chunks[cid].tiles[(incx + incy * 128) as usize]
	}
}

// my dad is being peer pressured into learning a country song called "pretty good at drinking beer"
// cod, carp, and the holy mackerel
