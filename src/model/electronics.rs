use egui::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ComponentKind {
    // Maker Boards & Modules
    RaspberryPiPico,
    Esp32Wroom,
    ArduinoNano,
    GenericIC { pin_count: u32, labels: Vec<String> },
    BreadboardSection,

    // Schematic Symbols
    Resistor,
    Capacitor,
    Inductor,
    Diode,
    Led,
    Battery,
    Ground,
    Switch,
    NpnTransistor,
    PnpTransistor,
    Mosfet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminalPin {
    pub id: usize,
    pub name: String,
    pub local_pos: Pos2,
}

impl ComponentKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentKind::RaspberryPiPico => "Raspberry Pi Pico",
            ComponentKind::Esp32Wroom => "ESP32-WROOM-32",
            ComponentKind::ArduinoNano => "Arduino Nano",
            ComponentKind::GenericIC { .. } => "IC DIP Package",
            ComponentKind::BreadboardSection => "Breadboard Section",
            ComponentKind::Resistor => "Resistor (R)",
            ComponentKind::Capacitor => "Capacitor (C)",
            ComponentKind::Inductor => "Inductor (L)",
            ComponentKind::Diode => "Diode (D)",
            ComponentKind::Led => "LED",
            ComponentKind::Battery => "DC Battery (VCC)",
            ComponentKind::Ground => "Ground (GND)",
            ComponentKind::Switch => "Switch (SW)",
            ComponentKind::NpnTransistor => "NPN Transistor (BJT)",
            ComponentKind::PnpTransistor => "PNP Transistor (BJT)",
            ComponentKind::Mosfet => "N-Channel MOSFET",
        }
    }

    pub fn default_designator(&self) -> &'static str {
        match self {
            ComponentKind::RaspberryPiPico => "MCU1",
            ComponentKind::Esp32Wroom => "ESP1",
            ComponentKind::ArduinoNano => "ARD1",
            ComponentKind::GenericIC { .. } => "U1",
            ComponentKind::BreadboardSection => "BB1",
            ComponentKind::Resistor => "R1",
            ComponentKind::Capacitor => "C1",
            ComponentKind::Inductor => "L1",
            ComponentKind::Diode => "D1",
            ComponentKind::Led => "LED1",
            ComponentKind::Battery => "BT1",
            ComponentKind::Ground => "GND",
            ComponentKind::Switch => "SW1",
            ComponentKind::NpnTransistor => "Q1",
            ComponentKind::PnpTransistor => "Q2",
            ComponentKind::Mosfet => "M1",
        }
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            ComponentKind::Resistor => "10kΩ",
            ComponentKind::Capacitor => "100nF",
            ComponentKind::Inductor => "10µH",
            ComponentKind::Diode => "1N4148",
            ComponentKind::Led => "Green 2.1V",
            ComponentKind::Battery => "3.7V LiPo",
            ComponentKind::Ground => "0V",
            ComponentKind::Switch => "SPST",
            ComponentKind::NpnTransistor => "2N2222",
            ComponentKind::PnpTransistor => "2N3906",
            ComponentKind::Mosfet => "2N7000",
            ComponentKind::RaspberryPiPico => "RP2040",
            ComponentKind::Esp32Wroom => "ESP32-D0WD",
            ComponentKind::ArduinoNano => "ATmega328P",
            ComponentKind::GenericIC { .. } => "DIP-IC",
            ComponentKind::BreadboardSection => "Half+Power",
        }
    }

    pub fn default_size(&self) -> (f32, f32) {
        match self {
            ComponentKind::RaspberryPiPico => (140.0, 320.0),
            ComponentKind::Esp32Wroom => (150.0, 300.0),
            ComponentKind::ArduinoNano => (130.0, 280.0),
            ComponentKind::GenericIC { pin_count, .. } => {
                let rows = ((*pin_count as f32) / 2.0).max(4.0);
                (100.0, rows * 26.0 + 30.0)
            }
            ComponentKind::BreadboardSection => (220.0, 360.0),
            ComponentKind::Resistor => (100.0, 30.0),
            ComponentKind::Capacitor => (70.0, 40.0),
            ComponentKind::Inductor => (90.0, 36.0),
            ComponentKind::Diode => (80.0, 36.0),
            ComponentKind::Led => (80.0, 40.0),
            ComponentKind::Battery => (60.0, 60.0),
            ComponentKind::Ground => (40.0, 50.0),
            ComponentKind::Switch => (80.0, 40.0),
            ComponentKind::NpnTransistor | ComponentKind::PnpTransistor => (80.0, 80.0),
            ComponentKind::Mosfet => (80.0, 80.0),
        }
    }

    pub fn generate_terminals(&self, width: f32, height: f32) -> Vec<TerminalPin> {
        let mut terminals = Vec::new();
        match self {
            ComponentKind::RaspberryPiPico => {
                let left_pins = [
                    "GP0", "GP1", "GND", "GP2", "GP3", "GP4", "GP5", "GND", "GP6", "GP7",
                    "GP8", "GP9", "GND", "GP10", "GP11", "GP12", "GP13", "GND", "GP14", "GP15",
                ];
                let right_pins = [
                    "VBUS", "VSYS", "GND", "3V3_EN", "3V3", "ADC_REF", "GP28", "GND", "GP27", "GP26",
                    "RUN", "GP22", "GND", "GP21", "GP20", "GP19", "GP18", "GND", "GP17", "GP16",
                ];
                let step = (height - 30.0) / (left_pins.len() as f32 - 1.0);
                for (i, name) in left_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: i,
                        name: name.to_string(),
                        local_pos: Pos2::new(0.0, 15.0 + i as f32 * step),
                    });
                }
                for (i, name) in right_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: 20 + i,
                        name: name.to_string(),
                        local_pos: Pos2::new(width, 15.0 + i as f32 * step),
                    });
                }
            }
            ComponentKind::Esp32Wroom => {
                let left_pins = [
                    "3V3", "EN", "VP", "VN", "G34", "G35", "G32", "G33", "G25", "G26",
                    "G27", "G14", "G12", "GND", "G13", "D2", "D3", "CMD", "GND",
                ];
                let right_pins = [
                    "G23", "G22", "TX0", "RX0", "G21", "GND", "G19", "G18", "G5", "G17",
                    "G16", "G4", "G0", "G2", "G15", "SD1", "SD0", "CLK", "VIN",
                ];
                let step = (height - 30.0) / (left_pins.len() as f32 - 1.0);
                for (i, name) in left_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: i,
                        name: name.to_string(),
                        local_pos: Pos2::new(0.0, 15.0 + i as f32 * step),
                    });
                }
                for (i, name) in right_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: 19 + i,
                        name: name.to_string(),
                        local_pos: Pos2::new(width, 15.0 + i as f32 * step),
                    });
                }
            }
            ComponentKind::ArduinoNano => {
                let left_pins = [
                    "D13", "3V3", "REF", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "5V", "RST", "GND", "VIN",
                ];
                let right_pins = [
                    "D12", "D11", "D10", "D9", "D8", "D7", "D6", "D5", "D4", "D3", "D2", "GND", "RST", "RX0", "TX1",
                ];
                let step = (height - 30.0) / (left_pins.len() as f32 - 1.0);
                for (i, name) in left_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: i,
                        name: name.to_string(),
                        local_pos: Pos2::new(0.0, 15.0 + i as f32 * step),
                    });
                }
                for (i, name) in right_pins.iter().enumerate() {
                    terminals.push(TerminalPin {
                        id: 15 + i,
                        name: name.to_string(),
                        local_pos: Pos2::new(width, 15.0 + i as f32 * step),
                    });
                }
            }
            ComponentKind::GenericIC { pin_count, labels } => {
                let count = (*pin_count).max(2);
                let per_side = (count / 2) as usize;
                let step = if per_side > 1 {
                    (height - 30.0) / (per_side as f32 - 1.0)
                } else {
                    0.0
                };
                for i in 0..per_side {
                    let label = labels.get(i).cloned().unwrap_or_else(|| format!("P{}", i + 1));
                    terminals.push(TerminalPin {
                        id: i,
                        name: label,
                        local_pos: Pos2::new(0.0, 15.0 + i as f32 * step),
                    });
                }
                for i in 0..per_side {
                    let pin_idx = per_side + i;
                    let label = labels.get(pin_idx).cloned().unwrap_or_else(|| format!("P{}", pin_idx + 1));
                    terminals.push(TerminalPin {
                        id: pin_idx,
                        name: label,
                        local_pos: Pos2::new(width, 15.0 + (per_side - 1 - i) as f32 * step),
                    });
                }
            }
            ComponentKind::BreadboardSection => {
                // Power rail left (+, -) and right (+, -)
                terminals.push(TerminalPin { id: 0, name: "V+".to_string(), local_pos: Pos2::new(15.0, 20.0) });
                terminals.push(TerminalPin { id: 1, name: "GND".to_string(), local_pos: Pos2::new(30.0, 20.0) });
                terminals.push(TerminalPin { id: 2, name: "V+".to_string(), local_pos: Pos2::new(width - 30.0, 20.0) });
                terminals.push(TerminalPin { id: 3, name: "GND".to_string(), local_pos: Pos2::new(width - 15.0, 20.0) });
                // Additional representative terminal tie points
                for row in 0..8 {
                    let y = 50.0 + row as f32 * 36.0;
                    terminals.push(TerminalPin { id: 10 + row * 2, name: format!("L{}", row + 1), local_pos: Pos2::new(55.0, y) });
                    terminals.push(TerminalPin { id: 10 + row * 2 + 1, name: format!("R{}", row + 1), local_pos: Pos2::new(width - 55.0, y) });
                }
            }
            ComponentKind::Resistor | ComponentKind::Capacitor | ComponentKind::Inductor | ComponentKind::Diode | ComponentKind::Led | ComponentKind::Switch => {
                terminals.push(TerminalPin {
                    id: 0,
                    name: "1".to_string(),
                    local_pos: Pos2::new(0.0, height / 2.0),
                });
                terminals.push(TerminalPin {
                    id: 1,
                    name: "2".to_string(),
                    local_pos: Pos2::new(width, height / 2.0),
                });
            }
            ComponentKind::Battery => {
                terminals.push(TerminalPin {
                    id: 0,
                    name: "+".to_string(),
                    local_pos: Pos2::new(width / 2.0, 0.0),
                });
                terminals.push(TerminalPin {
                    id: 1,
                    name: "-".to_string(),
                    local_pos: Pos2::new(width / 2.0, height),
                });
            }
            ComponentKind::Ground => {
                terminals.push(TerminalPin {
                    id: 0,
                    name: "GND".to_string(),
                    local_pos: Pos2::new(width / 2.0, 0.0),
                });
            }
            ComponentKind::NpnTransistor | ComponentKind::PnpTransistor => {
                terminals.push(TerminalPin {
                    id: 0,
                    name: "B".to_string(),
                    local_pos: Pos2::new(0.0, height / 2.0),
                });
                terminals.push(TerminalPin {
                    id: 1,
                    name: "C".to_string(),
                    local_pos: Pos2::new(width, 15.0),
                });
                terminals.push(TerminalPin {
                    id: 2,
                    name: "E".to_string(),
                    local_pos: Pos2::new(width, height - 15.0),
                });
            }
            ComponentKind::Mosfet => {
                terminals.push(TerminalPin {
                    id: 0,
                    name: "G".to_string(),
                    local_pos: Pos2::new(0.0, height / 2.0),
                });
                terminals.push(TerminalPin {
                    id: 1,
                    name: "D".to_string(),
                    local_pos: Pos2::new(width, 15.0),
                });
                terminals.push(TerminalPin {
                    id: 2,
                    name: "S".to_string(),
                    local_pos: Pos2::new(width, height - 15.0),
                });
            }
        }
        terminals
    }
}
