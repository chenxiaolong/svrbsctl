use std::{
    fmt,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BtAddr(u64);

impl BtAddr {
    fn new(addr: u64) -> Self {
        debug_assert!(addr <= 0xff_ff_ff_ff_ff_ff);

        Self(addr)
    }
}

impl fmt::Display for BtAddr {
    #[allow(clippy::erasing_op, clippy::identity_op)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<02x}:{:<02x}:{:<02x}:{:<02x}:{:<02x}:{:<02x}",
            self.0 >> (8 * 5) & 0xff,
            self.0 >> (8 * 4) & 0xff,
            self.0 >> (8 * 3) & 0xff,
            self.0 >> (8 * 2) & 0xff,
            self.0 >> (8 * 1) & 0xff,
            self.0 >> (8 * 0) & 0xff
        )
    }
}

impl From<BtAddr> for u64 {
    fn from(addr: BtAddr) -> Self {
        addr.0
    }
}

impl From<u64> for BtAddr {
    fn from(addr: u64) -> Self {
        Self::new(addr)
    }
}

impl FromStr for BtAddr {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut addr = 0u64;

        let mut n = 0;
        for byte in input.split(':') {
            if n == 6 {
                return Err("Too many octets".into());
            }

            addr = (addr << 8) | u8::from_str_radix(byte, 16)
                .map_err(|e| format!("Invalid octet: {}", e))? as u64;

            n += 1;
        }

        if n != 6 {
            return Err("Too few octets".into());
        }

        Ok(BtAddr::new(addr))
    }
}
