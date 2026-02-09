use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Profile ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub user_id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

// ── Recovery ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recovery {
    pub cycle_id: i64,
    pub sleep_id: String,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub score_state: String,
    pub score: Option<RecoveryScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryScore {
    pub recovery_score: f64,
    pub resting_heart_rate: f64,
    pub hrv_rmssd_milli: f64,
    pub user_calibrating: bool,
    #[serde(default)]
    pub spo2_percentage: Option<f64>,
    #[serde(default)]
    pub skin_temp_celsius: Option<f64>,
}

// ── Sleep ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sleep {
    pub id: String,
    pub cycle_id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub timezone_offset: String,
    pub nap: bool,
    pub score_state: String,
    pub score: Option<SleepScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepScore {
    pub stage_summary: SleepStageSummary,
    pub sleep_needed: SleepNeeded,
    #[serde(default)]
    pub respiratory_rate: Option<f64>,
    #[serde(default)]
    pub sleep_performance_percentage: Option<f64>,
    #[serde(default)]
    pub sleep_consistency_percentage: Option<f64>,
    #[serde(default)]
    pub sleep_efficiency_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepStageSummary {
    pub total_in_bed_time_milli: i64,
    pub total_awake_time_milli: i64,
    pub total_no_data_time_milli: i64,
    pub total_light_sleep_time_milli: i64,
    pub total_slow_wave_sleep_time_milli: i64,
    pub total_rem_sleep_time_milli: i64,
    pub sleep_cycle_count: i32,
    pub disturbance_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepNeeded {
    pub baseline_milli: i64,
    pub need_from_sleep_debt_milli: i64,
    pub need_from_recent_strain_milli: i64,
    pub need_from_recent_nap_milli: i64,
}

// ── Workout ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workout {
    pub id: String,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub timezone_offset: String,
    pub sport_name: String,
    pub score_state: String,
    pub score: Option<WorkoutScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutScore {
    pub strain: f64,
    pub average_heart_rate: i32,
    pub max_heart_rate: i32,
    pub kilojoule: f64,
    pub percent_recorded: f64,
    pub zone_durations: ZoneDurations,
    #[serde(default)]
    pub distance_meter: Option<f64>,
    #[serde(default)]
    pub altitude_gain_meter: Option<f64>,
    #[serde(default)]
    pub altitude_change_meter: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDurations {
    pub zone_zero_milli: i64,
    pub zone_one_milli: i64,
    pub zone_two_milli: i64,
    pub zone_three_milli: i64,
    pub zone_four_milli: i64,
    pub zone_five_milli: i64,
}

// ── Dashboard aggregate ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub profile: Option<Profile>,
    pub recovery: Vec<Recovery>,
    pub sleep: Vec<Sleep>,
    pub workouts: Vec<Workout>,
    pub refreshed_at: Option<DateTime<Utc>>,
}
