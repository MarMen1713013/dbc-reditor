use std::collections::HashMap;

use crate::{
    message::{Message, MessageCommand, MessageError, MessageId},
    node::{Node, NodeCommand, NodeError, NodeId},
    signal::{Signal, SignalId},
};

pub struct Dbc {
    nodes: HashMap<NodeId, Node>,
    next_node_id: u64,
    messages: HashMap<MessageId, Message>,
    next_msg_id: u64,
    signals: HashMap<SignalId, Signal>,
    next_signal_id: u64,
}

impl Dbc {
    pub fn new() -> Self {
        Self {
            next_msg_id: 0,
            next_node_id: 0,
            next_signal_id: 0,
            nodes: HashMap::new(),
            messages: HashMap::new(),
            signals: HashMap::new(),
        }
    }
    pub fn add_node(&mut self, node: Node) -> Result<NodeId, DbcError> {
        if self.node_exists(&node, None) {
            return Err(DbcError::NodeAlreadyExists);
        }
        let n_id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;

        self.nodes.insert(n_id, node);
        Ok(n_id)
    }
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn modify_node(&mut self, id: NodeId, cmd: NodeCommand) -> Result<(), DbcError> {
        if let Some(node) = self.nodes.get(&id) {
            let mut test = node.clone();
            test.apply_command(cmd)?;
            if self.node_exists(&test, Some(id)) {
                return Err(DbcError::NodeAlreadyExists);
            }
            self.nodes.insert(id, test);
            Ok(())
        } else {
            Err(DbcError::NodeNotFound)
        }
    }
    pub fn add_message(&mut self, msg: Message) -> Result<MessageId, DbcError> {
        if self.message_exists(&msg, None) {
            return Err(DbcError::MessageAlreadyExists);
        }
        if let Some(n_id) = msg.sender() {
            if self.absent_node_id(&n_id) {
                return Err(DbcError::MessageSenderNotAvailable);
            }
        }
        let m_id = MessageId::new(self.next_msg_id);
        self.messages.insert(m_id, msg);
        self.next_msg_id += 1;

        Ok(m_id)
    }
    pub fn get_message(&self, m_id: MessageId) -> Option<&Message> {
        self.messages.get(&m_id)
    }
    pub fn modify_message(&mut self, id: MessageId, cmd: MessageCommand) -> Result<(), DbcError> {
        if let Some(msg) = self.messages.get(&id) {
            let mut test = msg.clone();
            test.apply_command(cmd)?;
            if self.message_exists(&test, Some(id)) {
                return Err(DbcError::MessageAlreadyExists);
            }
            if let Some(n_id) = test.sender() {
                if self.absent_node_id(&n_id) {
                    return Err(DbcError::MessageSenderNotAvailable);
                }
            }
            self.messages.insert(id, test);
            Ok(())
        } else {
            Err(DbcError::MessageNotFound)
        }
    }
    pub fn add_signal(&mut self, signal: Signal) -> Result<SignalId, DbcError> {
        Ok(SignalId::from(0))
    }
    pub(crate) fn node_exists(&self, new_node: &Node, exclude: Option<NodeId>) -> bool {
        self.nodes
            .iter()
            .any(|(n_id, test)| test.name() == new_node.name() && Some(*n_id) != exclude)
    }
    pub(crate) fn message_exists(&self, new_message: &Message, exclude: Option<MessageId>) -> bool {
        self.messages.iter().any(|(m_id, test)| {
            test.frame_id() == new_message.frame_id()
                && test.frame_format().class() == new_message.frame_format().class()
                && Some(*m_id) != exclude
        })
    }
    pub(crate) fn absent_node_id(&self, n_id: &NodeId) -> bool {
        !self.nodes.contains_key(n_id)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum DbcError {
    MessageNotFound,
    MessageAlreadyExists,
    MessageSenderNotAvailable,
    Message(MessageError),
    NodeNotFound,
    NodeAlreadyExists,
    Node(NodeError),
    SignalNotFound,
    SignalAlreadyExists,
}

impl From<MessageError> for DbcError {
    fn from(input: MessageError) -> DbcError {
        DbcError::Message(input)
    }
}

impl From<NodeError> for DbcError {
    fn from(input: NodeError) -> DbcError {
        DbcError::Node(input)
    }
}
#[cfg(test)]
mod tests;
