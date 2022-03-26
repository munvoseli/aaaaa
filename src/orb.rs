use crate::player::Entpos;
use crate::world::World;

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
	pub fn step(world: &mut World) {
		let mut i = 0;
		loop {
			if i >= world.orbs.len() { break; }
			let mut orb = &mut world.orbs[i];
			if orb.age >= 200 {
				world.orbs.remove(i);
				continue;
			}
			orb.pos.addsub(orb.v.x, orb.v.y);
			orb.age += 1;
			let x = orb.pos.x;
			let y = orb.pos.y;
			drop(orb);
			let t = world.get_tile(x, y);
			let mut orb = &mut world.orbs[i];
			if t >= 0x90 && t < 0x98 {
				let tf = ((t >> 2) & 1) + 1;
				if orb.flavor == 0 || tf == orb.flavor {
					let s = (t as i32 & 2) - 1;
					let dx = (t as i32 & 1) * -s;
					let dy = ((t as i32 & 1) ^ 1) * s;
					orb.v.x = dx * 32;
					orb.v.y = dy * 32;
					orb.pos.subx = (128 + dx * 112) as u8;
					orb.pos.suby = (128 + dy * 112) as u8;
					if orb.flavor == 0 {
						orb.flavor = tf;
						orb.age = 0;
					}
				}
			}
			i += 1;
		}
	}
}

