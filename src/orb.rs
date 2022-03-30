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

pub fn dir_to_xyint(dir: u8) -> (i32, i32) {
	(
		[0, 1, 0, -1][dir as usize],
		[-1, 0, 1, 0][dir as usize]
	)
}

pub fn cast_from_runes(world: &mut World, flavor: u8, x: i32, y: i32) {
	for r in [(0,0),(0,-1),(1,0),(0,1),(-1,0)] {
		let t = world.get_tile(x + r.0, y + r.1);
		if (t | 3) != 0x8f { continue; }
		let card = t ^ 0x8c;
		let i = card * 3 + flavor;
//		let mut comque: Vec<u8> = Vec::new();
		match i {
		0 => { // gravity
			for player in &mut world.players {
				if (player.pos.x - x).abs() > 5 || (player.pos.y - y).abs() > 5 { continue; }
				player.comque.push(2);
				player.comque.push(0);
				player.comque.push(1);
			}
		},
		1 => { // gravity
			for player in &mut world.players {
				if (player.pos.x - x).abs() > 5 || (player.pos.y - y).abs() > 5 { continue; }
				player.comque.push(2);
				player.comque.push(0);
				player.comque.push(0);
			}
		},
		_ => {
			println!("unknown spell");
		}
		}
	}
}

impl Orb {
	pub fn new(x: i32, y: i32, dir: u8, fl: u8) -> Self {
		let v = dir_to_xyint(dir);
		Self {
			flavor: fl,
			pos: Entpos {
				x: x,
				y: y,
				subx: (128 + v.0 * 112) as u8,
				suby: (128 + v.1 * 112) as u8
			},
			v: Vel {
				x: v.0 * 128,
				y: v.1 * 128
			},
			age: 0
		}
	}
	pub fn step(world: &mut World) {
		let mut i = 0;
		loop {
			if i >= world.orbs.len() { break; }
			let mut orb = &mut world.orbs[i];
			if orb.age >= 100 {
				world.orbs.remove(i);
				continue;
			}
			orb.pos.addsub(orb.v.x, orb.v.y);
			orb.age += 1;
			let x = orb.pos.x;
			let y = orb.pos.y;
			drop(orb);
			let t = world.get_tile(x, y);
			if t >= 0x90 && t < 0x98 {
				let mut orb = &mut world.orbs[i];
				let tf = ((t >> 2) & 1) + 1;
				if orb.flavor == 0 || tf == orb.flavor {
					let s = (t as i32 & 2) - 1;
					let dx = (t as i32 & 1) * -s;
					let dy = ((t as i32 & 1) ^ 1) * s;
					orb.v.x = dx * 128;
					orb.v.y = dy * 128;
					orb.pos.subx = (128 + dx * 112) as u8;
					orb.pos.suby = (128 + dy * 112) as u8;
					if orb.flavor == 0 {
						orb.flavor = tf;
						orb.age = 0;
					}
				}
			} else if (t | 1) == 0x83 {
				let nt =
				if world.orbs[i].flavor == 1 { 0x82 }
				else { 0x83 };
				world.set_tile(x, y, nt);
			} else if (t | 3) == 0x8f {
				let flava = world.orbs[i].flavor;
				cast_from_runes(world, flava, x, y);
				world.orbs[i].age = 100;
			}
			i += 1;
		}
	}
}

