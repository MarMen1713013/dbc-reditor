use std::convert::TryFrom;

pub struct Message {
    frame_id: FrameId,
    frame_format: FrameFormat,
    name: String,
    payload_length: u8,
    sender: Option<NodeId>,
}

impl Message {
    const MAX_STANDARD_FRAME_ID: u32 = 0x7FF;
    pub fn new(
        name: &str, 
        frame_id: u32,
        frame_format: FrameFormat,
        length: u8,
        sender: Option<NodeId>) -> Result<Message, &'static str> {

        let f = FrameId::try_from(frame_id)?;
        match &frame_format{
            FrameFormat::CANopen
            | FrameFormat::StandardCan
            | FrameFormat::StandardCanFd => {
                if f.get_id() > Self::MAX_STANDARD_FRAME_ID {
                    return Err("Frame Id is too long for the selected frame format");
                }
            }
            _ => {}
        }
        let mut max_length = 8;
        match &frame_format{
            FrameFormat::StandardCanFd | FrameFormat::ExtendedCanFd => {
                max_length = 64;
            }
            _ => {}
        }
        if length > max_length {
            return Err("Payload length is too long for selected frame format");
        }
        Ok(Message {
            frame_id: f,
            frame_format: frame_format,
            name: String::from(name),
            payload_length: length,
            sender: sender,
        })
    }
}

pub enum FrameFormat {
    StandardCan,
    StandardCanFd,
    ExtendedCan,
    ExtendedCanFd,
    J1939,
    CANopen,
}

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

pub struct NodeId {
    id: u64,
}

