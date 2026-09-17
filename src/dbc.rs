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
mod tests {
    use super::*;

    use crate::{
        frame_id::{FrameFormat, FrameId},
        message::MessageCommand,
        node::NodeCommand,
    };

    #[test]
    fn add_node_returns_retrievable_node() {
        let mut dbc = Dbc::new();

        let id = dbc.add_node(Node::new("Node1")).unwrap();

        let node = dbc.get_node(id);

        assert!(node.is_some());
        assert_eq!(node.unwrap().name(), "Node1");
    }

    #[test]
    fn modify_node_updates() {
        let mut dbc = Dbc::new();

        let id = dbc.add_node(Node::new("Old")).unwrap();

        let result = dbc.modify_node(id, NodeCommand::Rename(String::from("New")));

        assert!(result.is_ok());
        assert_eq!(dbc.get_node(id).unwrap().name(), "New");
    }

    #[test]
    fn modify_missing_node_fails() {
        let mut dbc = Dbc::new();

        let result = dbc.modify_node(NodeId::new(100), NodeCommand::Rename(String::from("New")));

        assert_eq!(result, Err(DbcError::NodeNotFound));
    }

    #[test]
    fn add_message_returns_retrievable_message() {
        let mut dbc = Dbc::new();

        let msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let id = dbc.add_message(msg).unwrap();

        let msg = dbc.get_message(id);

        assert!(msg.is_some());
        assert_eq!(msg.unwrap().name(), "Test");
    }

    #[test]
    fn add_message_with_valid_sender_succeeds() {
        let mut dbc = Dbc::new();

        let node_id = dbc.add_node(Node::new("ECU")).unwrap();

        let msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, Some(node_id)).unwrap();

        let result = dbc.add_message(msg);

        assert!(result.is_ok());
    }

    #[test]
    fn add_message_with_invalid_sender_fails() {
        let mut dbc = Dbc::new();

        let msg = Message::new(
            "Test",
            0x100,
            FrameFormat::StandardCan,
            4,
            Some(NodeId::new(100)),
        )
        .unwrap();

        let result = dbc.add_message(msg);

        assert_eq!(result, Err(DbcError::MessageSenderNotAvailable));
    }

    #[test]
    fn modify_message_updates() {
        let mut dbc = Dbc::new();

        let msg = Message::new("Old", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(id, MessageCommand::Rename(String::from("New")));

        assert!(result.is_ok());
        assert_eq!(dbc.get_message(id).unwrap().name(), "New");
    }

    #[test]
    fn modify_missing_message_fails() {
        let mut dbc = Dbc::new();

        let result = dbc.modify_message(
            MessageId::new(100),
            MessageCommand::Rename(String::from("New")),
        );

        assert_eq!(result, Err(DbcError::MessageNotFound));
    }

    #[test]
    fn modify_message_propagates_message_error() {
        let mut dbc = Dbc::new();

        let msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(
            id,
            MessageCommand::SetFrameId(FrameId::try_from(0x800).unwrap()),
        );

        assert_eq!(
            result,
            Err(DbcError::Message(MessageError::FrameTooLargeForFormat))
        );

        assert_eq!(dbc.get_message(id).unwrap().frame_id().get_id(), 0x100);
    }

    #[test]
    fn set_valid_sender_updates_message() {
        let mut dbc = Dbc::new();

        let node_id = dbc.add_node(Node::new("ECU")).unwrap();

        let msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let msg_id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(msg_id, MessageCommand::SetSender(Some(node_id)));

        assert!(result.is_ok());
        assert_eq!(dbc.get_message(msg_id).unwrap().sender(), Some(node_id));
    }

    #[test]
    fn set_invalid_sender_preserves_old_sender() {
        let mut dbc = Dbc::new();

        let old_sender = dbc.add_node(Node::new("ECU")).unwrap();

        let msg =
            Message::new("Test", 0x100, FrameFormat::StandardCan, 4, Some(old_sender)).unwrap();

        let msg_id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(msg_id, MessageCommand::SetSender(Some(NodeId::new(100))));

        assert_eq!(result, Err(DbcError::MessageSenderNotAvailable));

        assert_eq!(dbc.get_message(msg_id).unwrap().sender(), Some(old_sender));
    }

    #[test]
    fn add_duplicate_node_fails() {
        let mut dbc = Dbc::new();

        dbc.add_node(Node::new("ECU")).unwrap();

        let result = dbc.add_node(Node::new("ECU"));

        assert_eq!(result, Err(DbcError::NodeAlreadyExists));
    }

    #[test]
    fn rename_node_to_existing_name_fails() {
        let mut dbc = Dbc::new();

        dbc.add_node(Node::new("ECU1")).unwrap();
        let id = dbc.add_node(Node::new("ECU2")).unwrap();

        let result = dbc.modify_node(id, NodeCommand::Rename(String::from("ECU1")));

        assert_eq!(result, Err(DbcError::NodeAlreadyExists));
        assert_eq!(dbc.get_node(id).unwrap().name(), "ECU2");
    }

    #[test]
    fn add_duplicate_message_fails() {
        let mut dbc = Dbc::new();

        let msg1 = Message::new("Msg1", 0x100, FrameFormat::StandardCan, 8, None).unwrap();

        let msg2 = Message::new("Msg2", 0x100, FrameFormat::StandardCanFd, 8, None).unwrap();

        dbc.add_message(msg1).unwrap();

        let result = dbc.add_message(msg2);

        assert_eq!(result, Err(DbcError::MessageAlreadyExists));
    }

    #[test]
    fn modify_message_to_existing_id_fails() {
        let mut dbc = Dbc::new();

        let msg1 = Message::new("Msg1", 0x100, FrameFormat::StandardCan, 8, None).unwrap();

        let msg2 = Message::new("Msg2", 0x200, FrameFormat::StandardCan, 8, None).unwrap();

        dbc.add_message(msg1).unwrap();
        let id2 = dbc.add_message(msg2).unwrap();

        let result = dbc.modify_message(
            id2,
            MessageCommand::SetFrameId(FrameId::try_from(0x100).unwrap()),
        );

        assert_eq!(result, Err(DbcError::MessageAlreadyExists));
        assert_eq!(dbc.get_message(id2).unwrap().frame_id().get_id(), 0x200);
    }
}
