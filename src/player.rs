use crate::World;
use crate::Orb;

pub struct Entpos {
	pub x: i32,
	pub y: i32,
	pub subx: u8,
	pub suby: u8,
}

impl Entpos {
//	pub fn new() -> Self {
//		Self {
//			x: 0,
//			y: 0,
//			subx: 0,
//			suby: 0
//		}
//	}
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
	pub pos: Entpos,
	pub comque: Vec<u8>
}

impl Player {
	fn idkwhat0(a: i32, b: i32, r: i32) -> i32 {
		if b < a { b - r }
		else { std::cmp::max(a + r + 1, b - r) }
	} // i literally have no idea what these do, i just worked them out
	fn idkwhat2(a: i32, b: i32, r: i32) -> i32 { // on paper
		if b < a { std::cmp::min(a - r - 1, b + r) }
		else { b + r } // in your head there's empty space
	}
	fn get_new_rects(ax: i32, ay: i32, bx: i32, by: i32) -> Vec<(i32,i32,i32,i32)> {
		let r = 5; // and there's a goat goat goat
		let mut waveborn: Vec<(i32, i32, i32, i32)> = Vec::new();
		if ax != bx { // livin' in my head
			let x0 = Self::idkwhat0(ax, bx, r);
			let x2 = Self::idkwhat2(ax, bx, r);
			let y0 = std::cmp::max(by - r, ay - r);
			let y2 = std::cmp::min(by + r, ay + r);
			waveborn.push((x0, y0, x2, y2));
		}
		if ay != by {
			let x0 = bx - r;
			let x2 = bx + r;
			let y0 = Self::idkwhat0(ay, by, r);
			let y2 = Self::idkwhat2(ay, by, r);
			waveborn.push((x0, y0, x2, y2));
		}
		waveborn
	}
	pub fn do_move_checks(ox: i32, oy: i32, world: &mut World, pid: usize) {
		let wavedied = Self::get_new_rects(world.players[pid].pos.x, world.players[pid].pos.y, ox, oy);
		for r in wavedied {
		for y in r.1..=r.3 {
		for x in r.0..=r.2 {
			let t = world.get_tile(x, y);
			if (t & 0xfc) == 0x88 {
//				println!("{} {} {:#04x}", r.1, r.3, t);
				world.orbs.push(Orb::new(x, y, t ^ 0x88 ^ 2, 0));
			}
		}}}
		let waveborn = Self::get_new_rects(ox, oy, world.players[pid].pos.x, world.players[pid].pos.y);
		for r in waveborn {
		for y in r.1..=r.3 {
		for x in r.0..=r.2 {
			let t = world.get_tile(x, y);
			if (t & 0xfc) == 0x88 {
//				println!("{} {} {:#04x}", r.1, r.3, t);
				world.orbs.push(Orb::new(x, y, t ^ 0x88, 0));
			}
		}}}
	}
}
