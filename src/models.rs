//! Nightscout v3 document types.
//!
//! Numeric fields use `Decimal` for API `Number` values.

use std::collections::BTreeMap;
use std::fmt;

use rust_decimal::Decimal;
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NightscoutSecrets {
    pub website: String,
    pub permission_role: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NightscoutBearer {
    pub token: String,
}

/// Fields shared by every Nightscout v3 document.
///
/// `identifier` is the client-controlled dedup key.
#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentBase {
    pub identifier: Option<String>,
    pub date: i64,
    pub utc_offset: Option<i32>,
    pub app: String,
    pub device: Option<String>,

    // Server-managed fields. Do not set.
    #[serde(rename = "_id")]
    pub id_internal: Option<String>,
    pub srv_created: Option<i64>,
    pub subject: Option<String>,
    pub srv_modified: Option<i64>,
    pub modified_by: Option<String>,
    pub is_valid: Option<bool>,
    pub is_read_only: Option<bool>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    #[serde(flatten)]
    pub base: DocumentBase,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub sgv: Option<Decimal>,
    pub direction: Option<String>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub noise: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub filtered: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub unfiltered: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub rssi: Option<Decimal>,
    pub units: Option<String>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Food {
    #[serde(flatten)]
    pub base: DocumentBase,
    pub food: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub name: Option<String>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub portion: Option<Decimal>,
    pub unit: Option<String>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub carbs: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub fat: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub protein: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub energy: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub gi: Option<Decimal>,
    pub hide_after_use: Option<bool>,
    pub hidden: Option<bool>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub position: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub portions: Option<Decimal>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Treatment {
    #[serde(flatten)]
    pub base: DocumentBase,
    pub event_type: Option<String>,
    /// Nightscout may send this as either a JSON string or number.
    #[serde(
        default,
        deserialize_with = "deserialize_decimal_maybe_string",
        serialize_with = "serialize_decimal_option"
    )]
    pub glucose: Option<Decimal>,
    pub glucose_type: Option<String>,
    pub units: Option<String>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub carbs: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub protein: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub fat: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub insulin: Option<Decimal>,
    /// Duration in minutes.
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub duration: Option<Decimal>,
    /// Pre-bolus offset in seconds.
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub pre_bolus: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub split_now: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub split_ext: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub percent: Option<Decimal>,
    /// Basal rate in units per hour.
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub absolute: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub target_top: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub target_bottom: Option<Decimal>,
    pub profile: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub entered_by: Option<String>,
}

impl Treatment {
    /// Whether this treatment is a bolus for fuzzy dedup.
    pub fn is_bolus(&self) -> bool {
        matches!(
            self.event_type.as_deref(),
            Some("Correction Bolus" | "Meal Bolus" | "Snack Bolus" | "Combo Bolus")
        )
    }

    /// Event type used for fuzzy dedup.
    pub fn dedup_event_type(&self) -> Option<&'static str> {
        match self.event_type.as_deref()? {
            "Site Change" => Some("Site Change"),
            "Suspend Pump" => Some("Suspend Pump"),
            "Resume Pump" => Some("Resume Pump"),
            "Announcement" => Some("Announcement"),
            _ => None,
        }
    }
}

/// Handle JSON string, number, or null.
fn deserialize_decimal_maybe_string<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl<'de> Visitor<'de> for V {
        type Value = Option<Decimal>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a decimal number, numeric string, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_any(V)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                return Ok(None);
            }
            v.parse::<Decimal>()
                .map(Some)
                .map_err(|e| E::custom(format!("decimal string {v:?}: {e}")))
        }
        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(Decimal::from(v)))
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(Decimal::from(v)))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Decimal::from_f64_retain(v)
                .map(Some)
                .ok_or_else(|| E::custom(format!("f64 {v} is not representable as Decimal")))
        }
    }

    deserializer.deserialize_any(V)
}

fn serialize_decimal_option<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    rust_decimal::serde::arbitrary_precision_option::serialize(value, serializer)
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Devicestatus {
    #[serde(flatten)]
    pub base: DocumentBase,
    #[serde(rename = "loop")]
    pub loop_: Option<LoopStatus>,
    pub pump: Option<PumpStatus>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoopStatus {
    pub name: Option<String>,
    pub version: Option<String>,
    pub iob: Option<LoopIob>,
    pub cob: Option<LoopCob>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoopIob {
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub iob: Option<Decimal>,
    pub timestamp: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoopCob {
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub cob: Option<Decimal>,
    pub timestamp: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PumpStatus {
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub reservoir: Option<Decimal>,
    pub battery: Option<PumpBattery>,
    pub clock: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PumpBattery {
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub percent: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub voltage: Option<Decimal>,
    pub status: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(flatten)]
    pub base: DocumentBase,
    pub default_profile: String,
    pub store: BTreeMap<String, ProfileStore>,
    pub start_date: Option<String>,
    pub mills: Option<i64>,
    pub units: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileStore {
    #[serde(default, with = "rust_decimal::serde::arbitrary_precision_option")]
    pub dia: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::arbitrary_precision_option")]
    pub carbs_hr: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::arbitrary_precision_option")]
    pub delay: Option<Decimal>,
    pub timezone: Option<String>,
    pub basal: Vec<TimeValue>,
    pub carbratio: Vec<TimeValue>,
    pub sens: Vec<TimeValue>,
    pub target_low: Vec<TimeValue>,
    pub target_high: Vec<TimeValue>,
    pub units: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeValue {
    pub time: String,
    #[serde(rename = "timeAsSeconds")]
    pub time_as_seconds: i64,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub value: Decimal,
}
