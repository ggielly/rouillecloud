use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// CalDAV Protocol Implementation (RFC 4791)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_shared: bool,
    pub permissions: CalendarPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarPermissions {
    pub read_users: Vec<Uuid>,
    pub write_users: Vec<Uuid>,
    pub admin_users: Vec<Uuid>,
    pub public_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub uid: String, // iCalendar UID
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub recurrence_rule: Option<String>,
    pub attendees: Vec<Attendee>,
    pub organizer: Option<Organizer>,
    pub status: EventStatus,
    pub priority: u8,
    pub transparency: Transparency,
    pub categories: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sequence: u32,
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendee {
    pub email: String,
    pub name: Option<String>,
    pub role: AttendeeRole,
    pub status: AttendeeStatus,
    pub rsvp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttendeeRole {
    Required,
    Optional,
    Chair,
    NonParticipant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttendeeStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organizer {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transparency {
    Opaque,
    Transparent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarQuery {
    pub calendar_id: Uuid,
    pub time_range: Option<TimeRange>,
    pub filters: Vec<CalendarFilter>,
    pub properties: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CalendarFilter {
    ComponentFilter {
        name: String,
        time_range: Option<TimeRange>,
        property_filters: Vec<PropertyFilter>,
    },
    PropertyFilter(PropertyFilter),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyFilter {
    pub name: String,
    pub is_not_defined: Option<bool>,
    pub time_range: Option<TimeRange>,
    pub text_match: Option<TextMatch>,
    pub parameter_filters: Vec<ParameterFilter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextMatch {
    pub collation: Option<String>,
    pub negate_condition: bool,
    pub match_type: MatchType,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MatchType {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterFilter {
    pub name: String,
    pub is_not_defined: Option<bool>,
    pub text_match: Option<TextMatch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarReport {
    pub events: Vec<CalendarEvent>,
    pub total_count: u64,
    pub sync_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FreeBusyRequest {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub attendees: Vec<String>,
    pub organizer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FreeBusyResponse {
    pub organizer: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub busy_periods: Vec<BusyPeriod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BusyPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub busy_type: BusyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BusyType {
    Busy,
    Tentative,
    Unavailable,
}
