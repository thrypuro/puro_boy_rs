#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuError {
    // MMU Operations
    ProhibitedRead,
    ProhibitedReadWord,
    ProhibitedWrite,
    ProhibitedWriteWord,

    RomNotInitialised,
}

pub type EmuRes<T> = std::result::Result<T, EmuError>;
