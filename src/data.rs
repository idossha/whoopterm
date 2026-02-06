use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub user_id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recovery {
    pub cycle_id: i64,
    pub sleep_id: String,
    pub score: RecoveryScore,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryScore {
    pub recovery_score: i32,
    pub resting_heart_rate: i32,
    pub hrv_rmssd_milli: f64,
    pub user_calibrating: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sleep {
    pub id: String,
    pub cycle_id: i64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub score: SleepScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepScore {
    pub stage_summary: SleepStageSummary,
    pub sleep_efficiency_percentage: i32,
    pub sleep_performance_percentage: i32,
    pub sleep_consistency_percentage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepStageSummary {
    pub total_in_bed_time_milli: i64,
    pub total_awake_time_milli: i64,
    pub total_light_sleep_time_milli: i64,
    pub total_slow_wave_sleep_time_milli: i64,
    pub total_rem_sleep_time_milli: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workout {
    pub id: String,
    pub sport_name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub score: WorkoutScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutScore {
    pub strain: f64,
    pub average_heart_rate: i32,
    pub max_heart_rate: i32,
    pub kilojoule: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub profile: Option<Profile>,
    pub recovery: Vec<Recovery>,
    pub sleep: Vec<Sleep>,
    pub workouts: Vec<Workout>,
    pub refreshed_at: Option<DateTime<Utc>>,
}
