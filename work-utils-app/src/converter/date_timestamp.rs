use std::sync::Arc;

use crate::{REQWEST_CLIENT, WORK_UTILS_API_URL};
use chrono::{
    DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, Offset, SubsecRound, TimeZone, Utc,
};
use chrono_tz::{OffsetComponents, OffsetName, Tz, TZ_VARIANTS};
use egui::*;
use egui_extras::DatePickerButton;
use tokio::sync::Mutex;

// TODO: This is the same as the base58 converter. We should be able to make this modular.
// TODO: Can be cleaned up with modularity- lots of repeated behaviour below.
pub struct DateConverter {
    pub data: Arc<Mutex<DateConverterData>>,
}

pub struct DateConverterData {
    pub custom_timezone: Tz,

    pub display_timestamp: String,
    pub display_utc_calendar: NaiveDate,
    pub display_utc_iso_8601: String,
    pub display_custom_calendar: NaiveDate,
    pub display_custom_iso_8601: String,

    pub display_solana_block: String,
    pub loading_solana_block: bool,

    pub display_error: Option<String>,
}

impl Default for DateConverter {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(DateConverterData {
                custom_timezone: Tz::UTC,
                display_timestamp: 0.to_string(),
                display_utc_calendar: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                display_utc_iso_8601: "1970-01-01 00:00:00".to_string(),
                display_custom_calendar: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                display_custom_iso_8601: "1970-01-01 00:00:00".to_string(),
                display_solana_block: "0".to_string(),
                loading_solana_block: false,
                display_error: None,
            })),
        }
    }
}

impl DateConverter {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Date converter to common formats");
        let data = self.data.clone();
        let mut data = data.blocking_lock();

        // Display error in red, if any
        if let Some(error) = &data.display_error {
            ui.colored_label(egui::Color32::RED, error);
        } else {
            ui.label(" ");
        }

        // Timestamp display
        ui.horizontal(|ui| {
            ui.label("Unix timestamp: ");
            let response = ui.text_edit_singleline(&mut data.display_timestamp);
            if response.changed() {
                match parse_timestamp(&data.display_timestamp) {
                    Ok(s) => Self::update_texts(Some(s), &mut data),
                    Err(e) => data.display_error = Some(e),
                }
            }
        });

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("UTC");
                    let response = ui.button("Now").on_hover_text("Set to current UTC time");
                    if response.clicked() {
                        Self::update_texts(Some(Utc::now()), &mut data);
                    };
                });
                // Calendar input and display
                ui.horizontal(|ui| {
                    ui.label("Date: ");
                    let response = DatePickerButton::new(&mut data.display_utc_calendar)
                        .id_source("utccal")
                        .ui(ui);
                    if response.changed() {
                        match parse_naive_date(&data.display_utc_calendar) {
                            Ok(s) => Self::update_texts(Some(s), &mut data),
                            Err(e) => data.display_error = Some(e),
                        }
                    }
                });

                // UTC ISO-8601 input and display
                ui.horizontal(|ui| {
                    ui.label("UTC ISO-8601: ");
                    let response = ui.text_edit_singleline(&mut data.display_utc_iso_8601);
                    if response.changed() {
                        match parse_iso_8601(&data.display_utc_iso_8601) {
                            Ok(iso) => Self::update_texts(Some(iso.to_utc()), &mut data),
                            Err(e) => data.display_error = Some(e),
                        }
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Time zone selection
                    let response = egui::ComboBox::from_id_source("tzpick")
                        .selected_text(format!("{:?}", data.custom_timezone))
                        .show_ui(ui, |ui| {
                            let mut any_clicked = false;
                            for timezone in TZ_VARIANTS {
                                any_clicked |= ui
                                    .selectable_value(
                                        &mut data.custom_timezone,
                                        timezone,
                                        timezone.name(),
                                    )
                                    .clicked();
                            }
                            any_clicked
                        })
                        .inner;
                    if response == Some(true) {
                        Self::update_texts(None, &mut data);
                    }

                    let response = ui.button("Guess");
                    if response.clicked() {
                        match guess_tz() {
                            Ok(tz) => {
                                data.custom_timezone = tz;
                                Self::update_texts(None, &mut data);
                            }
                            Err(e) => data.display_error = Some(e),
                        }
                    }
                });

                // Calendar input and display
                ui.horizontal(|ui| {
                    ui.label("Date: ");
                    let response = DatePickerButton::new(&mut data.display_custom_calendar)
                        .id_source("tzcal")
                        .ui(ui);
                    if response.changed() {
                        match parse_naive_date(&data.display_custom_calendar) {
                            Ok(s) => Self::update_texts(Some(s), &mut data),
                            Err(e) => data.display_error = Some(e),
                        }
                    }
                });

                // Custom ISO-8601
                ui.horizontal(|ui| {
                    ui.label("ISO-8601: ");
                    let response = ui.text_edit_singleline(&mut data.display_custom_iso_8601);
                    if response.changed() {
                        match parse_iso_8601(&data.display_custom_iso_8601) {
                            Ok(iso) => {
                                data.custom_timezone = iso.timezone();
                                Self::update_texts(Some(iso.to_utc()), &mut data);
                            }
                            Err(e) => data.display_error = Some(e),
                        }
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Solana block: ");
                    let response = ui.text_edit_singleline(&mut data.display_solana_block);
                    if response.changed() {
                        match data.display_solana_block.parse::<u64>() {
                            Ok(o) => {
                                // Start a new solana recalculator
                                data.loading_solana_block = true;
                                let current_sol_block = data.display_solana_block.clone();
                                let data_clone = self.data.clone();
                                let fut = async move {
                                    match get_solana_block_timestamp(
                                        o,
                                    )
                                    .await
                                    {
                                        Ok(o) => {
                                            let new_sol_block = {
                                                let data = data_clone.lock().await;
                                                data.display_solana_block.clone()
                                            };
                                            if new_sol_block == current_sol_block {
                                                let mut data_clone_lock = data_clone.lock().await;
                                                data_clone_lock.loading_solana_block = false;
                                                match parse_timestamp(&o.to_string()) {
                                                    Ok(s) => Self::update_texts(
                                                        Some(s),
                                                        &mut data_clone_lock,
                                                    ),
                                                    Err(e) => {
                                                        let mut data = data_clone.lock().await;
                                                        data.display_error = Some(e);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let mut data = data_clone.lock().await;
                                            println!("> Received error with {} == {}", current_sol_block, data.display_solana_block);
                                            if current_sol_block == data.display_solana_block {
                                                data.loading_solana_block = false;
                                                data.display_error =
                                                    Some(format!("Failed to get block: {}", e));
                                            }
                                        }
                                    }
                                };
                                #[cfg(feature = "web_app")]
                                {
                                    wasm_bindgen_futures::spawn_local(fut);
                                }
                                #[cfg(not(feature = "web_app"))]
                                {
                                    tokio::spawn(fut);
                                }
                            }
                            Err(e) => {
                                data.display_error = Some(format!("Failed to parse block: {}", e))
                            }
                        }
                    }

                    if data.loading_solana_block {
                        ui.spinner();
                    }
                });

                ui.label("Solana block check uses an API as a proxy to access the RPC.");
                ui.label("This is because the RPC is blocked when making requests from the browser (or WASM).");
                ui.label("Because it is on the render.com free tier, it spins down often. The first request in a while may take a minute, but subsequent requests should be faster.");
            })
        });
    }

    /// Update texts based on a new input (NaiveDateTime)
    /// This update happens asynchronously
    ///
    /// 'input' is the new timestamp to update to. If None, it will parse the current timestamp
    fn update_texts(input: Option<DateTime<Utc>>, data: &mut DateConverterData) {
        let input = match input {
            Some(i) => i,
            None => match parse_timestamp(&data.display_timestamp) {
                Ok(t) => t,
                Err(e) => {
                    data.display_error = Some(e);
                    return;
                }
            },
        };
        let input = input.round_subsecs(0);

        data.display_error = None;

        data.display_timestamp = input.timestamp().to_string();
        data.display_utc_calendar = input.date_naive();
        data.display_utc_iso_8601 = input.to_string();

        data.display_custom_calendar = input.with_timezone(&data.custom_timezone).date_naive();
        data.display_custom_iso_8601 = input.with_timezone(&data.custom_timezone).to_string();
    }
}

fn parse_timestamp(input: &String) -> Result<DateTime<Utc>, String> {
    if let Ok(timestamp) = input.parse::<i64>() {
        if let Some(date) = DateTime::from_timestamp(timestamp, 0) {
            return Ok(date);
        }
    }
    Err(format!("Failed to parse timestamp: {}", input))
}

fn parse_naive_date(input: &NaiveDate) -> Result<DateTime<Utc>, String> {
    // sets to 00:00:00
    match input.and_hms_opt(0, 0, 0) {
        Some(date) => Ok(date.and_utc()),
        None => Err(format!("Failed to parse date: {}", input)),
    }
}

// TODO: these coudl be the same function with a generic
fn parse_iso_8601(input: &str) -> Result<DateTime<Tz>, String> {
    // For an RFC3339 date, we guess the timezone from the offset
    if let Ok(date_time_rfc3339) = DateTime::parse_from_rfc3339(input) {
        let tz = guess_tz_from_fixed_offset(date_time_rfc3339.offset().fix()).unwrap_or(Tz::UTC);
        return Ok(tz.from_utc_datetime(&date_time_rfc3339.naive_utc()));
    }

    // If it fails UTC, we attempt to parse with a timezone
    // %Y-%m-%d %H:%M:%S %:z (seconds and timezone optional)
    match NaiveDateTime::parse_and_remainder(input, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_and_remainder(input, "%Y-%m-%d"))
    {
        Ok((date_time, tz)) => {
            if tz.is_empty() {
                Ok(Tz::UTC.from_utc_datetime(&date_time))
            } else {
                let tz: Tz = parse_timezone_abbreviation(tz.trim())
                    .map_err(|e| format!("Failed to parse timezone: {}", e))?;
                tz.from_local_datetime(&date_time)
                    .earliest()
                    .ok_or_else(|| "Invalid time".to_string())
            }
        }
        Err(e) => Err(format!("Failed to parse ISO-8601: {}", e)),
    }
}

// TODO: Can use below to guess timezone from offset
fn guess_tz() -> Result<Tz, String> {
    let now_local_naive = Local::now().naive_local();
    let now_utc = Utc::now();
    TZ_VARIANTS
        .into_iter()
        .find(|tz| {
            let c = tz.from_local_datetime(&now_local_naive).earliest();
            if let Some(c) = c {
                c.timestamp() == now_utc.timestamp()
            } else {
                false
            }
        })
        .ok_or_else(|| "Could not find local timezone.".to_string())
}

fn guess_tz_from_fixed_offset(offset: FixedOffset) -> Result<Tz, String> {
    TZ_VARIANTS
        .into_iter()
        .find(|tz| {
            tz.offset_from_utc_datetime(&Utc::now().naive_utc())
                .base_utc_offset()
                .num_seconds() as i32
                == offset.utc_minus_local()
        })
        .ok_or_else(|| "Could not find timezone from offset.".to_string())
}

// TODO: Find a crate for this- searching the array is not efficient
// TODO: Should include "+11", "-11" etc.
fn parse_timezone_abbreviation(input: &str) -> Result<Tz, String> {
    TZ_VARIANTS
        .into_iter()
        .find(|tz| {
            tz.offset_from_utc_datetime(&Utc::now().naive_utc())
                .abbreviation()
                == input
        })
        .ok_or_else(|| "Could not find timezone from offset.".to_string())
}

async fn get_solana_block_timestamp(block: u64) -> Result<i64, String> {
    let uri = format!("{}solana_blocktime/{}", WORK_UTILS_API_URL, block);
    println!("Requesting: {}", uri);
    let response = REQWEST_CLIENT
        .get(&uri)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {:?}", e))?;

    let response = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    println!("Response: {:?}", response);
    let value = response
        .get("timestamp")
        .ok_or(format!("Response missing result: {:?}", response))?
        .as_i64()
        .ok_or(format!("Response result not an i64: {:?}", response))?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use chrono::TimeZone;
    use chrono_tz::Tz;

    #[test]
    fn parse_timezone() {
        let tz = super::parse_iso_8601("2024-06-01 16:34:19 UTC").unwrap();
        assert_eq!(
            tz,
            Tz::UTC.from_utc_datetime(
                &NaiveDate::from_ymd_opt(2024, 6, 1)
                    .unwrap()
                    .and_hms_opt(16, 34, 19)
                    .unwrap()
            )
        );
    }
}
