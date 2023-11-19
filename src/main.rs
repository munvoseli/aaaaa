
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
mod hc;

use crate::world::World;
use crate::player::Player;
use crate::orb::Orb;
use crate::player::Entpos;
use crate::player::Inventory;
use crate::hc::handle_message;
//use crate::orb::Vel;
use std::sync::Arc;

type Amworld = std::sync::Arc<std::sync::Mutex<World>>;


#[tokio::main]
async fn main() {
	let h: i32 = -1;
	let g: u32 = h as u32;
	println!("{} {} {:#x} {:#x}", h, g, h, g);
	let world: Amworld = Arc::new(std::sync::Mutex::new(World::new()));
	tick_loop(Arc::clone(&world));
	{
		let tw = Arc::clone(&world);
		let mut w = tw.lock().unwrap();
		(*w).set_tile(0, 0, 5);
	}
	start_server(Arc::clone(&world)).await;
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
//		world.orbs.push(Orb {
//			flavor: 0,
//			pos: Entpos { x: 0, y: 0, subx: 128, suby: 240 },
//			v: Vel { x: 0, y: 32 },
//			age: 0
//		});
	}
	Orb::step(world);
}


async fn start_server(world: Amworld) {
	println!("start_server");
	ss::serve_blocking(3054, move | req: &mut hyper::Request<hyper::body::Incoming> | {
		use hyper::Response;
		use http_body_util::Full;
		use hyper::body::Bytes;
		let uri = &req.uri().path()[1..];
		let world = Arc::clone(&world);
		println!("main");
		match uri {
			"webs" => {
				let pid = {
					let world = Arc::clone(&world);
					let pid = {
						let mut world = world.lock().unwrap();
						let pid = world.players.len();
						world.players.push(Player {pos: Entpos {x: 0, y: 0, subx: 128, suby: 128}, comque: Vec::new(), inventory: Inventory::new()});
						pid
					};
					pid
				};
				let world = Arc::clone(&world);
				return ss::Potato::WebSocketHandler(Box::new(move |msg| {
					let world = Arc::clone(&world);
					if let tungstenite::Message::Binary(v) = msg {
							let world = Arc::clone(&world);
							let mut world = world.lock().unwrap();
							let now = std::time::Instant::now();
							let mut rv: Vec<u8> = handle_message(&v, &mut world, pid);
							rv.append(&mut world.players[pid].comque);
							//wsock.write(tungstenite::Message::Binary(rv)).unwrap();
							let elapsed = now.elapsed().as_millis();
							if elapsed > 100 {
								println!("{} {}", elapsed, world.chunks.len());
							}
							let v2 = hc::handle_message(&v, &mut world, pid);
							return Some(tungstenite::Message::Binary(v2));
					}
					None
				}));
			},
			uri => {
				let uri = if uri.len() < 2 { "index.html" } else { uri };
				println!("New path: {}", uri);
				let maybefile = std::fs::File::open(format!("html/{}", uri));
				match maybefile {
					Ok(mut file) => {
						let mut buf = Vec::new();
						std::io::Read::read_to_end(&mut file, &mut buf).unwrap();
						if &uri[uri.len()-2..] == "js" {
							ss::Potato::HTTPResponse(Response::builder()
							.status(200)
							.header("Content-Type", "application/javascript")
							.body(Full::new(Bytes::from(buf))).unwrap()
							)
						} else {
							ss::Potato::HTTPResponse(Response::new(Full::new(Bytes::from(buf))))
						}
					},
					Err(_) => {
						ss::Potato::HTTPResponse(Response::builder()
						.status(404)
						.body(Full::new(Bytes::from("404 Eroor"))).unwrap())
					}
				}
			}
		}
	}).await;
}

