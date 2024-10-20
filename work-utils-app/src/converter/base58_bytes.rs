use egui::*;
use primitive_types::U256;

#[derive(PartialEq)]
pub struct BaseBytesConverter {
    pub display_base58: String,
    pub display_hex: String,
    pub display_byte_list_u8: String,
    pub display_byte_list_i8: String,
    pub display_u256: String,

    pub display_error: Option<String>,
}

impl Default for BaseBytesConverter {
    fn default() -> Self {
        Self {
            display_base58: "".to_string(),
            display_hex: "".to_string(),
            display_byte_list_i8: "".to_string(),
            display_byte_list_u8: "".to_string(),
            display_u256: "".to_string(),

            display_error: None,
        }
    }
}

impl BaseBytesConverter {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Byte array converter to common formats");

        // Display error in red, if any
        if let Some(error) = &self.display_error {
            ui.colored_label(egui::Color32::RED, error);
        } else {
            ui.label(" ");
        }

        // Base58 input and display
        ui.horizontal(|ui| {
            ui.label("Base58: ");
            let response = ui.text_edit_singleline(&mut self.display_base58);
            if response.changed() {
                match parse_base58(&self.display_base58) {
                    Ok(s) => self.update_texts(s),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });

        // Hex input and display
        ui.horizontal(|ui| {
            ui.label("Hex: ");
            let response = ui.text_edit_singleline(&mut self.display_hex);
            if response.changed() {
                match parse_hex(&self.display_hex) {
                    Ok(s) => self.update_texts(s),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });

        // Byte list (u8) input and display
        ui.horizontal(|ui| {
            ui.label("Byte list (u8): ");
            let response = ui.text_edit_singleline(&mut self.display_byte_list_u8);
            if response.changed() {
                match parse_byte_list_u8(&self.display_byte_list_u8) {
                    Ok(byte_list) => self.update_texts(byte_list),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });

        // Byte list (i8) input and display
        ui.horizontal(|ui| {
            ui.label("Byte list (i8): ");
            let response = ui.text_edit_singleline(&mut self.display_byte_list_i8);
            if response.changed() {
                match parse_byte_list_i8(&self.display_byte_list_i8) {
                    Ok(byte_list) => self.update_texts(byte_list),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });

        // U256 input and display
        ui.horizontal(|ui| {
            ui.label("U256: ");
            let response = ui.text_edit_singleline(&mut self.display_u256);
            if response.changed() {
                match parse_u256(&self.display_u256) {
                    Ok(u256) => self.update_texts(u256),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });
    }

    /// Update texts based on a new input (Vec<u8>)
    /// input can be parsed using a helper function (parse_...  functions below)
    fn update_texts(&mut self, input: Vec<u8>) {
        self.display_error = None;

        self.display_base58 = bs58::encode(&input).into_string();
        self.display_hex = hex::encode(&input);

        let mut byte_list_u8 = String::new();
        let mut byte_list_i8 = String::new();
        for byte in input.iter() {
            byte_list_u8.push_str(&format!("{} ", byte));
            byte_list_i8.push_str(&format!("{} ", *byte as i8));
        }
        self.display_byte_list_u8 = byte_list_u8;
        self.display_byte_list_i8 = byte_list_i8;

        if input.len() > 4 * 8 {
            self.display_error = Some("Value is too large for u256".to_string());
            self.display_u256 = "".to_string();
        } else {
            self.display_u256 = primitive_types::U256::from_big_endian(&input).to_string();
        }
    }
}

/// Allow parsing of different types of byte arrays:
/// "[1, 2, 3]" -> vec![1, 2, 3]
/// "1 2 3" -> vec![1, 2, 3]
/// etc.
fn cleanse_byte_list_input(input: &str) -> String {
    input
        .replace([',', ';', ':', '\t', '\n'], " ")
        .replace(['[', ']'], "")
}

fn parse_byte_list_u8(input: &str) -> Result<Vec<u8>, String> {
    let mut result: Vec<u8> = Vec::new();
    for byte in cleanse_byte_list_input(input).split_whitespace() {
        match byte.parse::<u8>() {
            Ok(byte) => result.push(byte),
            Err(_) => return Err(format!("Failed to parse byte: {}", byte)),
        }
    }
    Ok(result)
}

fn parse_byte_list_i8(input: &str) -> Result<Vec<u8>, String> {
    let mut result: Vec<u8> = Vec::new();
    for byte in cleanse_byte_list_input(input).split_whitespace() {
        match byte.parse::<i8>() {
            Ok(byte) => result.push(byte as u8),
            Err(_) => return Err(format!("Failed to parse byte: {}", byte)),
        }
    }
    Ok(result)
}

fn parse_u256(input: &str) -> Result<Vec<u8>, String> {
    match U256::from_dec_str(input) {
        Ok(u256) => {
            let mut result = vec![0; 4 * 8];
            u256.to_big_endian(&mut result);
            Ok(result)
        }
        Err(e) => Err(format!("Failed to parse U256: {}", e)),
    }
}

fn parse_hex(input: &str) -> Result<Vec<u8>, String> {
    // If it starts with 0x, remove it
    let input = if let Some(i) = input.strip_prefix("0x") {
        i
    } else {
        input
    };

    // Verify that the input is valid hex
    match hex::decode(input) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("Failed to parse hex: {}", e)),
    }
}

fn parse_base58(input: &str) -> Result<Vec<u8>, String> {
    // Verify that the input is valid base58
    match bs58::decode(input).into_vec() {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("Failed to parse base58: {:?}", e)),
    }
}
