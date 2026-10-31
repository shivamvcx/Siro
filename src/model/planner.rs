use super::element::TableElement;
use egui::Pos2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub is_hackclub_milestone: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BomItem {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub spec_value: String,
    pub quantity: u32,
    pub unit_cost: f64,
    pub acquired: bool,
    pub supplier_url: String,
    pub footprint_or_notes: String,
}

impl BomItem {
    pub fn new(name: impl Into<String>, spec_value: impl Into<String>, quantity: u32, unit_cost: f64) -> Self {
        Self::new_with_id(Uuid::new_v4(), name, spec_value, quantity, unit_cost)
    }

    pub fn new_with_id(
        id: Uuid,
        name: impl Into<String>,
        spec_value: impl Into<String>,
        quantity: u32,
        unit_cost: f64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            part_number: String::new(),
            spec_value: spec_value.into(),
            quantity,
            unit_cost,
            acquired: false,
            supplier_url: String::new(),
            footprint_or_notes: String::new(),
        }
    }

    pub fn line_total(&self) -> f64 {
        self.quantity as f64 * self.unit_cost
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HackClubInfo {
    pub project_name: String,
    pub ship_status: String,
    pub update_url: String,
    pub slack_channel: String,
    pub arcade_hours: f32,
    pub grant_budget: f64,
}

impl Default for HackClubInfo {
    fn default() -> Self {
        Self {
            project_name: "Hardware Project".to_string(),
            ship_status: "Prototyping".to_string(),
            update_url: "https://scrapbook.hackclub.com".to_string(),
            slack_channel: "#electronics".to_string(),
            arcade_hours: 0.0,
            grant_budget: 100.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlannerData {
    pub goals: Vec<GoalItem>,
    pub bom: Vec<BomItem>,
    pub hackclub: HackClubInfo,
    pub scratchpad_notes: String,
}

impl Default for PlannerData {
    fn default() -> Self {
        Self {
            goals: vec![
                GoalItem {
                    id: Uuid::from_u128(1),
                    title: "Breadboard schematic & initial wire testing".to_string(),
                    completed: false,
                    is_hackclub_milestone: false,
                },
                GoalItem {
                    id: Uuid::from_u128(2),
                    title: "Firmware MVP (GPIO / I2C sensors working)".to_string(),
                    completed: false,
                    is_hackclub_milestone: true,
                },
                GoalItem {
                    id: Uuid::from_u128(3),
                    title: "Bill of Materials finalized & components sourced".to_string(),
                    completed: false,
                    is_hackclub_milestone: false,
                },
                GoalItem {
                    id: Uuid::from_u128(4),
                    title: "Ship update posted to Hack Club Scrapbook".to_string(),
                    completed: false,
                    is_hackclub_milestone: true,
                },
            ],
            bom: vec![
                BomItem::new_with_id(Uuid::from_u128(101), "RP2040 Pico", "Raspberry Pi Pico H", 1, 5.00),
                BomItem::new_with_id(Uuid::from_u128(102), "OLED Display", "0.96 inch I2C 128x64 SSD1306", 1, 3.50),
                BomItem::new_with_id(Uuid::from_u128(103), "Resistors Kit", "10kΩ 0.25W 1% (x10)", 1, 1.20),
                BomItem::new_with_id(Uuid::from_u128(104), "Breadboard & Jumpers", "830-point + 65 wires", 1, 4.50),
            ],
            hackclub: HackClubInfo::default(),
            scratchpad_notes: "PIN CONFIGURATION & QUICK NOTES:\n- I2C0: GP4 (SDA), GP5 (SCL)\n- UART0: GP0 (TX), GP1 (RX)\n- LED: GP25 (Onboard)\n- Target power draw: <120mA @ 3.3V".to_string(),
        }
    }
}

impl PlannerData {
    pub fn total_cost(&self) -> f64 {
        self.bom.iter().map(|item| item.line_total()).sum()
    }

    pub fn acquired_cost(&self) -> f64 {
        self.bom.iter().filter(|i| i.acquired).map(|item| item.line_total()).sum()
    }

    pub fn remaining_cost(&self) -> f64 {
        self.total_cost() - self.acquired_cost()
    }

    /// Generates or converts the BOM into an interactive canvas TableElement
    pub fn to_canvas_table(&self, pos: Pos2) -> TableElement {
        let rows = self.bom.len() + 1; // +1 header
        let cols = 6;
        let mut cells = Vec::with_capacity(rows);

        // Header
        cells.push(vec![
            "Item / Part".to_string(),
            "Specification".to_string(),
            "Qty".to_string(),
            "Unit ($)".to_string(),
            "Total ($)".to_string(),
            "Status".to_string(),
        ]);

        for item in &self.bom {
            cells.push(vec![
                item.name.clone(),
                item.spec_value.clone(),
                item.quantity.to_string(),
                format!("{:.2}", item.unit_cost),
                format!("{:.2}", item.line_total()),
                if item.acquired { "Acquired ✓".to_string() } else { "Needed".to_string() },
            ]);
        }

        let mut table = TableElement::new(Uuid::new_v4(), pos, rows, cols);
        table.cells = cells;
        table.col_widths = vec![140.0, 160.0, 50.0, 75.0, 80.0, 95.0];
        table.is_bom_linked = true;
        table
    }
}
