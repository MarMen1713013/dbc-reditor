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
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value & Self::INVALID_BIT_MASK != 0 {
            Err("Input value too high or trailing EFF flag")
        } else {
            Ok(FrameId{id: value})
        }
    }
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
}
