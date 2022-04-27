use global_counter::generic::Counter;
use global_counter::primitive::exact::CounterI32;
use hecs::World;
use lazy_static::lazy_static;
use mc_varint::VarInt;
use mcprotocol::packet::Packet;
use quartz_nbt::NbtCompound;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Player {
	pub username: String,
}

static ID_COUNTER: CounterI32 = CounterI32::new(0);

lazy_static! {
	pub static ref WORLD: Mutex<World> = Mutex::new(World::new());
}

fn main() -> Result<()> {
	let listener = TcpListener::bind("0.0.0.0:25565").unwrap();

	for stream in listener.incoming() {
		let mut stream = stream.unwrap();
		let res = handle_connection(&mut stream);
		if let Err(err) = res {
			println!("{:?}", err);
		}
	}
	Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
	// Packet length
	let _ = mcprotocol::protocol::read_varint(stream);
	// Packet ID
	let packet_id = mcprotocol::protocol::read_varint(stream).unwrap();
	println!("Packet ID: {}", packet_id);
	match packet_id {
		0x00 => initializing_connection(stream)?,
		_ => (),
	}
	println!("------");
	Ok(())
}

fn initializing_connection(stream: &mut TcpStream) -> Result<()> {
	// Reading Next State value to determine if this is a login or ping attempt
	let protocol_version = mcprotocol::protocol::read_varint(stream).unwrap();
	println!("Protocol Version: {}", protocol_version);
	// Server address
	let server_addr = mcprotocol::protocol::read_string(stream).unwrap();
	println!("Server Address: {}", server_addr);
	let server_port = mcprotocol::protocol::read_ushort(stream).unwrap();
	println!("Server Port: {}", server_port);
	let next_state = mcprotocol::protocol::read_varint(stream).unwrap();
	println!("Next State: {}", next_state);
	match next_state {
		2 => client_login(stream)?,
		_ => handle_server_list_ping(stream)?,
	}
	Ok(())
}

pub fn client_login(stream: &mut TcpStream) -> Result<()> {
	// Login Start
	let client_username = mcprotocol::protocol::read_string(stream)?;
	println!("Client name:  {}", client_username);

	let id = ID_COUNTER.get();
	ID_COUNTER.inc();

	WORLD.lock().unwrap().spawn((
		id,
		Player {
			username: client_username.clone(),
		},
	));

	// // Set compression threshold to max
	let mut compression_packet = Packet {
		packet_id: 0x03,
		payload: Vec::new(),
	};
	compression_packet.write_varint(VarInt::from(65535))?;
	compression_packet.send(stream)?;

	// Login Success
	let mut login_packet = Packet {
		packet_id: 0x02,
		payload: Vec::new(),
	};
	login_packet.write_uuid(Uuid::new_v4())?;
	login_packet.write_string(client_username)?;
	login_packet.send(stream)?;
	// protocol::write_uuid(stream, Uuid::new_v4())?;
	// protocol::write_string(stream, client_username)?;
	// stream.flush()

	let mut join_game_packet = Packet {
		packet_id: 0x26,
		payload: Vec::new(),
	};

	// the entity id of the player
	join_game_packet.write_varint(VarInt::from(id));
	// whether or not it is hardcore mode
	join_game_packet.write_varint(VarInt::from(0));

	//let mut chunk_data_packet = Packet {
	//packet_id: 0x22,
	//payload: Vec::new(),
	//};
	//chunk_data_packet.write_varint(VarInt::from(0))?;
	//chunk_data_packet.write_varint(VarInt::from(0))?;
	// TODO: nbt?

	Ok(())
}

pub fn handle_server_list_ping(stream: &mut TcpStream) -> Result<()> {
	// Emptying the read buffer
	let mut buffer = [0; 1024];
	let _ = stream.read(&mut buffer)?;
	// Sending server infos
	let payload = json::parse(
		r#"
        {
            "description": {
                "text": "A minecraft server in Rust"
			},
            "players": {
                "max": 10,
                "online": 100000
            },
            "version": {
                "name": "1.18.2",
                "protocol": 758
            }
        }
    "#,
	)
	.unwrap()
	.dump();

	let mut packet = Packet {
		packet_id: 0x00,
		payload: Vec::new(),
	};
	packet.write_string(String::from(payload))?;
	packet.send(stream)?;

	packet.payload = Vec::new();
	stream.read(&mut buffer)?; // PING
	stream.write(&mut buffer)?; // PONG
	stream.flush()
}
