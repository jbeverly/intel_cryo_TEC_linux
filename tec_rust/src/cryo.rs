use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use mockall::predicate::*;
use mockall::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

struct SerialPortImpl {
    port: Box<dyn serialport::SerialPort>,
}

#[automock]
pub trait SerialPort {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>;
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()>;
}

impl SerialPortImpl {
    fn new(path: &str) -> Self {
        SerialPortImpl {
            port: serialport::new(path, 115200)
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One)
                .parity(serialport::Parity::None)
                .timeout(Duration::from_millis(1000))
                .open()
                .unwrap(),
        }
    }
}

impl SerialPort for SerialPortImpl {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.port.write_all(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.port.read_exact(buf)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum OpCode {
    Heartbeat = 0,
    GetTecTemperature = 1,
    GetHumidity = 2,
    GetDewPoint = 3,
    GetSetPointOffset = 4,
    GetPCoefficient = 5,
    GetICoefficient = 6,
    GetDCoefficient = 7,
    GetTecPowerLevel = 8,
    GetHwVersion = 9,
    GetFwVersion = 10,
    SetSetPointOffset = 20,
    SetPCoefficient = 21,
    SetICoefficient = 22,
    SetDCoefficient = 23,
    SetLowPowerMode = 24,
    SetCPUTemp = 25,
    SetNtcCoefficient = 26,
    GetNtcCoefficient = 27,
    SetTempSensorMode = 28,
    SetTecPowerLevel = 29,
    ResetBoard = 30,
    GetBoardTemp = 31,
    GetVoltageAndCurrent = 34,
    GetTecVoltage = 35,
    GetTecCurrent = 36,
}

static HEARTBEAT_STATUS: &'static [(u8, &'static str)] = &[
    (0, "Board initialisation completed"),
    (1, "Power supply OK"),
    (2, "TEC thermistor reading in range"),
    (3, "Humidity sensor reading in range"),
    (4, "Last received command OK"),
    (5, "Last received command had a bad CRC"),
    (6, "Last received command is incomplete"),
    (7, "Failsafe has been activated"),
    (8, "PID constants were loaded and accepted"),
    (9, "PID constants were rejected"),
    (10, "Set point for the PID is out of range"),
    (11, "Default set point was loaded"),
    (12, "PID is running"),
    (13, "Overcurrent protection has been triggered"),
    (14, "Board temperature is in range"),
    (15, "TEC connection OK"),
    (16, "Low power mode enabled"),
    (17, "Using NTC thermistor"),
];

#[derive(Debug, PartialEq, Clone)]
pub enum HandlerResult {
    Float(f32),
    TupleFloat(f32, f32),
    VecStr(Vec<&'static str>),
    Int(u32),
}

fn handle_data(op: OpCode, x: u32) -> HandlerResult {
    match op {
        OpCode::GetTecVoltage => HandlerResult::Float((x as f32) / 21.25),
        OpCode::GetTecCurrent => HandlerResult::Float((x as f32) / 4.6545),
        OpCode::GetVoltageAndCurrent => {
            HandlerResult::TupleFloat((x & 0xFF) as f32 / 21.25, ((x >> 8) & 0xFF) as f32 / 4.6545)
        }
        OpCode::Heartbeat => HandlerResult::VecStr(
            HEARTBEAT_STATUS
                .iter()
                .filter(|&&(k, _)| (2u32.pow(k as u32)) & x != 0)
                .map(|&(_, v)| v)
                .collect::<Vec<_>>(),
        ),
        OpCode::GetTecTemperature | OpCode::GetDewPoint | OpCode::GetHumidity => {
            HandlerResult::Float(unpack_int_to_float(x))
        }
        _ => HandlerResult::Int(x),
    }
}
fn pretty_print_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn unpack_int_to_float(x: u32) -> f32 {
    let mut bytes = [0u8; 4];
    (&mut bytes[..]).write_u32::<LittleEndian>(x).unwrap();
    (&bytes[..]).read_f32::<LittleEndian>().unwrap()
}

pub fn unpack_float_to_int(x: f32) -> u32 {
    let mut bytes = [0u8; 4];
    (&mut bytes[..]).write_f32::<LittleEndian>(x).unwrap();
    (&bytes[..]).read_u32::<BigEndian>().unwrap()
}

fn crc16(data: &[u8], poly: u16, crc_start: u16) -> u16 {
    let mut crc = crc_start;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ poly;
            } else {
                crc <<= 1;
            }
        }
    }
    crc & 0xFFFF
}

pub fn send_data(opcode: u8, operand: u32, s: &mut Box<dyn SerialPort>) {
    let mut data_header = vec![0xAA];
    data_header.write_u8(opcode).unwrap();
    data_header.write_u32::<BigEndian>(operand).unwrap();
    let crc = crc16(&data_header, 0x1021, 0);
    data_header.write_u16::<LittleEndian>(crc).unwrap();
    s.write_all(&data_header).unwrap();
    println!("WROTE: {}", pretty_print_bytes(&data_header));
}

pub fn read_data(s: &mut Box<dyn SerialPort>) -> (OpCode, HandlerResult) {
    let mut data = vec![0u8; 8];
    s.read_exact(&mut data).unwrap();

    println!("READ: {}", pretty_print_bytes(&data));

    let opcode = data[1];
    let operand = (&data[2..6]).read_u32::<LittleEndian>().unwrap();
    let checksum = (&data[6..8]).read_u16::<LittleEndian>().unwrap();
    let expected = crc16(&data[0..6], 0x1021, 0);
    if expected != checksum {
        panic!(
            "Did not match checksum expected: {:x}, got {:x}",
            expected, checksum
        );
    }
    let op = OpCode::from_u8(opcode - 127).unwrap();
    (op, handle_data(op, operand))
}
pub fn submit_command_s(
    op: OpCode,
    data: u32,
    s: &mut Box<dyn SerialPort>,
) -> (OpCode, HandlerResult) {
    send_data(op as u8, data, s);
    read_data(s)
}

pub fn submit_command(op: OpCode, data: u32) -> (OpCode, HandlerResult) {
    let path = "/dev/ttyUSB0";
    let mut s: Box<dyn SerialPort> = Box::new(SerialPortImpl::new(path));
    submit_command_s(op, data, &mut s)
}

fn monitor_loop() {
    fs::create_dir_all("/var/run/intel_cryo_tec").unwrap();
    loop {
        let mut heartbeat_details = HashMap::new();
        for &(_, v) in HEARTBEAT_STATUS.iter() {
            heartbeat_details.insert(v, false);
        }

        let (_, heartbeat_result) = submit_command(OpCode::Heartbeat, 0);
        if let HandlerResult::VecStr(heartbeats) = heartbeat_result {
            for v in heartbeats {
                heartbeat_details.insert(v, true);
            }
        }

        let (voltage, current) = match submit_command(OpCode::GetVoltageAndCurrent, 0) {
            (_, HandlerResult::TupleFloat(v, c)) => (v, c),
            _ => (0.0, 0.0),
        };

        let dewpoint = match submit_command(OpCode::GetDewPoint, 0) {
            (_, HandlerResult::Float(d)) => d,
            _ => 0.0,
        };

        let temperature = match submit_command(OpCode::GetTecTemperature, 0) {
            (_, HandlerResult::Float(t)) => t,
            _ => 0.0,
        };

        let power_level = match submit_command(OpCode::GetTecPowerLevel, 0) {
            (_, HandlerResult::Int(p)) => p,
            _ => 0,
        };

        let humidity = match submit_command(OpCode::GetHumidity, 0) {
            (_, HandlerResult::Float(h)) => h,
            _ => 0.0,
        };

        let data = json!({
            "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            "heartbeat": heartbeat_details,
            "voltage": voltage,
            "current": current,
            "dewpoint": dewpoint,
            "temperature": temperature,
            "power_level": power_level,
            "humidity": humidity,
        });

        fs::write("/var/run/intel_cryo_tec/status.json", data.to_string()).unwrap();
        sleep(Duration::from_secs(1));
    }
}

pub fn set_cryo_mode() {
    submit_command(OpCode::SetSetPointOffset, unpack_float_to_int(2.0));
    submit_command(OpCode::SetPCoefficient, unpack_float_to_int(100.0));
    submit_command(OpCode::SetICoefficient, unpack_float_to_int(1.0));
    submit_command(OpCode::SetDCoefficient, 0);
    submit_command(OpCode::SetLowPowerMode, 0);
}
pub fn run() {
    submit_command(OpCode::ResetBoard, 0);
    sleep(Duration::from_secs(5));
    set_cryo_mode();
    monitor_loop();
}
