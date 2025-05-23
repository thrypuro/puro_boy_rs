use json;

pub const DEB: bool = true;

pub struct Registers {
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
    PC,
}
#[derive(Copy, Clone, Debug)]
pub enum FlagNames {
    Z,
    N,
    H,
    C,
    NZ,
    NC,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    // Single byte instructions
    NOP,
    RRCA,
    INC,
    DEC,
    RLCA,
    RLA,
    RRA,
    PUSH,
    POP,

    // Logical instructions
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    OR,
    XOR,
    LD,
    LDH,

    // Bit manipulation instructions
    CP,
    DAA,
    CPL,
    SCF,
    CCF,

    // Control flow instructions
    JP,
    JR,
    CALL,
    RET,
    RETI,
    RST,

    // Other instructions
    DI,

    // PREFIX instruction (0xCB)
    PREFIX,

    // CB Prefixed instructions
    RLC,
    RRC,
    RL,
    RR,
    SLA,
    SRA,
    SRL,
    SWAP,
    BIT,
    RES,
    SET,
}

#[derive(Copy, Clone, Debug)]
pub enum Operand {
    Register(RegisterNames),
    Memory(u16),      // Memory address
    Immediate(u8),    // Immediate value
    Immediate16(u16), // 16-bit immediate value
    Flag(FlagNames),
    NIL,
}

pub struct Flag {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

pub fn get_opcodes() -> json::JsonValue {
    // Load the opcodes from the JSON file
    let opcodes = include_str!("opcodes.json");
    json::parse(opcodes).unwrap()
}

// Helper functions
pub fn match_string_to_instruction(instr: &str) -> Instruction {
    // match the instruction to the enum
    match instr {
        "ADD" => Instruction::ADD,
        "ADC" => Instruction::ADC,
        "SUB" => Instruction::SUB,
        "SBC" => Instruction::SBC,
        "AND" => Instruction::AND,
        "OR" => Instruction::OR,
        "XOR" => Instruction::XOR,
        "CP" => Instruction::CP,
        "DAA" => Instruction::DAA,
        "CPL" => Instruction::CPL,
        "SCF" => Instruction::SCF,
        "CCF" => Instruction::CCF,
        "NOP" => Instruction::NOP,
        "INC" => Instruction::INC,
        "DEC" => Instruction::DEC,
        "LD" => Instruction::LD,
        "LDH" => Instruction::LDH,
        "RRCA" => Instruction::RRCA,
        "RLA" => Instruction::RLA,
        "RRA" => Instruction::RRA,
        "RLCA" => Instruction::RLCA,
        "PUSH" => Instruction::PUSH,
        "POP" => Instruction::POP,
        "JP" => Instruction::JP,
        "JR" => Instruction::JR,
        "CALL" => Instruction::CALL,
        "RET" => Instruction::RET,
        "DI" => Instruction::DI,
        "PREFIX" => Instruction::PREFIX,
        // CB Prefixed instructions
        "RLC" => Instruction::RLC,
        "RRC" => Instruction::RRC,
        "RL" => Instruction::RL,
        "RR" => Instruction::RR,
        "SLA" => Instruction::SLA,
        "SRA" => Instruction::SRA,
        "SRL" => Instruction::SRL,
        "SWAP" => Instruction::SWAP,
        "BIT" => Instruction::BIT,
        "RES" => Instruction::RES,
        "SET" => Instruction::SET,
        _ => panic!("Unknown instruction: {}", instr),
    }
}

pub fn match_string_preinstruction(instr: &str) -> Instruction {
    match instr {
        "RLC" => Instruction::RLC,
        "RRC" => Instruction::RRC,
        "RL" => Instruction::RL,
        "RR" => Instruction::RR,
        "SLA" => Instruction::SLA,
        "SRA" => Instruction::SRA,
        "SWAP" => Instruction::SWAP,
        "SRL" => Instruction::SRL,
        "BIT" => Instruction::BIT,
        "RES" => Instruction::RES,
        "SET" => Instruction::SET,
        _ => panic!("Unknown CB-prefixed instruction: {}", instr),
    }
}

pub fn match_string_to_register(reg: &str) -> RegisterNames {
    // match the register to the enum
    match reg {
        "A" => RegisterNames::A,
        "B" => RegisterNames::B,
        "C" => RegisterNames::C,
        "D" => RegisterNames::D,
        "E" => RegisterNames::E,
        "H" => RegisterNames::H,
        "L" => RegisterNames::L,

        _ => match_string_to_register16(reg),
    }
}
pub fn match_string_to_register16(reg: &str) -> RegisterNames {
    // match the register to the enum
    match reg {
        "AF" => RegisterNames::AF,
        "BC" => RegisterNames::BC,
        "DE" => RegisterNames::DE,
        "HL" => RegisterNames::HL,
        "SP" => RegisterNames::SP,
        "PC" => RegisterNames::PC,

        _ => panic!("Unknown register: {}", reg),
    }
}

pub fn match_string_to_flag(flag: &str) -> FlagNames {
    match flag {
        "Z" => FlagNames::Z,
        "H" => FlagNames::H,
        "C" => FlagNames::C,
        "N" => FlagNames::N,
        "NZ" => FlagNames::NZ,
        "NC" => FlagNames::NC,
        _ => panic!("Invalid Flag type {:?}", flag),
    }
}

pub fn is_flag(operand: &str, instr: &Instruction) -> bool {
    return (operand.contains("Z") || operand.contains("NZ") || operand.contains("C"))
        && (*instr == Instruction::JP
            || *instr == Instruction::CALL
            || *instr == Instruction::RET
            || *instr == Instruction::JR);
}
