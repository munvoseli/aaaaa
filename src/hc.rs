//pub mod hc;
use crate::World;
use crate::Player;

pub fn handle_message(v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let mut i: usize = 0; // client might crash server without command size checks
	let mut sc: Vec<u8> = Vec::new();
	loop {
		let code: u8 = v[i];
		i += 1;
		let mut rv = match code {
		0 => hc_get_tiles(&mut i, v, world),
		1 => hc_set_loc(&mut i, v, world, pid),
		2 => hc_break(&mut i, v, world, pid),
		3 => hc_get_entities(world, pid),
		4 => hc_place_tile(&mut i, v, world, pid),
		5 => hc_get_inventory(world, pid),
		_ => Vec::new()
		};
		sc.append(&mut rv);
		if i == v.len() {
			break;
		}
	}
	return sc;
}

fn read_as_int(i: usize, v: &Vec<u8>) -> i32 {
//	(((v[i] as i32) >> 6) - 1) * (((v[i] & 127) as i32) << 24) | ((v[i + 1] as i32) << 16) | ((v[i + 2] as i32) << 8) | (v[i + 3] as i32)
	i32::from_be_bytes([v[i], v[i+1], v[i+2], v[i+3]])
}
//fn int_into_vec(x: i32) -> Vec<u8> {
//	vec!(((x >> 24) & 8) as u8, ((x >> 16) & 8) as u8, ((x >> 8) & 8) as u8, (x & 8) as u8)
//}
fn append_int(v: &mut Vec<u8>, x: i32) {
	let bytes = x.to_be_bytes();
	v.push(bytes[0]);
	v.push(bytes[1]);
	v.push(bytes[2]);
	v.push(bytes[3]);
//	v.push(((x >> 24)      ) as u8);
//	v.push(((x >> 16) & 255) as u8);
//	v.push(((x >>  8) & 255) as u8);
//	v.push( (x        & 255) as u8);
}

fn hc_get_tiles(i: &mut usize, v: &Vec<u8>, world: &mut World) -> Vec<u8> {
//	println!("{} {} {} {}", (v[*i] as i32) << 24, (v[*i + 1] as i32) << 16, (v[*i + 2] as i32) << 8, (v[*i + 3] as i32));
	let x: i32 = read_as_int(*i, v);
	*i += 4;
	let y: i32 = read_as_int(*i, v);
	*i += 4;
	let mut r: i32 = v[*i] as i32;
	if r > 64 { r = 63; }
	if r < 0 { r = 0; }
	*i += 1;
	let mut rv: Vec<u8> = Vec::new();
	rv.push(0);
//	println!("{} {}", x, r);
	append_int(&mut rv, x);
	append_int(&mut rv, y);
	rv.push((r & 255) as u8);
	for wy in (y-r)..=(y+r) {
	for wx in (x-r)..=(x+r) {
		rv.push(world.get_tile(wx, wy));
	}
	}
	rv
}

fn hc_set_loc(i: &mut usize, v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let ox;
	let oy;
	{
		let player = world.players.get_mut(&pid).unwrap();
		ox = player.pos.x;
		oy = player.pos.y;
		player.pos.x = read_as_int(*i, v);
		*i += 4;
		player.pos.y = read_as_int(*i, v);
		*i += 4;
		player.pos.subx = v[*i];
		*i += 1;
		player.pos.suby = v[*i];
		*i += 1;
	}
	Player::do_move_checks(ox, oy, world, pid);
	Vec::new()
}

fn item_from_tile(t: u8) -> u8 {
	let nt = t & 0xfc;
	if nt == 0x88 || nt == 0x90 || nt == 0x94 { return nt; }
	t
}

fn hc_break(i: &mut usize, v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let x = read_as_int(*i, v); *i += 4;
	let y = read_as_int(*i, v); *i += 4;
	let t = world.get_tile(x, y);
	world.players.get_mut(&pid).unwrap().inventory.put(item_from_tile(t));
	world.set_tile(x, y, 0x80);
	Vec::new()
}

fn hc_get_entities(world: &mut World, pid: usize) -> Vec<u8> {
	let mut rv: Vec<u8> = Vec::new();
	rv.push(1);
	rv.push((world.players.len() - 1 + world.orbs.len()) as u8);
	for p in &world.players {
		if pid == *p.0 { continue; }
		let p = p.1;
		rv.push(0);
		append_int(&mut rv, p.pos.x);
		append_int(&mut rv, p.pos.y);
		rv.push(p.pos.subx);
		rv.push(p.pos.suby);
	}
	for orb in &world.orbs {
		rv.push(1);
		append_int(&mut rv, orb.pos.x);
		append_int(&mut rv, orb.pos.y);
		rv.push(orb.pos.subx);
		rv.push(orb.pos.suby);
		rv.push(orb.flavor);
	}
	rv
}

fn hc_place_tile(i: &mut usize, v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let x = read_as_int(*i, v); *i += 4;
	let y = read_as_int(*i, v); *i += 4;
	let tnow = world.get_tile(x, y);
	if tnow == 0x80 {
		let t = v[*i]; *i += 1;
		let success = world.players.get_mut(&pid).unwrap().inventory.rem(item_from_tile(t)); // may have bad things with directional
		if success {
			world.set_tile(x, y, t);
		}
	}
	Vec::new()
}

fn hc_get_inventory(world: &mut World, pid: usize) -> Vec<u8> {
	let mut rv: Vec<u8> = Vec::new();
	rv.push(4);
	rv.push(world.players[&pid].inventory.items.len() as u8);
	for boi in &world.players[&pid].inventory.items {
		append_int(&mut rv, boi.0 as i32);
		rv.push(boi.1);
	}
	rv
}
