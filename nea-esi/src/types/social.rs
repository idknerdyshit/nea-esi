use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A character contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContract {
    pub contract_id: i64,
    pub issuer_id: i64,
    pub issuer_corporation_id: i64,
    #[serde(default)]
    pub assignee_id: Option<i64>,
    #[serde(default)]
    pub acceptor_id: Option<i64>,
    #[serde(rename = "type")]
    pub contract_type: String,
    pub status: String,
    pub availability: String,
    pub date_issued: DateTime<Utc>,
    pub date_expired: DateTime<Utc>,
    pub for_corporation: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub date_accepted: Option<DateTime<Utc>>,
    #[serde(default)]
    pub date_completed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub reward: Option<f64>,
    #[serde(default)]
    pub collateral: Option<f64>,
    #[serde(default)]
    pub buyout: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
    #[serde(default)]
    pub days_to_complete: Option<i32>,
    #[serde(default)]
    pub start_location_id: Option<i64>,
    #[serde(default)]
    pub end_location_id: Option<i64>,
}

/// An item in a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContractItem {
    pub record_id: i64,
    pub type_id: i32,
    pub quantity: i32,
    pub is_included: bool,
    #[serde(default)]
    pub is_singleton: Option<bool>,
    #[serde(default)]
    pub raw_quantity: Option<i32>,
}

/// A bid on an auction contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContractBid {
    pub bid_id: i64,
    pub bidder_id: i64,
    pub date_bid: DateTime<Utc>,
    pub amount: f64,
}

/// A saved ship fitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFitting {
    pub fitting_id: i64,
    pub name: String,
    pub description: String,
    pub ship_type_id: i32,
    #[serde(default)]
    pub items: Vec<EsiFittingItem>,
}

/// An item in a fitting (used for both GET and POST).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFittingItem {
    pub type_id: i32,
    pub flag: i32,
    pub quantity: i32,
}

/// Body for creating a new fitting.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewFitting {
    pub name: String,
    pub description: String,
    pub ship_type_id: i32,
    pub items: Vec<EsiFittingItem>,
}

/// Response from creating a fitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiNewFittingResponse {
    pub fitting_id: i64,
}

/// A mail header from a character's inbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailHeader {
    pub mail_id: i64,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub is_read: Option<bool>,
    #[serde(default)]
    pub labels: Vec<i32>,
    #[serde(default)]
    pub recipients: Vec<EsiMailRecipient>,
}

/// A mail recipient (used in both GET and POST).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailRecipient {
    pub recipient_id: i64,
    pub recipient_type: String,
}

/// A mail body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailBody {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub read: Option<bool>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub labels: Vec<i32>,
    #[serde(default)]
    pub recipients: Vec<EsiMailRecipient>,
}

/// Body for sending a new mail.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewMail {
    pub recipients: Vec<EsiMailRecipient>,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_cost: Option<i64>,
}

/// Character mail labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailLabels {
    pub total_unread_count: i32,
    #[serde(default)]
    pub labels: Vec<EsiMailLabel>,
}

/// A single mail label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailLabel {
    pub label_id: i32,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub unread_count: Option<i32>,
}

/// A character notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiNotification {
    pub notification_id: i64,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub sender_id: i64,
    pub sender_type: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub is_read: Option<bool>,
    #[serde(default)]
    pub text: Option<String>,
}

/// A character contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContact {
    pub contact_id: i64,
    pub contact_type: String,
    pub standing: f64,
    #[serde(default)]
    pub label_ids: Vec<i64>,
    #[serde(default)]
    pub is_watched: Option<bool>,
}

/// A contact label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContactLabel {
    pub label_id: i64,
    pub label_name: String,
}

/// A calendar event summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCalendarEvent {
    pub event_id: i64,
    pub event_date: DateTime<Utc>,
    pub title: String,
    #[serde(default)]
    pub importance: Option<i32>,
    #[serde(default)]
    pub event_response: Option<String>,
}

/// A calendar event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCalendarEventDetail {
    pub event_id: i64,
    pub date: DateTime<Utc>,
    pub title: String,
    pub owner_id: i64,
    pub owner_name: String,
    pub owner_type: String,
    pub duration: i32,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub importance: Option<i32>,
    #[serde(default)]
    pub response: Option<String>,
}

/// A calendar event attendee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiEventAttendee {
    pub character_id: i64,
    #[serde(default)]
    pub event_response: Option<String>,
}

/// Body for setting an event response.
#[derive(Debug, Clone, Serialize)]
pub struct EsiEventResponse {
    pub response: String,
}

/// Body for creating a mail label.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewMailLabel {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A mailing list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailingList {
    pub mailing_list_id: i64,
    pub name: String,
}

/// Body for updating mail metadata (read status, labels).
#[derive(Debug, Clone, Serialize)]
pub struct EsiMailUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i32>>,
}

/// Body for opening a new mail window.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewMailWindow {
    pub recipients: Vec<i64>,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_corp_or_alliance_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_mailing_list_id: Option<i64>,
}
