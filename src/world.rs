
use crate::chunk::Chunk;
use crate::player::Player;
use crate::orb::Orb;

pub struct World {
	pub chunks: Vec<Chunk>,
	pub players: Vec<Player>,
	pub orbs: Vec<Orb>
}

impl World {
	pub fn new() -> Self {
		Self {
			chunks: Vec::new(),
			players: Vec::new(),
			orbs: Vec::new()
		}
	}
	pub fn unload_unused_chunks(&mut self) {
		let bfn = self.chunks.len();
		let mut i = 0;
		'chunkboi:
		loop {
			if i >= self.chunks.len() { break; }
			let x = self.chunks[i].x >> 7;
			let y = self.chunks[i].y >> 7;
			if self.chunks[i].modified {
				println!("saving chunk {} {}", x, y);
				self.chunks[i].save();
				self.chunks[i].modified = false;
			}
			for player in &self.players {
				if (x - (player.pos.x >> 7)).abs() <= 1 && (y - (player.pos.y >> 7)).abs() <= 1 {
					i += 1;
					continue 'chunkboi;
				}
			}
			self.chunks.remove(i);
		}
		println!("Amount of chunks was {} and is now {}", bfn, self.chunks.len());
	}
	fn get_chunk(&mut self, x: i32, y: i32) -> usize {
		assert!(x % 128 == 0);
		assert!(y % 128 == 0);
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
		let cid = self.get_chunk(x ^ incx, y ^ incy);
		self.chunks[cid].modified = true;
		self.chunks[cid].tiles[(incx + incy * 128) as usize] = tile;
	}
	pub fn get_tile(&mut self, x: i32, y: i32) -> u8 {
		let incx = x & 127;
		let incy = y & 127;
		let cid = self.get_chunk(x ^ incx, y ^ incy);
		self.chunks[cid].tiles[(incx + incy * 128) as usize]
	}
}

// my dad is being peer pressured into learning a country song called "pretty good at drinking beer"
// cod, carp, and the holy mackerel
