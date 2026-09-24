use crate::{message::MessageId, node::NodeId};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    name: String,
    layout: SignalLayout,
    value: SignalValue,
    unit: Option<String>,
    receivers: Vec<NodeId>,
    multiplexer: bool,
    multiplexed_by: Option<SignalMultiplexer>,
    message_id: MessageId,
}

impl Signal {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_layout(&self) -> &SignalLayout {
        &self.layout
    }
    pub fn get_value(&self) -> &SignalValue {
        &self.value
    }
    pub fn get_unit(&self) -> Option<&String> {
        self.unit.as_ref()
    }
    pub fn get_receivers(&self) -> &Vec<NodeId> {
        &self.receivers
    }
    pub fn is_multiplexer(&self) -> bool {
        self.multiplexer
    }
    pub fn get_signal_multiplexer(&self) -> Option<&SignalMultiplexer> {
        self.multiplexed_by.as_ref()
    }
    pub fn get_message_id(&self) -> MessageId {
        self.message_id
    }
    pub(crate) fn apply_command(&mut self, cmd: SignalCommand) -> Result<(), SignalError> {
        match cmd {
            SignalCommand::Rename(new_name) => {
                self.name = new_name;
                Ok(())
            }
            SignalCommand::ChangeLayout(new_layout) => {
                self.layout = new_layout;
                Ok(())
            }
            SignalCommand::ChangeUnit(new_unit) => {
                self.unit = new_unit;
                Ok(())
            }
            SignalCommand::ChangeValue(new_value) => {
                if Signal::is_value_valid(&new_value) {
                    self.value = new_value;
                    Ok(())
                } else {
                    Err(SignalError::MinMaxError)
                }
            }
            SignalCommand::AddReceiver(n_id) => {
                if self.receivers.contains(&n_id) {
                    Err(SignalError::ReceiverAlreadyPresent)
                } else {
                    self.receivers.push(n_id);
                    Ok(())
                }
            }
            SignalCommand::RemoveReceiver(n_id) => {
                if let Some(pos) = self.receivers.iter().position(|x| *x == n_id) {
                    self.receivers.remove(pos);
                    Ok(())
                } else {
                    Err(SignalError::ReceiverNotPresent)
                }
            }
            SignalCommand::SetAsMultiplexer => {
                self.multiplexer = true;
                Ok(())
            }
            SignalCommand::RemoveAsMultiplexer => {
                self.multiplexer = false;
                Ok(())
            }
            SignalCommand::AddMultiplexer(new_multiplexer) => {
                if self.multiplexed_by.is_none() {
                    self.multiplexed_by = Some(new_multiplexer);
                    Ok(())
                } else {
                    Err(SignalError::MultiplexerAlreadyPresent)
                }
            }
            SignalCommand::RemoveMultiplexer(old_multiplexer) => {
                if self.multiplexed_by.is_none() {
                    return Err(SignalError::MultiplexerSignalNotPresent);
                }
                if let Some(sig_mul) = &self.multiplexed_by {
                    if sig_mul.get_id() != old_multiplexer {
                        return Err(SignalError::MultiplexerSignalNotPresent);
                    }
                }
                self.multiplexed_by = None;
                Ok(())
            }
            SignalCommand::AddMultiplexerRange(m_range) => {
                if let Some(mul_sig) = &mut self.multiplexed_by {
                    if mul_sig.get_ranges().contains(&m_range) {
                        return Err(SignalError::MultiplexerSignalRangeAlreadyPresent);
                    }
                    mul_sig.get_ranges_mut().push(m_range);
                    Ok(())
                } else {
                    Err(SignalError::MultiplexerSignalNotPresent)
                }
            }
            SignalCommand::RemoveMultiplexerRange(m_range) => {
                if let Some(mul_sig) = &mut self.multiplexed_by {
                    if let Some(pos) = mul_sig.get_ranges().iter().position(|x| *x == m_range) {
                        mul_sig.get_ranges_mut().remove(pos);
                        return Ok(());
                    }
                    Err(SignalError::MultiplexerSignalRangeNotPresent)
                } else {
                    Err(SignalError::MultiplexerSignalNotPresent)
                }
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn is_value_valid(value: &SignalValue) -> bool {
        value.min < value.max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SignalLayout {
    start_bit: u16,
    length: u16,
    byte_order: SignalLayoutEndianness,
    sign: SignalLayoutSign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SignalValue {
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalLayoutEndianness {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalLayoutSign {
    Signed,
    Unsigned,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct SignalId {
    id: u64,
}

impl SignalId {
    pub(crate) fn new(input: u64) -> Self {
        Self { id: input }
    }
}
impl From<u64> for SignalId {
    fn from(input: u64) -> SignalId {
        SignalId::new(input)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SignalMultiplexer {
    multiplexer_id: SignalId,
    ranges: Vec<RangeInclusive<u64>>,
}

impl SignalMultiplexer {
    pub fn get_id(&self) -> SignalId {
        self.multiplexer_id
    }
    pub fn get_ranges(&self) -> &Vec<RangeInclusive<u64>> {
        self.ranges.as_ref()
    }
    pub(crate) fn get_ranges_mut(&mut self) -> &mut Vec<RangeInclusive<u64>> {
        self.ranges.as_mut()
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum SignalCommand {
    Rename(String),
    ChangeLayout(SignalLayout),
    ChangeValue(SignalValue),
    ChangeUnit(Option<String>),
    AddReceiver(NodeId),
    RemoveReceiver(NodeId),
    SetAsMultiplexer,
    RemoveAsMultiplexer,
    AddMultiplexer(SignalMultiplexer),
    RemoveMultiplexer(SignalId),
    AddMultiplexerRange(RangeInclusive<u64>),
    RemoveMultiplexerRange(RangeInclusive<u64>),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SignalError {
    InvalidLength,
    MinMaxError,
    ReceiverAlreadyPresent,
    ReceiverNotPresent,
    MultiplexerAlreadyPresent,
    MultiplexerSignalNotPresent,
    MultiplexerSignalRangeAlreadyPresent,
    MultiplexerSignalRangeNotPresent,
}
