use crate::{
    frame_id::{FrameFormat, FrameId, FrameIdError},
    node::NodeId,
};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct MessageId {
    id: u64,
}

impl MessageId {
    pub(crate) fn new(input: u64) -> Self {
        Self { id: input }
    }
}

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
        sender: Option<NodeId>,
    ) -> Result<Self, MessageError> {
        let f = FrameId::try_from(frame_id)?;
        match &frame_format {
            FrameFormat::CANopen | FrameFormat::StandardCan | FrameFormat::StandardCanFd => {
                if f.get_id() > Self::MAX_STANDARD_FRAME_ID {
                    return Err(MessageError::FrameTooLargeForFormat);
                }
            }
            _ => {}
        }
        let mut max_length = 8;
        match &frame_format {
            FrameFormat::StandardCanFd | FrameFormat::ExtendedCanFd => {
                max_length = 64;
            }
            _ => {}
        }
        if length > max_length {
            return Err(MessageError::PayloadTooLong);
        }
        Ok(Self {
            frame_id: f,
            frame_format: frame_format,
            name: String::from(name),
            payload_length: length,
            sender: sender,
        })
    }
    pub(crate) fn set_frame_id(&mut self, f_id: FrameId) -> Result<(), MessageError> {
        match &self.frame_format {
            FrameFormat::CANopen | FrameFormat::StandardCan | FrameFormat::StandardCanFd => {
                if f_id.get_id() > Self::MAX_STANDARD_FRAME_ID {
                    return Err(MessageError::FrameTooLargeForFormat);
                }
            }
            _ => {}
        }
        self.frame_id = f_id;
        Ok(())
    }
    pub fn frame_id(&self) -> &FrameId {
        &self.frame_id
    }
    pub(crate) fn set_payload_length(&mut self, length: u8) -> Result<(), MessageError> {
        let mut max_length = 8;
        match &self.frame_format {
            FrameFormat::StandardCanFd | FrameFormat::ExtendedCanFd => {
                max_length = 64;
            }
            _ => {}
        }
        if length > max_length {
            return Err(MessageError::PayloadTooLong);
        }
        self.payload_length = length;
        Ok(())
    }
    pub fn payload_length(&self) -> u8 {
        self.payload_length
    }
    pub(crate) fn set_frame_format(&mut self, frame_format: FrameFormat) -> Result<(), MessageError> {
        let mut max_length = 8;
        match &frame_format {
            FrameFormat::StandardCanFd | FrameFormat::ExtendedCanFd => {
                max_length = 64;
            }
            _ => {}
        }
        if self.payload_length() > max_length {
            return Err(MessageError::PayloadTooLong);
        }
        match &frame_format {
            FrameFormat::CANopen | FrameFormat::StandardCan | FrameFormat::StandardCanFd => {
                if self.frame_id().get_id() > Self::MAX_STANDARD_FRAME_ID {
                    return Err(MessageError::FrameTooLargeForFormat);
                }
            }
            _ => {}
        }
        self.frame_format = frame_format;
        Ok(())
    }
    pub fn frame_format(&self) -> FrameFormat {
        self.frame_format
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn sender(&self) -> Option<NodeId> {
        self.sender
    }
    pub(crate) fn apply_command(&mut self, cmd: MessageCommand) -> Result<(), MessageError> {
        match cmd {
            MessageCommand::Rename(new_name) => {
                self.name = new_name;
            }
            MessageCommand::SetFrameId(new_frame_id) => {
                self.set_frame_id(new_frame_id)?;
            }
            MessageCommand::SetFrameFormat(new_frame_format) => {
                self.set_frame_format(new_frame_format)?;
            }
            MessageCommand::SetPayloadLength(new_length) => {
                self.set_payload_length(new_length)?;
            }
            MessageCommand::SetSender(new_sender) => {
                self.sender = new_sender;
            }
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum MessageError {
    InvalidFrameId(FrameIdError),
    FrameTooLargeForFormat,
    PayloadTooLong,
}

impl From<FrameIdError> for MessageError {
    fn from(error: FrameIdError) -> Self {
        MessageError::InvalidFrameId(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageCommand {
    Rename(String),
    SetFrameId(FrameId),
    SetFrameFormat(FrameFormat),
    SetPayloadLength(u8),
    SetSender(Option<NodeId>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_validation() {
        let cases = [
            (0x7FF, FrameFormat::StandardCan, 8, true),
            (0x800, FrameFormat::StandardCan, 8, false),
            (0x7FF, FrameFormat::StandardCan, 9, false),
            (0x800, FrameFormat::ExtendedCan, 8, true),
            (0x1FFF_FFFF, FrameFormat::ExtendedCan, 8, true),
            (0x800, FrameFormat::ExtendedCan, 9, false),
            (0x7FF, FrameFormat::StandardCanFd, 64, true),
            (0x800, FrameFormat::StandardCanFd, 64, false),
            (0x7FF, FrameFormat::StandardCanFd, 65, false),
            (0x1FFF_FFFF, FrameFormat::ExtendedCanFd, 64, true),
            (0x1FFF_FFFF, FrameFormat::ExtendedCanFd, 65, false),
            (0x7FF, FrameFormat::CANopen, 8, true),
            (0x800, FrameFormat::CANopen, 8, false),
            (0x7FF, FrameFormat::CANopen, 9, false),
            (0x12345, FrameFormat::J1939, 8, true),
            (0x12345, FrameFormat::J1939, 9, false),
        ];

        for (f_id, f_format, len, expected) in cases {
            let result = Message::new("Test message", f_id, f_format, len, None);

            assert_eq!(result.is_ok(), expected, "frame_id={f_id:#X}, length={len}");
        }
    }
}
