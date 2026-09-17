use porpoise_core::error::{PorpoiseError, Result};

pub const PROTOCOL_MAGIC: [u8; 2] = [0x50, 0x50];
pub const PROTOCOL_VERSION: u8 = 0x01;
pub const MIN_PROTOCOL_VERSION: u8 = 0x01;
pub const HEADER_SIZE: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameFlags(u8);

impl FrameFlags {
    pub const REQUEST: Self = Self(0b0000_0001);
    pub const RESPONSE: Self = Self(0b0000_0010);
    pub const EVENT: Self = Self(0b0000_0100);
    pub const COMPRESSED: Self = Self(0b0000_1000);
    pub const ACK: Self = Self(0b0001_0000);
    pub const STREAM: Self = Self(0b0010_0000);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        // Accept any combination of known bits; reject unknown bits
        if bits & !0b0011_1111 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & 0b0011_1111)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for FrameFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for FrameFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for FrameFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for FrameFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub version: u8,
    pub flags: FrameFlags,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(flags: FrameFlags, payload: Vec<u8>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            flags,
            payload,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&PROTOCOL_MAGIC);
        buf.push(self.version);
        buf.push(self.flags.bits());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(PorpoiseError::Ipc("short frame".into()));
        }
        if data[0..2] != PROTOCOL_MAGIC {
            return Err(PorpoiseError::Ipc("bad magic".into()));
        }
        let version = data[2];
        if version < MIN_PROTOCOL_VERSION || version > PROTOCOL_VERSION {
            return Err(PorpoiseError::IpcVersionMismatch {
                server_min: MIN_PROTOCOL_VERSION,
                server_max: PROTOCOL_VERSION,
                client: version,
            });
        }
        let flags = FrameFlags::from_bits(data[3]).ok_or_else(|| PorpoiseError::Ipc("bad flags".into()))?;
        let len = u32::from_le_bytes(
            data[4..8]
                .try_into()
                .map_err(|_| PorpoiseError::Ipc("header length slice misaligned".into()))?,
        ) as usize;
        if data.len() < HEADER_SIZE + len {
            return Err(PorpoiseError::Ipc("truncated".into()));
        }
        Ok(Self {
            version,
            flags,
            payload: data[HEADER_SIZE..HEADER_SIZE + len].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_roundtrip() {
        let f = Frame::new(FrameFlags::REQUEST, b"hi".to_vec());
        let d = Frame::decode(&f.encode().unwrap()).unwrap();
        assert_eq!(d.flags, FrameFlags::REQUEST);
        assert_eq!(d.payload, b"hi");
    }

    #[test]
    fn test_version_below_min_rejected() {
        let buf = vec![PROTOCOL_MAGIC[0], PROTOCOL_MAGIC[1], 0x00, 0x00, 0, 0, 0, 0];
        assert!(Frame::decode(&buf).is_err());
    }

    #[test]
    fn test_version_current_accepted() {
        let f = Frame::new(FrameFlags::REQUEST, b"hi".to_vec());
        let encoded = f.encode().unwrap();
        assert!(Frame::decode(&encoded).is_ok());
    }

    #[test]
    fn test_version_above_max_rejected() {
        let buf = vec![PROTOCOL_MAGIC[0], PROTOCOL_MAGIC[1], 0x02, 0x00, 0, 0, 0, 0];
        assert!(Frame::decode(&buf).is_err());
    }
}
