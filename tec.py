#!/usr/bin/env python3
import serial
import struct
import json
import os
import os.path
from enum import Enum
from pprint import pprint
from typing import Tuple, Any
from time import sleep
 
class OpCode(Enum):
    heartbeat = 0
    getTecTemperature = 1
    getHumidity = 2
    getDewPoint = 3
    getSetPointOffset = 4
    getPCoefficient = 5
    getICoefficient = 6
    getDCoefficient = 7
    getTecPowerLevel = 8
    getHwVersion = 9
    getFwVersion = 10
    setSetPointOffset = 20
    setPCoefficient = 21
    setICoefficient = 22
    setDCoefficient = 23
    setLowPowerMode = 24
    setCPUTemp = 25
    setNtcCoefficient = 26
    getNtcCoefficient = 27
    setTempSensorMode = 28
    setTecPowerLevel = 29
    resetBoard = 30
    getBoardTemp = 31
    getVoltageAndCurrent = 34
    getTecVoltage = 35
    getTecCurrent = 36
 
 
heartbeat_status = {
    0: "Board initialisation completed",
    1: "Power supply OK",
    2: "TEC thermistor reading in range",
    3: "Humidity sensor reading in range",
    4: "Last received command OK",
    5: "Last received command had a bad CRC",
    6: "Last received command is incomplete",
    7: "Failsafe has been activated",
    8: "PID constants were loaded and accepted",
    9: "PID constants were rejected",
    10: "Set point for the PID is out of range",
    11: "Default set point was loaded",
    12: "PID is running",
    13: "Overcurrent protection has been triggered",
    14: "Board temperature is in range",
    15: "TEC connection OK",
    16: "Low power mode enabled",
    17: "Using NTC thermistor",
}
 
def unpack_float(x):
    return float(struct.unpack('<f', struct.pack('<I', x))[0])
 
def unpack_int(x):
    return int(struct.unpack('>I', struct.pack('<f', x))[0])
 
 
handlers = {
    OpCode.getTecVoltage: lambda x: float(x) / 21.25,
    OpCode.getTecCurrent: lambda x: float(x) / 4.6545,
    OpCode.getVoltageAndCurrent: lambda x: (handlers[OpCode.getTecVoltage](x & 0xFF), handlers[OpCode.getTecCurrent]((x >> 8) & 0xFF)),
    OpCode.heartbeat: lambda x: [v for k, v in heartbeat_status.items() if (2**k) & x],
    OpCode.getTecTemperature: lambda x: unpack_float(x),
    OpCode.getDewPoint: lambda x: unpack_float(x),
    OpCode.getHumidity: lambda x: unpack_float(x),
}
 
def crc16(data: bytes, poly=0x1021, crc=0x0000):
    '''
    CRC-16-CCITT Algorithm
    '''
    data = bytearray(data)
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = (crc << 1) ^ poly
            else:
                crc <<= 1
            crc &= 0xFFFF
    return crc
 
 
 
def send_data(opcode: int, operand: int, s: serial.Serial): 
    data_header = struct.pack('<B', 0xAA)
    data_opcode = struct.pack('<B', opcode)
    data_operand = struct.pack('>I', operand)
    payload = data_header + data_opcode + data_operand
    crc = crc16(payload)
    checksum = struct.pack('<H', crc)
    s.write(payload + checksum)
 
def read_data(s: serial.Serial) -> Tuple[OpCode, Any]:
    data = s.read(8)
 
    _, opcode, operand, checksum = struct.unpack('<BBIH', data)
 
    expected = struct.pack('<H', crc16(data[0:6]))
    if expected != data[6:8]:
        print(f"Did not match checksum expected: {expected.hex()}, got {hex(checksum)}")
 
    opcode = OpCode(opcode - 127)

    result = operand
    if opcode in handlers:
        result = handlers[opcode](operand)
    return opcode, result
 
def submit_command(opcode: OpCode, data: int):
    s = serial.Serial('/dev/ttyUSB0', 115200, bytesize=8, stopbits=1, parity='N')
    send_data(opcode.value, data, s)
    opcode, result = read_data(s)
    pprint((opcode, result))
    return opcode, result
 
 
def monitor_loop():
    os.makedirs("/var/run/intel_cryo_tec", exist_ok=True)

    while True:
        _, heartbeat = submit_command(OpCode.heartbeat, 0x00)
        _, (voltage, current) = submit_command(OpCode.getVoltageAndCurrent, 0x00)
        _, dewpoint = submit_command(OpCode.getDewPoint, 0x00)
        _, temperature = submit_command(OpCode.getTecTemperature, 0x00)
        _, power_level = submit_command(OpCode.getTecPowerLevel, 0x00)
        _, humidity = submit_command(OpCode.getHumidity, 0x00)

        heartbeat_details = {x: False for x in heartbeat_status.values() }
        heartbeat_details.update({x: True for x in heartbeat})

        with open("/var/run/intel_cryo_tec/status.json", "w") as f:
            json.dump({
                "heartbeat" : heartbeat_details,
                "voltage" : voltage,
                "current" : current,
                "dewpoint" : dewpoint,
                "temperature" : temperature,
                "power_level" : power_level,
                "humidity" : humidity,
            }, f)
        sleep(1)
 
def set_cryo_mode():
    submit_command(OpCode.setSetPointOffset, unpack_int(2.0))
    submit_command(OpCode.setPCoefficient, unpack_int(100.0))
    submit_command(OpCode.setICoefficient, unpack_int(1.0))
    submit_command(OpCode.setDCoefficient, 0x00)
    submit_command(OpCode.setLowPowerMode, 0x00)

submit_command(OpCode.resetBoard, 0x00)
sleep(5)
set_cryo_mode()
monitor_loop()
