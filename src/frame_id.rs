#[derive(Debug, Clone, PartialEq)]
pub struct FrameId {
    id: u32,
}

impl FrameId {
    // Constants
    const INVALID_BIT_MASK: u32 = 0xE000_0000;

    // Functions
    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl TryFrom<u32> for FrameId {
    type Error = FrameIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value & Self::INVALID_BIT_MASK != 0 {
            Err(FrameIdError::OutOfRange)
        } else {
            Ok(FrameId { id: value })
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FrameFormat {
    StandardCan,
    StandardCanFd,
    ExtendedCan,
    ExtendedCanFd,
    J1939,
    CANopen,
}

#[derive(Eq, PartialEq, Debug)]
pub enum FrameIdError {
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_id_is_not_valid() {
        let result = FrameId::try_from(0x3FFF_FFFF);
        assert!(result.is_err());
    }
    #[test]
    fn frame_id_is_valid() {
        let result = FrameId::try_from(0x1FFF_FFFF);
        assert!(result.is_ok());
    }
    #[test]
    fn frame_id_validation() {
        let cases = [
            (0x0000_0000, true),
            (0x1FFF_FFFF, true),
            (0x2000_0000, false),
            (0x8000_0000, false),
        ];

        for (value, expected_valid) in cases {
            let result = FrameId::try_from(value);

            assert_eq!(
                result.is_ok(),
                expected_valid,
                "failed for value {value:#010X}"
            );
        }
    }
}
