use crate::{
    frame_id::{FrameFormat, FrameId, FrameIdError},
    node::NodeId,
    signal::SignalId,
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

#[derive(Clone)]
pub struct Message {
    frame_id: FrameId,
    frame_format: FrameFormat,
    name: String,
    payload_length: u8,
    sender: Option<NodeId>,
    signals: Vec<SignalId>,
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
            signals: Vec::new(),
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
    pub(crate) fn add_signal_id(&mut self, id: SignalId) -> Result<(), MessageError> {
        if self.signals.contains(&id) {
            return Err(MessageError::SignalAlreadyPresent(id));
        }
        self.signals.push(id);
        Ok(())
    }
    pub(crate) fn remove_signal_id(&mut self, id: SignalId) -> Result<(), MessageError> {
        if let Some(pos) = self.signals.iter().position(|s_id| *s_id == id) {
            self.signals.remove(pos);
            Ok(())
        } else {
            Err(MessageError::NoSignalWithGivenId(id))
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum MessageError {
    InvalidFrameId(FrameIdError),
    FrameTooLargeForFormat,
    PayloadTooLong,
    SignalAlreadyPresent(SignalId),
    NoSignalWithGivenId(SignalId),
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
mod tests;
