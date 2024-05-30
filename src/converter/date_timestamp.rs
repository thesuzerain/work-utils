use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use egui::*;
use egui_extras::DatePickerButton;

// TODO: This is the same as the base58 converter. We should be able to make this modular.
// TODO: Can be cleaned up with modularity- lots of repeated behaviour below.
#[derive(PartialEq)]
pub struct DateConverter {
    pub custom_timezone: Tz,

    pub display_timestamp: String,
    pub display_utc_calendar: NaiveDate,
    pub display_utc_iso_8601: String,
    pub display_custom_calendar: NaiveDate,
    pub display_custom_iso_8601: String,
    pub display_error: Option<String>,
}

impl Default for DateConverter {
    fn default() -> Self {
        Self {
            custom_timezone: Tz::UTC,
            display_timestamp: 0.to_string(),
            display_utc_calendar: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            display_utc_iso_8601: "1970-01-01 00:00:00".to_string(),
            display_custom_calendar: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            display_custom_iso_8601: "1970-01-01 00:00:00".to_string(),
            display_error: None,
        }
    }
}

impl DateConverter {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Date converter to common formats");

        // Display error in red, if any
        if let Some(error) = &self.display_error {
            ui.colored_label(egui::Color32::RED, error);
        } else {
            ui.label(" ");
        }

        // Timestamp display
        ui.horizontal(|ui| {
            ui.label("Unix timestamp: ");
            let response = ui.text_edit_singleline(&mut self.display_timestamp);
            if response.changed() {
                match parse_timestamp(&self.display_timestamp) {
                    Ok(s) => self.update_texts(Some(s)),
                    Err(e) => self.display_error = Some(e),
                }
            }
        });

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("UTC");
                // Calendar input and display
                ui.horizontal(|ui| {
                    ui.label("Date: ");
                    let response = DatePickerButton::new(&mut self.display_utc_calendar)
                        .id_source("utccal")
                        .ui(ui);
                    if response.changed() {
                        match parse_naive_date(&self.display_utc_calendar) {
                            Ok(s) => self.update_texts(Some(s)),
                            Err(e) => self.display_error = Some(e),
                        }
                    }
                });

                // UTC ISO-8601 input and display
                ui.horizontal(|ui| {
                    ui.label("UTC ISO-8601: ");
                    let response = ui.text_edit_singleline(&mut self.display_utc_iso_8601);
                    if response.changed() {
                        match parse_utc_iso_8601(&self.display_utc_iso_8601) {
                            Ok(iso) => self.update_texts(Some(iso)),
                            Err(e) => self.display_error = Some(e),
                        }
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let response = egui::ComboBox::from_id_source("tzpick")
                        .selected_text(format!("{:?}", self.custom_timezone))
                        .show_ui(ui, |ui| {
                            for timezone in TZ_VARIANTS {
                                ui.selectable_value(
                                    &mut self.custom_timezone,
                                    timezone,
                                    timezone.name(),
                                );
                            }
                        })
                        .response;
                    if response.changed() {
                        self.update_texts(None);
                    }

                    let response = ui.button("Guess");
                    if response.clicked() {
                        match guess_tz() {
                            Ok(tz) => {
                                self.custom_timezone = tz;
                                self.update_texts(None);
                            }
                            Err(e) => self.display_error = Some(e),
                        }
                    }
                });

                // Calendar input and display
                ui.horizontal(|ui| {
                    ui.label("Date: ");
                    let response = DatePickerButton::new(&mut self.display_custom_calendar)
                        .id_source("tzcal")
                        .ui(ui);
                    if response.changed() {
                        match parse_naive_date(&self.display_custom_calendar) {
                            Ok(s) => self.update_texts(Some(s)),
                            Err(e) => self.display_error = Some(e),
                        }
                    }
                });

                // Custom ISO-8601
                ui.horizontal(|ui| {
                    ui.label("ISO-8601: ");
                    let response = ui.text_edit_singleline(&mut self.display_custom_iso_8601);
                    if response.changed() {
                        match parse_custom_iso_8601(
                            &self.custom_timezone,
                            &self.display_custom_iso_8601,
                        ) {
                            Ok(iso) => self.update_texts(Some(iso)),
                            Err(e) => self.display_error = Some(e),
                        }
                    }
                });
            });
        });
    }

    /// Update texts based on a new input (NaiveDateTime)
    fn update_texts(&mut self, input: Option<DateTime<Utc>>) {
        let input = match input {
            Some(i) => i,
            None => match parse_timestamp(&self.display_timestamp) {
                Ok(t) => t,
                Err(e) => {
                    self.display_error = Some(e);
                    return;
                }
            },
        };

        self.display_error = None;

        self.display_timestamp = input.timestamp().to_string();
        self.display_utc_calendar = input.date_naive();
        self.display_utc_iso_8601 = input.to_string();

        self.display_custom_calendar = input.with_timezone(&self.custom_timezone).date_naive();
        self.display_custom_iso_8601 = input.with_timezone(&self.custom_timezone).to_string();
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
fn parse_utc_iso_8601(input: &str) -> Result<DateTime<Utc>, String> {
    let with_time = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S");
    let without_time =
        NaiveDate::parse_from_str(input, "%Y-%m-%d").map(|date| date.and_hms_opt(0, 0, 0).unwrap());
    match with_time.or(without_time) {
        Ok(date) => Ok(date.and_utc()),
        Err(e) => Err(format!("Failed to parse ISO-8601: {}", e)),
    }
}

fn parse_custom_iso_8601(tz: &Tz, input: &str) -> Result<DateTime<Utc>, String> {
    let with_time = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S");
    let without_time =
        NaiveDate::parse_from_str(input, "%Y-%m-%d").map(|date| date.and_hms_opt(0, 0, 0).unwrap());

    match with_time
        .or(without_time)
        .map(|t| tz.from_local_datetime(&t).earliest().unwrap().to_utc())
    {
        Ok(date) => Ok(date),
        Err(e) => Err(format!("Failed to parse ISO-8601: {}", e)),
    }
}

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

#[test]
fn list_offsetstest() {
    println!()
}
