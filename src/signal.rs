use crate::node::NodeId;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    name: String,
    layout: SignalLayout,
    value: SignalValue,
    unit: String,
    receivers: Vec<NodeId>,
    multiplexer: bool,
    multiplexed_by: Option<SignalMultiplexer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalLayout {
    start_bit: u16,
    length: u16,
    byte_order: SignalLayoutEndianness,
    sign: SignalLayoutSign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SignalValue {
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalLayoutEndianness {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalLayoutSign {
    Signed,
    Unsigned,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct SignalId {
    id: u64,
}

impl SignalId {
    pub(crate) fn new(input: u64) -> Self {
        Self { id: input }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SignalMultiplexer {
    multiplexer: SignalId,
    ranges: Vec<RangeInclusive<u64>>,
}
