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
