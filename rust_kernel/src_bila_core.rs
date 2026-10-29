use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum BilaReflexError {
    #[error("Ungültiger Opcode: {0}")]
    InvalidOpcode(u16),
    #[error("Reflex Emergency Cutout aktiviert")]
    EmergencyCutout,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReflexCommand {
    MoveForward { speed: u16 },
    ReadAdcSensor { channel: u16 },
    TransmitUart { baud_rate_code: u16 },
    SetEmergencyLock,
    UnknownOpcode(u16),
}

pub struct BilaReflexCore {
    pub pc: u16,
    pub temp_reg: u16,
    pub execution_counter: u64,
}

impl BilaReflexCore {
    pub fn new() -> Self {
        Self { pc: 0, temp_reg: 0, execution_counter: 0 }
    }

    #[inline(always)]
    pub fn decode_bitstream(&mut self, instruction: u16) -> ReflexCommand {
        let opcode = (instruction >> 12) & 0x0F;
        let param = instruction & 0x0FFF;
        self.temp_reg = param;
        self.pc = self.pc.wrapping_add(1);

        match opcode {
            0b0001 => ReflexCommand::MoveForward { speed: param },
            0b0010 => ReflexCommand::ReadAdcSensor { channel: param },
            0b0011 => ReflexCommand::TransmitUart { baud_rate_code: param },
            0b1111 => ReflexCommand::SetEmergencyLock,
            other => ReflexCommand::UnknownOpcode(other),
        }
    }

    #[inline(always)]
    pub fn execute_reflex(&mut self, cmd: ReflexCommand) -> Result<&'static str, BilaReflexError> {
        self.execution_counter += 1;
        match cmd {
            ReflexCommand::MoveForward { speed: _ } => Ok("BILA_REFLEX_EXEC: MOVE_FORWARD_PASS"),
            ReflexCommand::ReadAdcSensor { channel: _ } => Ok("BILA_REFLEX_EXEC: READ_ADC_PASS"),
            ReflexCommand::TransmitUart { baud_rate_code: _ } => Ok("BILA_REFLEX_EXEC: TX_UART_PASS"),
            ReflexCommand::SetEmergencyLock => Err(BilaReflexError::EmergencyCutout),
            ReflexCommand::UnknownOpcode(op) => Err(BilaReflexError::InvalidOpcode(op)),
        }
    }
}
