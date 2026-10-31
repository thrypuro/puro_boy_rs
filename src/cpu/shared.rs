pub struct Registers {
    /// High byte is A. The low byte is unused: F is built from `flag` on read.
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub flag: Flag,
}

#[derive(Copy, Clone, Debug)]
pub enum RegisterNames {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
}

pub struct Flag {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

/// The opcode table from gbdev.io. Only the tests use it now, to check cycle counts.
#[cfg(test)]
pub fn get_opcodes() -> json::JsonValue {
    json::parse(include_str!("opcodes.json")).unwrap()
}
