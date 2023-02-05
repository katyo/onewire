pub trait OpCode {
    fn op_code(&self) -> u8;
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Command {
    MatchRom = 0x55,
    SearchRom = 0xF0,
    SearchRomAlarmed = 0xEC,
    SkipRom = 0xCC,
    ReadRom = 0x33,
}

impl OpCode for Command {
    fn op_code(&self) -> u8 {
        *self as _
    }
}
