use json;
#[derive(Debug)]
pub enum Instruction {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    OR,
    XOR,
    CP,
    DAA,
    CPL,
    SCF,
    CCF,
    LD,
    NOP,
    INC,
    DEC,
    RLCA,
    RLA,
    RRCA,
    RRA,
}

pub fn get_opcodes() -> json::JsonValue {
    // Load the opcodes from the JSON file
    let opcodes = include_str!("opcodes.json");
    json::parse(opcodes).unwrap()
}


// Helper functions 
pub fn match_string_to_instruction(instr : &str) -> Instruction {
    // match the instruction to the enum
    match instr {
        "ADD" => Instruction::ADD,
        "ADC" => Instruction::ADC,
        "SUB" => Instruction::SUB,
        "SBC" => Instruction::SBC,
        "AND" => Instruction::AND,
        "OR"  => Instruction::OR,
        "XOR" => Instruction::XOR,
        "CP"  => Instruction::CP,
        "DAA" => Instruction::DAA,
        "CPL" => Instruction::CPL,
        "SCF" => Instruction::SCF,
        "CCF" => Instruction::CCF,
        "NOP" => Instruction::NOP,
        "INC" => Instruction::INC,
        "DEC" => Instruction::DEC,
        "LD"  => Instruction::LD,
        _     => panic!("Unknown instruction: {}", instr),
    }
}
