#[cfg(test)]
mod tests {
    use intel_cryo_tec_monitor::cryo::{
        read_data, send_data, submit_command_s, unpack_float_to_int, unpack_int_to_float,
        HandlerResult, MockSerialPort, OpCode, SerialPort,
    };
    use mockall::predicate::*;

    #[test]
    fn test_serial_port_write() {
        let mut mock = Box::new(MockSerialPort::new());

        // Set expectations
        mock.expect_write_all()
            .with(eq(vec![0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A, 0x67]))
            .returning(|_| Ok(()));

        let mut mock: Box<dyn SerialPort> = mock;
        send_data(OpCode::Heartbeat as u8, 0, &mut mock);
    }

    #[test]
    fn test_serial_port_read() {
        let mut mock = Box::new(MockSerialPort::new());
        mock.expect_read_exact()
            .withf(|buf: &[u8]| buf.len() == 8)
            .returning(|buf| {
                buf.copy_from_slice(&[0xAA, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x55, 0x1f]);
                Ok(())
            });
        let mut mock: Box<dyn SerialPort> = mock;

        let r: (OpCode, HandlerResult) = read_data(&mut mock);
        assert_eq!(r.0, OpCode::Heartbeat);
        assert_eq!(r.1, HandlerResult::VecStr([].to_vec()))
    }

    #[test]
    fn test_floating_point_pack() {
        let mut mock = Box::new(MockSerialPort::new());

        // Set expectations
        mock.expect_write_all()
            .with(eq(vec![0xAA, 0x01, 0x00, 0x00, 0x00, 0x00, 0x7b, 0xcd]))
            .returning(|_| Ok(()));

        mock.expect_read_exact()
            .withf(|buf: &[u8]| buf.len() == 8)
            .returning(|buf| {
                buf.copy_from_slice(&[0xAA, 0x80, 0x29, 0x5c, 0x5b, 0x41, 0xcc, 0x20]);
                Ok(())
            });
        let mut mock: Box<dyn SerialPort> = mock;

        let r: (OpCode, HandlerResult) =
            submit_command_s(OpCode::GetTecTemperature, 0x00, &mut mock);
        assert_eq!(r.0, OpCode::GetTecTemperature);
        assert_eq!(r.1, HandlerResult::Float(13.71))
    }

    #[test]
    fn test_floating_point_pack_100() {
        let mut mock = Box::new(MockSerialPort::new());

        // Set expectations
        mock.expect_write_all()
            .with(eq(vec![0xAA, 0x01, 0x00, 0x00, 0x00, 0x00, 0x7b, 0xcd]))
            .returning(|_| Ok(()));

        mock.expect_read_exact()
            .withf(|buf: &[u8]| buf.len() == 8)
            .returning(|buf| {
                buf.copy_from_slice(&[0xAA, 0x80, 0x00, 0x00, 0xc8, 0x42, 0x81, 0xb2]);
                Ok(())
            });
        let mut mock: Box<dyn SerialPort> = mock;

        let r: (OpCode, HandlerResult) =
            submit_command_s(OpCode::GetTecTemperature, 0x00, &mut mock);
        assert_eq!(r.0, OpCode::GetTecTemperature);
        assert_eq!(r.1, HandlerResult::Float(100.0))
    }
}
