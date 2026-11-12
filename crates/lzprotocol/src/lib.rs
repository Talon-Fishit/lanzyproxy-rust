//! Growtopia wire protocol.
//!
//! Growtopia uses ENet as its transport. Each ENet payload is a `NetMessage`:
//! a little-endian `u32` message type followed by the payload.
//!
//! `GAME_PACKET` (type 4) carries a `GameUpdatePacket` — a fixed 56-byte
//! header — optionally followed by extension data such as a `VariantList`.
//!
//! Field offsets follow the layout used by Growtopia 3.x proxies. They are
//! collected here so a protocol change needs one edit. Verify against your
//! target client before relying on a field.

/// NetMessage type, the first `u32` of every ENet payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NetMessage {
    Unknown = 0,
    ServerHello = 1,
    GenericText = 2,
    GameMessage = 3,
    GamePacket = 4,
    Error = 5,
    Track = 6,
    ClientLogRequest = 7,
}

impl NetMessage {
    /// Map a raw type to its variant, or `Unknown` for anything unrecognised.
    pub fn from_raw(value: u32) -> Self {
        match value {
            1 => NetMessage::ServerHello,
            2 => NetMessage::GenericText,
            3 => NetMessage::GameMessage,
            4 => NetMessage::GamePacket,
            5 => NetMessage::Error,
            6 => NetMessage::Track,
            7 => NetMessage::ClientLogRequest,
            _ => NetMessage::Unknown,
        }
    }
}

/// Tank type, carried in the `type` field of a `GameUpdatePacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Tank {
    State = 0,
    CallFunction = 1,
    UpdateStatus = 2,
    TileChangeRequest = 3,
    SendMapData = 4,
    SendTileUpdateData = 5,
    SendTileUpdateDataMultiple = 6,
    TileActivateRequest = 7,
    TileApplyDamage = 8,
    SendInventoryState = 9,
    ItemActivateRequest = 10,
    ItemActivateObjectRequest = 11,
    SendTileTreeState = 12,
    ModifyItemInventory = 13,
    ItemChangeObject = 14,
    SendLock = 15,
    SendItemDatabaseData = 16,
    SendParticleEffect = 17,
    SetIconState = 18,
    ItemEffect = 19,
    SetCharacterState = 20,
    PingReply = 22,
    PingRequest = 23,
    GotPunched = 24,
    PunchEffect = 25,
    StartTrade = 26,
    TradeData = 27,
    SendMapDataV2 = 28,
    Ping = 29,
}

/// Prefix a message with its type.
pub fn frame(msg_type: NetMessage, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&(msg_type as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Split a framed payload into its type and body.
pub fn unframe(data: &[u8]) -> (NetMessage, &[u8]) {
    if data.len() < 4 {
        return (NetMessage::Unknown, &[]);
    }
    let raw = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    (NetMessage::from_raw(raw), &data[4..])
}

/// Type tag of a `Variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VariantType {
    Float = 2,
    String = 3,
    Int = 5,
}

/// A single typed value in a `VariantList`.
#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    Float(f32),
    String(String),
    Int(i32),
}

impl Variant {
    /// Infer the variant type from a Rust value.
    pub fn from_value(value: &str) -> Self {
        Variant::String(value.to_string())
    }

    /// The wire type tag.
    pub fn type_tag(&self) -> VariantType {
        match self {
            Variant::Float(_) => VariantType::Float,
            Variant::String(_) => VariantType::String,
            Variant::Int(_) => VariantType::Int,
        }
    }

    /// The value as a string, for logging and dialogs.
    pub fn as_string(&self) -> String {
        match self {
            Variant::Float(v) => v.to_string(),
            Variant::String(s) => s.clone(),
            Variant::Int(v) => v.to_string(),
        }
    }

    /// Encode the variant.
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Variant::String(s) => {
                let raw = s.as_bytes();
                let mut out = Vec::with_capacity(5 + raw.len());
                out.push(VariantType::String as u8);
                out.extend_from_slice(&(raw.len() as u32).to_le_bytes());
                out.extend_from_slice(raw);
                out
            }
            Variant::Int(v) => {
                let mut out = Vec::with_capacity(5);
                out.push(VariantType::Int as u8);
                out.extend_from_slice(&v.to_le_bytes());
                out
            }
            Variant::Float(v) => {
                let mut out = Vec::with_capacity(5);
                out.push(VariantType::Float as u8);
                out.extend_from_slice(&v.to_le_bytes());
                out
            }
        }
    }

    /// Decode one variant, returning it and the offset just past it.
    pub fn parse(data: &[u8], offset: usize) -> Result<(Variant, usize), ProtocolError> {
        let tag = *data.get(offset).ok_or(ProtocolError::Truncated)?;
        let mut at = offset + 1;
        match tag {
            3 => {
                let len_bytes = data
                    .get(at..at + 4)
                    .ok_or(ProtocolError::Truncated)?;
                let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]])
                    as usize;
                at += 4;
                let bytes = data.get(at..at + len).ok_or(ProtocolError::Truncated)?;
                // Growtopia strings are latin-1 in practice.
                let text = bytes.iter().map(|&b| b as char).collect();
                Ok((Variant::String(text), at + len))
            }
            5 => {
                let bytes = data.get(at..at + 4).ok_or(ProtocolError::Truncated)?;
                let v = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Ok((Variant::Int(v), at + 4))
            }
            2 => {
                let bytes = data.get(at..at + 4).ok_or(ProtocolError::Truncated)?;
                let v = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Ok((Variant::Float(v), at + 4))
            }
            other => Err(ProtocolError::UnsupportedVariant(other)),
        }
    }
}

/// A function name plus its ordered arguments.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariantList {
    pub func: String,
    pub params: Vec<Variant>,
}

impl VariantList {
    /// An empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// A list carrying only a function name.
    pub fn with_func(func: &str) -> Self {
        Self {
            func: func.to_string(),
            params: Vec::new(),
        }
    }

    /// Append an argument.
    pub fn add(&mut self, value: Variant) -> &mut Self {
        self.params.push(value);
        self
    }

    /// Append a string argument.
    pub fn add_str(&mut self, value: &str) -> &mut Self {
        self.params.push(Variant::String(value.to_string()));
        self
    }

    /// Append an integer argument.
    pub fn add_int(&mut self, value: i32) -> &mut Self {
        self.params.push(Variant::Int(value));
        self
    }

    /// Encode the list: a leading zero byte, the count, then every variant.
    pub fn serialize(&self) -> Vec<u8> {
        let mut all = Vec::with_capacity(self.params.len() + 1);
        all.push(Variant::String(self.func.clone()));
        all.extend(self.params.iter().cloned());

        let mut out = Vec::with_capacity(5 + all.len() * 8);
        out.push(0);
        out.extend_from_slice(&(all.len() as u32).to_le_bytes());
        for v in &all {
            out.extend_from_slice(&v.serialize());
        }
        out
    }

    /// Decode a list.
    pub fn parse(data: &[u8]) -> Result<VariantList, ProtocolError> {
        if data.is_empty() {
            return Ok(VariantList::new());
        }
        let count_bytes = data.get(1..5).ok_or(ProtocolError::Truncated)?;
        let count = u32::from_le_bytes([
            count_bytes[0],
            count_bytes[1],
            count_bytes[2],
            count_bytes[3],
        ]) as usize;

        let mut offset = 5;
        let mut variants = Vec::with_capacity(count);
        for _ in 0..count {
            let (v, next) = Variant::parse(data, offset)?;
            variants.push(v);
            offset = next;
        }
        let func = match variants.first() {
            Some(Variant::String(s)) => s.clone(),
            Some(other) => other.as_string(),
            None => String::new(),
        };
        let params = variants.into_iter().skip(1).collect();
        Ok(VariantList { func, params })
    }
}

/// The 56-byte `GAME_PACKET` header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameUpdatePacket {
    pub packet_type: u8,
    pub pad1: u8,
    pub pad2: u8,
    pub pad3: u8,
    pub netid: i32,
    pub target_netid: i32,
    pub flags: i32,
    pub x: f32,
    pub y: f32,
    pub xspeed: f32,
    pub yspeed: f32,
    pub xpos: f32,
    pub ypos: f32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub extra: [u8; 8],
}

/// Size of the serialised `GameUpdatePacket`.
pub const GUP_SIZE: usize = 56;

impl Default for GameUpdatePacket {
    fn default() -> Self {
        Self {
            packet_type: 0,
            pad1: 0,
            pad2: 0,
            pad3: 0,
            netid: 0,
            target_netid: 0,
            flags: 0,
            x: 0.0,
            y: 0.0,
            xspeed: 0.0,
            yspeed: 0.0,
            xpos: 0.0,
            ypos: 0.0,
            tile_x: 0,
            tile_y: 0,
            extra: [0u8; 8],
        }
    }
}

impl GameUpdatePacket {
    /// Encode the header.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(GUP_SIZE);
        out.push(self.packet_type);
        out.push(self.pad1);
        out.push(self.pad2);
        out.push(self.pad3);
        out.extend_from_slice(&self.netid.to_le_bytes());
        out.extend_from_slice(&self.target_netid.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&self.x.to_le_bytes());
        out.extend_from_slice(&self.y.to_le_bytes());
        out.extend_from_slice(&self.xspeed.to_le_bytes());
        out.extend_from_slice(&self.yspeed.to_le_bytes());
        out.extend_from_slice(&self.xpos.to_le_bytes());
        out.extend_from_slice(&self.ypos.to_le_bytes());
        out.extend_from_slice(&self.tile_x.to_le_bytes());
        out.extend_from_slice(&self.tile_y.to_le_bytes());
        let mut extra = self.extra;
        out.extend_from_slice(&extra);
        out
    }

    /// Decode the header.
    pub fn parse(data: &[u8]) -> Result<GameUpdatePacket, ProtocolError> {
        if data.len() < GUP_SIZE {
            return Err(ProtocolError::Truncated);
        }
        let mut p = GameUpdatePacket::default();
        p.packet_type = data[0];
        p.pad1 = data[1];
        p.pad2 = data[2];
        p.pad3 = data[3];
        p.netid = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        p.target_netid = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        p.flags = i32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        p.x = f32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        p.y = f32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        p.xspeed = f32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        p.yspeed = f32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        p.xpos = f32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        p.ypos = f32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        p.tile_x = i32::from_le_bytes([data[40], data[41], data[42], data[43]]);
        p.tile_y = i32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        p.extra.copy_from_slice(&data[48..56]);
        Ok(p)
    }
}

// ---------------------------------------------------------------------------
// Packet constructors
// ---------------------------------------------------------------------------

/// A `GENERIC_TEXT` message: a 3-byte length followed by the text.
pub fn text_packet(text: &str, msg_type: NetMessage) -> Vec<u8> {
    let payload = text.as_bytes();
    let mut body = Vec::with_capacity(3 + payload.len());
    body.extend_from_slice(&(payload.len() as u32).to_le_bytes()[..3]);
    body.extend_from_slice(payload);
    frame(msg_type, &body)
}

/// A `GAME_PACKET` carrying a header plus optional extension bytes.
pub fn game_packet(pkt: &GameUpdatePacket, extension: &[u8]) -> Vec<u8> {
    let mut body = pkt.serialize();
    body.extend_from_slice(extension);
    frame(NetMessage::GamePacket, &body)
}

/// A `GAME_PACKET` whose extension is a `VariantList`.
pub fn variant_packet(list: &VariantList) -> Vec<u8> {
    let pkt = GameUpdatePacket {
        packet_type: Tank::CallFunction as u8,
        netid: -1,
        ..Default::default()
    };
    game_packet(&pkt, &list.serialize())
}

/// A console line.
pub fn console_message(text: &str) -> Vec<u8> {
    let mut list = VariantList::with_func("OnConsoleMessage");
    list.add_str(text);
    variant_packet(&list)
}

/// A server-side dialog request. `embed_data` is the dialog markup.
pub fn dialog_request(name: &str, embed_data: &str) -> Vec<u8> {
    let mut list = VariantList::with_func("OnDialogRequest");
    list.add_str(&format!("dialog_name|{name}\nembed_data|{embed_data}"));
    variant_packet(&list)
}

/// A client-side `action|...` message.
pub fn action_packet(text: &str) -> Vec<u8> {
    text_packet(text, NetMessage::GenericText)
}

// ---------------------------------------------------------------------------
// Text protocol helpers
// ---------------------------------------------------------------------------

/// Decode a `GENERIC_TEXT` payload: 3-byte length then data.
pub fn parse_text_payload(payload: &[u8]) -> String {
    if payload.len() >= 4 {
        let declared = u32::from_le_bytes([payload[0], payload[1], payload[2], 0]) as usize;
        if declared > 0 && declared <= payload.len() - 3 {
            return String::from_utf8_lossy(&payload[3..3 + declared]).into_owned();
        }
    }
    String::from_utf8_lossy(payload)
        .trim_end_matches('\0')
        .to_string()
}

/// Parse `action|name\n|key|value` into the action name and its fields.
pub fn parse_action(text: &str) -> (String, Vec<(String, String)>) {
    let mut action = String::new();
    let mut fields = Vec::new();
    for line in text.split('\n') {
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("action|") {
            action = rest.to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix('|') {
            match rest.split_once('|') {
                Some((k, v)) => fields.push((k.to_string(), v.to_string())),
                None => fields.push((rest.to_string(), String::new())),
            }
        }
    }
    (action, fields)
}

/// Failures that can come out of decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// The buffer ended before the value was complete.
    Truncated,
    /// A variant carried a type tag this build does not know.
    UnsupportedVariant(u8),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::Truncated => write!(f, "truncated packet"),
            ProtocolError::UnsupportedVariant(t) => write!(f, "unsupported variant type {t}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_round_trips() {
        let payload = b"hello";
        let framed = frame(NetMessage::GamePacket, payload);
        assert_eq!(framed.len(), 9);
        let (kind, body) = unframe(&framed);
        assert_eq!(kind, NetMessage::GamePacket);
        assert_eq!(body, payload);
    }

    #[test]
    fn variant_list_round_trips() {
        let mut list = VariantList::with_func("OnConsoleMessage");
        list.add_str("hi");
        list.add_int(42);
        let bytes = list.serialize();
        let parsed = VariantList::parse(&bytes).expect("parse");
        assert_eq!(parsed.func, "OnConsoleMessage");
        assert_eq!(parsed.params[0], Variant::String("hi".into()));
        assert_eq!(parsed.params[1], Variant::Int(42));
    }

    #[test]
    fn game_update_packet_is_56_bytes() {
        let pkt = GameUpdatePacket {
            netid: 7,
            x: 1.5,
            tile_x: 11,
            ..Default::default()
        };
        let bytes = pkt.serialize();
        assert_eq!(bytes.len(), GUP_SIZE);
        let back = GameUpdatePacket::parse(&bytes).expect("parse");
        assert_eq!(back.netid, 7);
        assert_eq!(back.x, 1.5);
        assert_eq!(back.tile_x, 11);
    }

    #[test]
    fn text_payload_uses_three_byte_length() {
        let pkt = text_packet("abc", NetMessage::GenericText);
        let (kind, body) = unframe(&pkt);
        assert_eq!(kind, NetMessage::GenericText);
        // A three-byte little-endian length, then the text.
        assert_eq!(body[0], 3);
        assert_eq!(body[1], 0);
        assert_eq!(body[2], 0);
        assert_eq!(&body[3..6], b"abc");
    }

    #[test]
    fn action_parsing() {
        let (action, fields) = parse_action("action|dialog_return\n|dialog_name|shop\n|item|1");
        assert_eq!(action, "dialog_return");
        assert_eq!(fields[0], ("dialog_name".to_string(), "shop".to_string()));
        assert_eq!(fields[1], ("item".to_string(), "1".to_string()));
    }
}
