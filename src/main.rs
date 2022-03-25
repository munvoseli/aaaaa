
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
use crate::world::World;
use crate::player::Player;
use std::sync::Arc;
use std::sync::Mutex;

type Amworld = std::sync::Arc<std::sync::Mutex<World>>;


#[tokio::main]
async fn main() {
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
		loop {
			println!("  begin tick loop");
			let a = Arc::clone(&world);
			let mut tw = a.lock().unwrap();
			println!("  got lock");
			tick_step(&mut tw);
			drop(tw);
			std::thread::sleep(std::time::Duration::from_millis(5000));
			println!("  end tick loop");
		}
	});
}

fn tick_step(world: &mut World) {
	world.unload_unused_chunks();
}

fn net_loop(world: Amworld) {
	let tcp_server = std::net::TcpListener::bind("127.0.0.1:3012").unwrap();
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
			world.players.push(Player {x: 0, y: 0, subx: 128, suby: 128});
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
						let rv: Vec<u8> = handle_message(&v, &mut world, pid);
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
	world.players[pid].x = read_as_int(*i, v);
	*i += 4;
	world.players[pid].y = read_as_int(*i, v);
	*i += 4;
	world.players[pid].subx = v[*i];
	*i += 1;
	world.players[pid].suby = v[*i];
	*i += 1;
	Vec::new()
}
