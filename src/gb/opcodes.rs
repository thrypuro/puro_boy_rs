use json;
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
        "DI" => Instruction::DI,
        _ => panic!("Unknown instruction: {}", instr),
    }
}
