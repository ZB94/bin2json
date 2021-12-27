use deku::bitvec::{BitSlice, Msb0};
use deku::ctx::{Limit, Size};
use deku::DekuRead;

use crate::ReadBinError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Checksum {
    /// 异或校验
    Xor,
}

impl Checksum {
    pub fn read<'a>(&self, bits: &'a BitSlice<Msb0, u8>) -> Result<(Vec<u8>, &'a BitSlice<Msb0, u8>), ReadBinError> {
        match self {
            Self::Xor => {
                let (v, d) = Vec::<u8>::read(bits, Limit::new_size(Size::Bytes(1)))?;
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
        }
    }

    pub fn check(&self, data: &[u8], checksum: &[u8]) -> bool {
        return self.checksum(data) == checksum;
    }
}
