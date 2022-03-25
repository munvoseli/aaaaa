use crate::player::Entpos;

pub struct Vel {
	pub x: i32,
	pub y: i32
}

pub struct Orb {
	pub flavor: u8,
	pub pos: Entpos,
	pub v: Vel,
	pub age: u32
}

impl Orb {
	pub fn step(&mut self) {
	}
}

