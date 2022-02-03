use crate::byte_reader::ByteReader;
use crate::structure::*;
use std::io;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum MqttPacket {
    Connect(ConnectPacket),
    Connack(ConnackPacket),
    Subscribe(SubscribePacket),
    Suback(SubackPacket),
    Publish(PublishPacket),
    Puback(ConfirmationPacket),
    Pubrec(ConfirmationPacket),
    Pubrel(ConfirmationPacket),
    Pubcomp(ConfirmationPacket),
    Unsubscribe(UnsubscribePacket),
    Unsuback(UnsubackPacket),
    Pingreq,
    Pingresp,
    Disconnect(DisconnectPacket),
    Auth(AuthPacket),
}

impl MqttPacket {
    /// Encodes any MqttPacket
    ///MqttPacket::Suback(packet)
    /// # Examples
    ///
    /// ```
    /// use mqtt_packet_3_5::*;
    /// let packet = MqttPacket::Pingreq;
    /// assert_eq!(Ok(vec![
    ///     192, 0, // Header
    /// ]), packet.encode(5));
    ///
    ///
    /// ```
    ///
    /// # Connect example
    ///
    /// ```
    /// // need to import Packet trait in order to get .encode() to work
    /// use mqtt_packet_3_5::{ConnectPacket, Packet, Protocol};
    /// let packet = ConnectPacket {
    ///     protocol_id: Protocol::MQIsdp,
    ///     protocol_version: 3,
    ///     keep_alive: 30,
    ///     clean_session: false,
    ///     user_name: None,
    ///     password: None,
    ///     will: None,
    ///     client_id: "test".to_string(),
    ///     properties: None,
    /// };
    /// let buf = vec![
    ///     16, 18, // Header
    ///     0, 6, // Protocol ID length
    ///     77, 81, 73, 115, 100, 112, // Protocol ID
    ///     3,   // Protocol version
    ///     0,   // Connect flags
    ///     0, 30, // Keepalive
    ///     0, 4, // Client ID length
    ///     116, 101, 115, 116, // Client ID
    /// ];
    /// assert_eq!(Ok(buf), packet.encode(3)); // encode as v3
    /// ```
    ///
    pub fn encode(self, protocol_version: u8) -> Res<Vec<u8>> {
        match self {
            MqttPacket::Puback(packet)
            | MqttPacket::Pubrec(packet)
            | MqttPacket::Pubrel(packet)
            | MqttPacket::Pubcomp(packet) => packet.encode(protocol_version),
            MqttPacket::Suback(packet) => packet.encode(protocol_version),
            MqttPacket::Subscribe(packet) => packet.encode(protocol_version),
            MqttPacket::Publish(packet) => packet.encode(protocol_version),
            MqttPacket::Connect(packet) => packet.encode(protocol_version),
            MqttPacket::Connack(packet) => packet.encode(protocol_version),
            MqttPacket::Unsubscribe(packet) => packet.encode(protocol_version),
            MqttPacket::Unsuback(packet) => packet.encode(protocol_version),
            MqttPacket::Disconnect(packet) => packet.encode(protocol_version),
            MqttPacket::Pingreq => PingreqPacket {}.encode(protocol_version),
            MqttPacket::Pingresp => PingrespPacket {}.encode(protocol_version),
            MqttPacket::Auth(packet) => packet.encode(protocol_version),
        }
    }
}

pub struct PacketDecoder<R: io::Read> {
    pub reader: ByteReader<R>,
}

impl<R: io::Read> PacketDecoder<R> {
    pub fn new(reader: ByteReader<R>) -> PacketDecoder<R> {
        PacketDecoder { reader }
    }

    /// Creates a new decoder and binds it to a stream
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_stream(io::Cursor::new(vec![192, 0])); // pingreq
    /// while decoder.has_more() {
    ///     decoder.decode_packet(5); // will parse packets of version 5
    /// }
    ///
    ///
    /// ```
    pub fn from_stream(src: R) -> PacketDecoder<R> {
        PacketDecoder::new(ByteReader::new(io::BufReader::new(src)))
    }

    /// Creates a new decoder and binds it to a BufReader
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// let buf = io::BufReader::new(io::Cursor::new(vec![192, 0])); // pingreq
    /// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_bufreader(buf);
    /// while decoder.has_more() {
    ///     decoder.decode_packet(5); // will parse packets of version 5
    /// }
    ///
    ///
    /// ```
    pub fn from_bufreader(buf: io::BufReader<R>) -> PacketDecoder<R> {
        PacketDecoder::new(ByteReader::new(buf))
    }

    /// Decodes MQTT messages from an underlying readable
    ///
    /// If an error happens the decoder tries to get the packet length (variable length in in position 1-4)
    /// and discard `length` bytes. It's up to the user of this crate to close connections/streams
    /// that deliver invalid data if necessary
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// let buf = io::BufReader::new(io::Cursor::new(vec![
    ///        16, 18, // Header
    ///        0, 6, // Protocol ID length
    ///        77, 81, 73, 115, 100, 112, // Protocol ID
    ///        3,   // Protocol version
    ///        0,   // Connect flags
    ///        0, 30, // Keepalive
    ///        0, 4, // Client ID length
    ///        116, 101, 115, 116, // Client ID
    ///        // new packet here
    ///        192, 0
    /// ])); // pingreq
    /// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_bufreader(buf);
    /// let mut protocol_version = 5;
    /// while decoder.has_more() {
    ///     let msg = decoder.decode_packet(protocol_version); // will parse packets of version 5
    ///     // set the protocol version for a client on a stream
    ///     if let Ok(mqtt_packet_3_5::MqttPacket::Connect(packet)) = msg {
    ///         protocol_version = packet.protocol_version;
    ///     }
    /// }
    ///
    /// ```
    pub fn decode_packet(&mut self, protocol_version: u8) -> Res<MqttPacket> {
        let (length, fixed) = self.reader.read_header()?;
        let dec = self.decode_by_type(fixed, length, protocol_version);
        if dec.is_err() {
            // TODO: this should probably return an Error that indicates some
            // critical failure
            self.reader.consume()?;
        }
        self.reader.reset_limit();
        dec
    }

    pub fn has_more(&mut self) -> bool {
        self.reader.has_more()
    }

    fn decode_by_type(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<MqttPacket> {
        // let reader = self.reader.take(length);
        Ok(match fixed.cmd {
            PacketType::Connect => {
                // passing protocol_version is unnecessary here
                MqttPacket::Connect(ConnectPacket::decode(&mut self.reader, fixed, length, 5)?)
            }
            PacketType::Connack => MqttPacket::Connack(ConnackPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Subscribe => MqttPacket::Subscribe(SubscribePacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Suback => MqttPacket::Suback(SubackPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Publish => MqttPacket::Publish(PublishPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Puback => MqttPacket::Puback(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrec => MqttPacket::Pubrec(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrel => MqttPacket::Pubrel(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubcomp => MqttPacket::Pubcomp(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Unsubscribe => MqttPacket::Unsubscribe(UnsubscribePacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Unsuback => MqttPacket::Unsuback(UnsubackPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pingreq => MqttPacket::Pingreq,
            PacketType::Pingresp => MqttPacket::Pingresp,
            PacketType::Disconnect => MqttPacket::Disconnect(DisconnectPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Auth => MqttPacket::Auth(AuthPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Reserved => return Err("Cannot use RESERVED message type".to_string()),
        })
    }
}
