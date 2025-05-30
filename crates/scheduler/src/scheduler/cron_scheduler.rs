use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;
use tracing::{info, warn};

use crate::error::{Result, SchedulerError};

/// Cron expression scheduler for calculating next execution times
pub struct CronScheduler {
    timezone: Tz,
}

impl CronScheduler {
    /// Create a new cron scheduler with the specified timezone
    pub fn new(timezone: Option<&str>) -> Result<Self> {
        let tz = match timezone {
            Some(tz_str) => tz_str.parse::<Tz>().map_err(|e| {
                SchedulerError::Validation(format!("Invalid timezone '{}': {}", tz_str, e))
            })?,
            None => Tz::UTC,
        };

        Ok(Self { timezone: tz })
    }

    /// Parse and validate a cron expression
    pub fn parse_cron_expression(&self, cron_expr: &str) -> Result<Schedule> {
        Schedule::from_str(cron_expr).map_err(|e| {
            SchedulerError::Validation(format!("Invalid cron expression '{}': {}", cron_expr, e))
        })
    }

    /// Calculate the next execution time for a cron expression
    pub fn next_execution_time(
        &self,
        cron_expr: &str,
        after: Option<DateTime<Utc>>,
    ) -> Result<Option<DateTime<Utc>>> {
        let schedule = self.parse_cron_expression(cron_expr)?;
        let reference_time = after.unwrap_or_else(Utc::now);

        // Convert to the target timezone for calculation
        let local_time = reference_time.with_timezone(&self.timezone);

        // Get the next execution time in the target timezone
        let next_local = schedule.after(&local_time).next();

        match next_local {
            Some(next) => {
                // Convert back to UTC
                let next_utc = next.with_timezone(&Utc);
                info!(
                    cron_expression = cron_expr,
                    timezone = %self.timezone,
                    reference_time = %reference_time,
                    next_execution = %next_utc,
                    "Calculated next execution time"
                );
                Ok(Some(next_utc))
            }
            None => {
                warn!(
                    cron_expression = cron_expr,
                    timezone = %self.timezone,
                    "No future execution time found for cron expression"
                );
                Ok(None)
            }
        }
    }

    /// Calculate multiple upcoming execution times
    pub fn upcoming_execution_times(
        &self,
        cron_expr: &str,
        count: usize,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<DateTime<Utc>>> {
        let schedule = self.parse_cron_expression(cron_expr)?;
        let reference_time = after.unwrap_or_else(Utc::now);
        let local_time = reference_time.with_timezone(&self.timezone);

        let upcoming: Vec<DateTime<Utc>> = schedule
            .after(&local_time)
            .take(count)
            .map(|dt| dt.with_timezone(&Utc))
            .collect();

        info!(
            cron_expression = cron_expr,
            timezone = %self.timezone,
            count = upcoming.len(),
            "Calculated upcoming execution times"
        );

        Ok(upcoming)
    }

    /// Validate if a cron expression is valid
    pub fn validate_cron_expression(&self, cron_expr: &str) -> Result<()> {
        self.parse_cron_expression(cron_expr)?;

        // Additional validation: ensure the expression can produce at least one future execution
        match self.next_execution_time(cron_expr, None)? {
            Some(_) => Ok(()),
            None => Err(SchedulerError::Validation(format!(
                "Cron expression '{}' does not produce any future execution times",
                cron_expr
            ))),
        }
    }

    /// Get the timezone being used by this scheduler
    pub fn timezone(&self) -> &Tz {
        &self.timezone
    }

    /// Check if a given time matches the cron schedule
    pub fn matches_schedule(&self, cron_expr: &str, time: DateTime<Utc>) -> Result<bool> {
        let schedule = self.parse_cron_expression(cron_expr)?;
        let local_time = time.with_timezone(&self.timezone);

        // Check if the time is within a minute of a scheduled time
        let prev_minute = local_time - chrono::Duration::minutes(1);
        let next_scheduled = schedule.after(&prev_minute).next();

        match next_scheduled {
            Some(scheduled) => {
                let diff = (scheduled.timestamp() - local_time.timestamp()).abs();
                Ok(diff < 60) // Within 1 minute
            }
            None => Ok(false),
        }
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self { timezone: Tz::UTC }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_parse_valid_cron_expression() {
        let scheduler = CronScheduler::default();

        // Test valid expressions (6-field format: sec min hour day month dow)
        assert!(scheduler.parse_cron_expression("0 0 0 * * *").is_ok()); // Daily at midnight
        assert!(scheduler.parse_cron_expression("0 */15 * * * *").is_ok()); // Every 15 minutes
        assert!(scheduler.parse_cron_expression("0 0 9-17 * * 1-5").is_ok()); // Business hours
    }

    #[test]
    fn test_parse_invalid_cron_expression() {
        let scheduler = CronScheduler::default();

        // Test invalid expressions
        assert!(scheduler.parse_cron_expression("invalid").is_err());
        assert!(scheduler.parse_cron_expression("60 60 * * * *").is_err()); // Invalid minute
        assert!(scheduler.parse_cron_expression("0 0 25 * * *").is_err()); // Invalid hour (25)
    }

    #[test]
    fn test_next_execution_time() {
        let scheduler = CronScheduler::default();
        let reference = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        // Test daily at midnight (6-field format: sec min hour day month dow)
        let next = scheduler
            .next_execution_time("0 0 0 * * *", Some(reference))
            .unwrap();
        assert!(next.is_some());

        let next_time = next.unwrap();
        assert_eq!(next_time.hour(), 0);
        assert_eq!(next_time.minute(), 0);
    }

    #[test]
    fn test_timezone_handling() {
        let scheduler = CronScheduler::new(Some("America/New_York")).unwrap();
        assert_eq!(scheduler.timezone().name(), "America/New_York");

        // Test invalid timezone
        assert!(CronScheduler::new(Some("Invalid/Timezone")).is_err());
    }
}
