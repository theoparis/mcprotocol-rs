use mc_varint::{VarInt, VarIntWrite};
use std::io::Result;
use std::io::Write;
use std::net::TcpStream;
use uuid::Uuid;

pub struct Packet {
	pub packet_id: i32,
	pub payload: Vec<u8>,
}

impl Packet {
	pub fn send(&self, stream: &mut TcpStream) -> Result<()> {
		let mut content: &[u8] = &self.payload;
		let packet_id_length: i32 = 1;
		stream.write_var_int(VarInt::from(
			content.len() as i32 + packet_id_length,
		))?;
		stream.write_var_int(VarInt::from(self.packet_id))?;
		stream.write(&mut content)?;
		stream.flush()
	}

	pub fn write_string(&mut self, value: String) -> Result<()> {
		self.write_varint(VarInt::from(value.len() as i32))?;
		self.payload.write(value.as_bytes())?;
		Ok(())
	}

	pub fn write_varint(&mut self, value: VarInt) -> Result<()> {
		self.payload.write_var_int(value)
	}

	pub fn write_uuid(&mut self, value: Uuid) -> Result<()> {
		self.payload.write(value.as_bytes())?;
		Ok(())
	}
}
