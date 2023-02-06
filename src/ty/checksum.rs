use deku::bitvec::{BitSlice, Msb0};
use deku::ctx::{ByteSize, Limit};
use deku::DekuRead;

use crate::ReadBinError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Checksum {
    /// 异或校验
    Xor,
    /// 补码
    Complement,
}

impl Checksum {
    pub fn read<'a>(
        &self,
        bits: &'a BitSlice<u8, Msb0>,
    ) -> Result<(Vec<u8>, &'a BitSlice<u8, Msb0>), ReadBinError> {
        match self {
            Self::Xor | Checksum::Complement => {
                let (v, d) = Vec::<u8>::read(bits, Limit::new_byte_size(ByteSize(1)))?;
                Ok((d, v))
            }
        }
    }

    pub fn checksum(&self, data: &[u8]) -> Vec<u8> {
        assert!(data.len() > 0, "用以计算校验和的数据为空");
        match self {
            Self::Xor => {
                let mut c = data[0];
                for b in &data[1..] {
                    c ^= *b;
                }
                vec![c]
            }
            Checksum::Complement => {
                let sum: u8 = data.iter().sum();
                vec![(!sum) + 1]
            }
        }
    }

    pub fn check(&self, data: &[u8], checksum: &[u8]) -> bool {
        return self.checksum(data) == checksum;
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Checksum::Xor => "异或",
            Checksum::Complement => "补码",
        }
    }
}
