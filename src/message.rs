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

impl From<u64> for MessageId {
    fn from(input: u64) -> MessageId {
        MessageId::new(input)
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
        if !Self::validate_frame_id(&f, frame_format) {
            return Err(MessageError::FrameTooLargeForFormat);
        }
        if !Self::validate_payload_length(length, frame_format) {
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
        if !Self::validate_frame_id(&f_id, self.frame_format) {
            return Err(MessageError::FrameTooLargeForFormat);
        }
        self.frame_id = f_id;
        Ok(())
    }
    pub fn frame_id(&self) -> &FrameId {
        &self.frame_id
    }
    pub(crate) fn set_payload_length(&mut self, length: u8) -> Result<(), MessageError> {
        if !Self::validate_payload_length(length, self.frame_format) {
            return Err(MessageError::PayloadTooLong);
        }
        self.payload_length = length;
        Ok(())
    }
    pub fn payload_length(&self) -> u8 {
        self.payload_length
    }
    pub(crate) fn set_frame_format(
        &mut self,
        frame_format: FrameFormat,
    ) -> Result<(), MessageError> {
        if !Self::validate_payload_length(self.payload_length, frame_format) {
            return Err(MessageError::PayloadTooLong);
        }
        if !Self::validate_frame_id(self.frame_id(), frame_format) {
            return Err(MessageError::FrameTooLargeForFormat);
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
    fn validate_frame_id(f_id: &FrameId, f_format: FrameFormat) -> bool {
        match f_format {
            FrameFormat::CANopen | FrameFormat::StandardCan | FrameFormat::StandardCanFd => {
                if f_id.get_id() > Self::MAX_STANDARD_FRAME_ID {
                    return false;
                }
            }
            _ => {}
        }
        true
    }
    fn validate_payload_length(len: u8, f_format: FrameFormat) -> bool {
        let mut max_length = 8;
        match f_format {
            FrameFormat::StandardCanFd | FrameFormat::ExtendedCanFd => {
                max_length = 64;
            }
            _ => {}
        }
        if len > max_length {
            return false;
        }
        true
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
    #[test]
    fn frame_id_valid_updates() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetFrameId(
            FrameId::try_from(0x200).unwrap(),
        ));

        assert!(result.is_ok());
        assert_eq!(msg.frame_id().get_id(), 0x200);
    }
    #[test]
    fn frame_id_invalid_preserves() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetFrameId(
            FrameId::try_from(0x800).unwrap(),
        ));

        assert!(result.is_err());
        assert_eq!(msg.frame_id().get_id(), 0x100);
    }

    #[test]
    fn payload_valid_updates() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetPayloadLength(8));

        assert!(result.is_ok());
        assert_eq!(msg.payload_length(), 8);
    }

    #[test]
    fn payload_invalid_preserves() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetPayloadLength(9));

        assert!(result.is_err());
        assert_eq!(msg.payload_length(), 4);
    }

    #[test]
    fn frame_format_valid_updates() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetFrameFormat(FrameFormat::ExtendedCan));

        assert!(result.is_ok());
        assert_eq!(msg.frame_format(), FrameFormat::ExtendedCan);
    }

    #[test]
    fn frame_format_invalid_id_preserves() {
        let mut msg = Message::new("Test", 0x1000, FrameFormat::ExtendedCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetFrameFormat(FrameFormat::StandardCan));

        assert!(result.is_err());
        assert_eq!(msg.frame_format(), FrameFormat::ExtendedCan);
    }

    #[test]
    fn frame_format_invalid_payload_preserves() {
        let mut msg = Message::new("Test", 0x100, FrameFormat::ExtendedCanFd, 32, None).unwrap();

        let result = msg.apply_command(MessageCommand::SetFrameFormat(FrameFormat::ExtendedCan));

        assert!(result.is_err());
        assert_eq!(msg.frame_format(), FrameFormat::ExtendedCanFd);
    }

    #[test]
    fn rename_updates_name() {
        let mut msg = Message::new("Old", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let result = msg.apply_command(MessageCommand::Rename(String::from("New")));

        assert!(result.is_ok());
        assert_eq!(msg.name(), "New");
    }
}
