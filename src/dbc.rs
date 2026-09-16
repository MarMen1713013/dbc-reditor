use std::collections::HashMap;

use crate::{
    message::{Message, MessageCommand, MessageError, MessageId},
    node::{Node, NodeCommand, NodeError, NodeId},
};

pub struct Dbc {
    nodes: HashMap<NodeId, Node>,
    next_node_id: u64,
    messages: HashMap<MessageId, Message>,
    next_msg_id: u64,
}

impl Dbc {
    pub fn new() -> Self {
        Self {
            next_msg_id: 0,
            next_node_id: 0,
            nodes: HashMap::new(),
            messages: HashMap::new(),
        }
    }
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let n_id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;

        self.nodes.insert(n_id, node);
        n_id
    }
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn modify_node(&mut self, id: NodeId, cmd: NodeCommand) -> Result<(), DbcError> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.apply_command(cmd)?;
            Ok(())
        } else {
            Err(DbcError::NodeNotFound)
        }
    }
    pub fn add_message(&mut self, msg: Message) -> Result<MessageId, DbcError> {
        if let Some(n_id) = msg.sender() {
            if self.get_node(n_id).is_none() {
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
        if let Some(msg) = self.messages.get_mut(&id) {
            if let MessageCommand::SetSender(Some(n_id)) = &cmd {
                if !self.nodes.contains_key(n_id) {
                    return Err(DbcError::MessageSenderNotAvailable);
                }
            }
            msg.apply_command(cmd)?;
            Ok(())
        } else {
            Err(DbcError::MessageNotFound)
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum DbcError {
    MessageNotFound,
    MessageSenderNotAvailable,
    Message(MessageError),
    NodeNotFound,
    Node(NodeError),
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

        let id = dbc.add_node(Node::new("Node1"));

        let node = dbc.get_node(id);

        assert!(node.is_some());
        assert_eq!(node.unwrap().name(), "Node1");
    }

    #[test]
    fn modify_node_updates() {
        let mut dbc = Dbc::new();

        let id = dbc.add_node(Node::new("Old"));

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

        let node_id = dbc.add_node(Node::new("ECU"));

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

        let node_id = dbc.add_node(Node::new("ECU"));

        let msg = Message::new("Test", 0x100, FrameFormat::StandardCan, 4, None).unwrap();

        let msg_id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(msg_id, MessageCommand::SetSender(Some(node_id)));

        assert!(result.is_ok());
        assert_eq!(dbc.get_message(msg_id).unwrap().sender(), Some(node_id));
    }

    #[test]
    fn set_invalid_sender_preserves_old_sender() {
        let mut dbc = Dbc::new();

        let old_sender = dbc.add_node(Node::new("ECU"));

        let msg =
            Message::new("Test", 0x100, FrameFormat::StandardCan, 4, Some(old_sender)).unwrap();

        let msg_id = dbc.add_message(msg).unwrap();

        let result = dbc.modify_message(msg_id, MessageCommand::SetSender(Some(NodeId::new(100))));

        assert_eq!(result, Err(DbcError::MessageSenderNotAvailable));

        assert_eq!(dbc.get_message(msg_id).unwrap().sender(), Some(old_sender));
    }
}
