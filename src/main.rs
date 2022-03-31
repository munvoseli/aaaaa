
// half-goal:
// the server's only purpose was to mediate the clients' doings
// and the game logic is built into each client
// and it is the clients which spawn enemies and stuff
// where the server sees "entity of type 1:5", client a sees "gerald with 5 health" and client b sees "harold with 5 swords"
// client a might see the world as hexagons and a user of client b might see a user of client a doing something weird with gravity
// the server is only a bookkeeper for tiles and entities
// and each entity would have a client that was "in charge" of it
// but also:
// i just want to get my own project done

// apiforma arcana
// abbreviated aaaaa

// not to be confused with
// glyphica arcana
// arcane
// apionet
// application programming interface
// votgil

mod world;
mod chunk;
mod player;
mod orb;
use crate::world::World;
use crate::player::Player;
use crate::orb::Orb;
use crate::player::Entpos;
use crate::player::Inventory;
use crate::orb::Vel;
use std::sync::Arc;
use std::sync::Mutex;

type Amworld = std::sync::Arc<std::sync::Mutex<World>>;


#[tokio::main]
async fn main() {
	let h: i32 = -1;
	let g: u32 = h as u32;
	println!("{} {} {:#x} {:#x}", h, g, h, g);
	let world: Amworld = Arc::new(Mutex::new(World::new()));
	tick_loop(Arc::clone(&world));
	{
		let tw = Arc::clone(&world);
		let mut w = tw.lock().unwrap();
		(*w).set_tile(0, 0, 5);
	}
	net_loop(Arc::clone(&world));
}

fn tick_loop(world: Amworld) {
	tokio::spawn(async move {
		println!("tick handle");
		let mut i: u32 = 0;
		loop {
			let a = Arc::clone(&world);
			let mut tw = a.lock().unwrap();
			tick_step(&mut tw, i);
			drop(tw);
			std::thread::sleep(std::time::Duration::from_millis(50));
			i += 1;
			if i >= 100 {
				i = 0;
			}
		}
	});
}

fn tick_step(world: &mut World, ticki: u32) {
	if ticki == 0 {
		world.unload_unused_chunks();
		world.orbs.push(Orb {
			flavor: 0,
			pos: Entpos { x: 0, y: 0, subx: 128, suby: 240 },
			v: Vel { x: 0, y: 32 },
			age: 0
		});
	}
	Orb::step(world);
}

fn net_loop(world: Amworld) {
	let tcp_server = std::net::TcpListener::bind("0.0.0.0:3012").unwrap();
	println!("net_loop called");
	for stream in tcp_server.incoming() {
		let wrld = Arc::clone(&world);
		println!("new stream from tcp server");
		tokio::spawn(async move {
			let mut world = wrld.lock().unwrap();
			println!("spawned process for tcp server (mutex locked)");
			let tile = world.get_tile(0, 0) + 1;
			world.set_tile(0, 0, tile);
//			println!("world tile 0 0 {}", world.get_tile(0, 0));
			let callback = |_req: &tungstenite::handshake::server::Request,
			response: tungstenite::handshake::server::Response| {
				println!("new ws handshake");
				Ok(response)
			};
			let mut wsock = tungstenite::accept_hdr(
				stream.unwrap(), callback
				).unwrap();
			let pid = world.players.len();
			world.players.push(Player {pos: Entpos {x: 0, y: 0, subx: 128, suby: 128}, comque: Vec::new(), inventory: Inventory::new()});
			drop(world);
			loop {
				let msg = wsock.read_message();
				match msg {
					Err(_) => {
						println!("closing websocket");
						return;
					},
					_ => ()
				}
				let msg = msg.unwrap();
				match msg {
					tungstenite::Message::Text(h) => {
						println!("received text data: {}", h);
					},
					tungstenite::Message::Binary(v) => {
						let wrld = Arc::clone(&wrld);
						let mut world = wrld.lock().unwrap();
//						println!("received bin data of length {}", v.len());
						let now = std::time::Instant::now();
						let mut rv: Vec<u8> = handle_message(&v, &mut world, pid);
						rv.append(&mut world.players[pid].comque);
						wsock.write_message(tungstenite::Message::Binary(rv)).unwrap();
						let elapsed = now.elapsed().as_millis();
						if elapsed > 100 {
							println!("{} {}", elapsed, world.chunks.len());
						}
						drop(world);
					},
					_ => {
						println!("received non-bin non-text data");
					}
				}
			}
		});
	}
	println!("net_loop exiting");
}

fn handle_message(v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let mut i: usize = 0; // client might crash server without command size checks
	let mut sc: Vec<u8> = Vec::new();
	loop {
		let code: u8 = v[i];
		i += 1;
		let mut rv = match code {
		0 => hc_get_tiles(&mut i, v, world),
		1 => hc_set_loc(&mut i, v, world, pid),
		2 => hc_break(&mut i, v, world),
		3 => hc_get_entities(world, pid),
		4 => hc_place_tile(&mut i, v, world),
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
	*i += 1;
	let mut rv: Vec<u8> = Vec::new();
	rv.push(0);
//	println!("{} {}", x, r);
	append_int(&mut rv, x);
	append_int(&mut rv, y);
	rv.push((r & 255) as u8);
	for wy in (y-r)..=(y+r) {
	for wx in (x-r)..=(x+r) { // client can crash server with subtract overflow
		rv.push(world.get_tile(wx, wy));
	}
	}
	rv
}

fn hc_set_loc(i: &mut usize, v: &Vec<u8>, world: &mut World, pid: usize) -> Vec<u8> {
	let ox = world.players[pid].pos.x;
	let oy = world.players[pid].pos.y;
	world.players[pid].pos.x = read_as_int(*i, v);
	*i += 4;
	world.players[pid].pos.y = read_as_int(*i, v);
	*i += 4;
	world.players[pid].pos.subx = v[*i];
	*i += 1;
	world.players[pid].pos.suby = v[*i];
	*i += 1;
	Player::do_move_checks(ox, oy, world, pid);
	Vec::new()
}

fn hc_break(i: &mut usize, v: &Vec<u8>, world: &mut World) -> Vec<u8> {
	let x = read_as_int(*i, v); *i += 4;
	let y = read_as_int(*i, v); *i += 4;
	world.set_tile(x, y, 0x80);
	Vec::new()
}

fn hc_get_entities(world: &mut World, pid: usize) -> Vec<u8> {
	let mut rv: Vec<u8> = Vec::new();
	rv.push(1);
	rv.push((world.players.len() - 1 + world.orbs.len()) as u8);
	for i in 0..world.players.len() {
		if i == pid { continue; }
		rv.push(0);
		append_int(&mut rv, world.players[i].pos.x);
		append_int(&mut rv, world.players[i].pos.y);
		rv.push(world.players[i].pos.subx);
		rv.push(world.players[i].pos.suby);
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

fn hc_place_tile(i: &mut usize, v: &Vec<u8>, world: &mut World) -> Vec<u8> {
	let x = read_as_int(*i, v); *i += 4;
	let y = read_as_int(*i, v); *i += 4;
	let t = v[*i]; *i += 1;
	world.set_tile(x, y, t);
	Vec::new()
}
