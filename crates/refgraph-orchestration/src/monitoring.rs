//! Monitoring and Alerting System for S1 Execution
//! Detects anomalies and generates alerts for human intervention

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Execution metric tracked daily
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetric {
    pub day: u32,
    pub date: String,
    pub hit_rate: f32,
    pub prudential_hits: usize,
    pub axa_hits: usize,
    pub thai_health_hits: usize,
    pub total_queries: usize,
}

/// Alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// Hit rate dropped below 50% - Plan B trigger
    CriticalThresholdBreach,
    /// Hit rate in warning zone 50-74%
    WarningZone,
    /// Sudden drop > 10% from previous day
    SuddenDegradation,
    /// Degrading trend over 3+ days
    TrendDegradation,
    /// One company significantly underperforming
    CompanyUnderperformance,
    /// Recovery after alert
    Recovery,
}

/// Generated alert with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: AlertType,
    pub severity: Severity,
    pub day: u32,
    pub message: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Monitoring engine
pub struct Monitor {
    metrics_history: VecDeque<DailyMetric>,
    alerts: Vec<Alert>,
    max_history: usize,
}

impl Monitor {
    pub fn new(max_history: usize) -> Self {
        Self {
            metrics_history: VecDeque::with_capacity(max_history),
            alerts: Vec::new(),
            max_history,
        }
    }

    /// Record a day's metrics and check for anomalies
    pub fn record_day(&mut self, metric: DailyMetric) {
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.pop_front();
        }

        // Check for anomalies
        self.check_for_alerts(&metric);

        self.metrics_history.push_back(metric);
    }

    fn check_for_alerts(&mut self, metric: &DailyMetric) {
        // Alert 1: Critical threshold breach
        if metric.hit_rate < 0.50 {
            self.alerts.push(Alert {
                alert_type: AlertType::CriticalThresholdBreach,
                severity: Severity::Critical,
                day: metric.day,
                message: format!(
                    "Hit rate {:.1}% < 50% threshold - Plan B activation required",
                    metric.hit_rate * 100.0
                ),
                recommended_action: "Activate Plan B: Switch embedding model BGE-M3 → Typhoon"
                    .to_string(),
            });
        }
        // Alert 2: Warning zone
        else if metric.hit_rate < 0.75 && metric.hit_rate >= 0.50 {
            self.alerts.push(Alert {
                alert_type: AlertType::WarningZone,
                severity: Severity::Warning,
                day: metric.day,
                message: format!(
                    "Hit rate {:.1}% in warning zone (50-74%)",
                    metric.hit_rate * 100.0
                ),
                recommended_action: "Review consolidation quality, monitor trend".to_string(),
            });
        }

        // Alert 3: Sudden degradation
        if let Some(prev) = self.metrics_history.back() {
            let drop = prev.hit_rate - metric.hit_rate;
            if drop > 0.10 {
                self.alerts.push(Alert {
                    alert_type: AlertType::SuddenDegradation,
                    severity: Severity::Warning,
                    day: metric.day,
                    message: format!(
                        "Sudden drop: {:.1}% → {:.1}% ({:.1}% decrease)",
                        prev.hit_rate * 100.0,
                        metric.hit_rate * 100.0,
                        drop * 100.0
                    ),
                    recommended_action: "Investigate consolidation for day {} issues".to_string(),
                });
            }

            // Alert 4: Trend degradation
            if self.metrics_history.len() >= 3 {
                let recent_avg = self.recent_average(3);
                if metric.hit_rate < recent_avg - 0.05 {
                    self.alerts.push(Alert {
                        alert_type: AlertType::TrendDegradation,
                        severity: Severity::Warning,
                        day: metric.day,
                        message: format!(
                            "Degrading trend detected: {:.1}% below 3-day average {:.1}%",
                            metric.hit_rate * 100.0,
                            recent_avg * 100.0
                        ),
                        recommended_action: "Monitor for continued degradation".to_string(),
                    });
                }
            }
        }

        // Alert 5: Company underperformance
        let avg_per_company = metric.total_queries as f32 / 3.0;
        let underperforming = (metric.prudential_hits as f32 / avg_per_company < 0.8)
            || (metric.axa_hits as f32 / avg_per_company < 0.8)
            || (metric.thai_health_hits as f32 / avg_per_company < 0.8);

        if underperforming {
            let companies = [
                ("Prudential", metric.prudential_hits),
                ("AXA", metric.axa_hits),
                ("Thai Health", metric.thai_health_hits),
            ];

            for (name, hits) in companies {
                if (hits as f32) < avg_per_company * 0.8 {
                    self.alerts.push(Alert {
                        alert_type: AlertType::CompanyUnderperformance,
                        severity: Severity::Warning,
                        day: metric.day,
                        message: format!(
                            "{} underperforming: {} hits vs {:.1} average",
                            name, hits, avg_per_company
                        ),
                        recommended_action: format!(
                            "Review {} consolidation process",
                            name.to_lowercase()
                        ),
                    });
                }
            }
        }
    }

    pub fn recent_average(&self, days: usize) -> f32 {
        let take = self.metrics_history.len().min(days);
        if take == 0 {
            return 0.0;
        }

        self.metrics_history
            .iter()
            .rev()
            .take(take)
            .map(|m| m.hit_rate)
            .sum::<f32>()
            / take as f32
    }

    pub fn get_alerts(&self) -> Vec<Alert> {
        self.alerts.clone()
    }

    pub fn get_critical_alerts(&self) -> Vec<Alert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == Severity::Critical)
            .cloned()
            .collect()
    }

    pub fn get_metrics_history(&self) -> Vec<DailyMetric> {
        self.metrics_history.iter().cloned().collect()
    }

    /// Generate daily monitoring report
    pub fn generate_daily_report(&self, day: u32) -> String {
        let mut report = String::new();

        if let Some(metric) = self.metrics_history.back() {
            if metric.day == day {
                report.push_str(&format!("=== DAY {} MONITORING REPORT ===\n\n", day));
                report.push_str(&format!("Hit Rate: {:.1}%\n", metric.hit_rate * 100.0));
                report.push_str(&format!("  Prudential: {}/{}\n", metric.prudential_hits, metric.total_queries / 3));
                report.push_str(&format!("  AXA: {}/{}\n", metric.axa_hits, metric.total_queries / 3));
                report.push_str(&format!("  Thai Health: {}/{}\n", metric.thai_health_hits, metric.total_queries / 3));

                let day_alerts: Vec<_> = self.alerts.iter().filter(|a| a.day == day).collect();
                if !day_alerts.is_empty() {
                    report.push_str(&format!("\nAlerts ({}):\n", day_alerts.len()));
                    for alert in day_alerts {
                        report.push_str(&format!("  [{:?}] {}\n", alert.severity, alert.message));
                        report.push_str(&format!("         → {}\n", alert.recommended_action));
                    }
                } else {
                    report.push_str("\nNo alerts - System healthy ✅\n");
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_new() {
        let monitor = Monitor::new(10);
        assert_eq!(monitor.get_alerts().len(), 0);
    }

    #[test]
    fn test_monitor_critical_alert() {
        let mut monitor = Monitor::new(10);
        let metric = DailyMetric {
            day: 1,
            date: "2026-06-02".to_string(),
            hit_rate: 0.45,
            prudential_hits: 1,
            axa_hits: 1,
            thai_health_hits: 2,
            total_queries: 10,
        };

        monitor.record_day(metric);

        let alerts = monitor.get_critical_alerts();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].alert_type, AlertType::CriticalThresholdBreach);
    }

    #[test]
    fn test_monitor_warning_alert() {
        let mut monitor = Monitor::new(10);
        let metric = DailyMetric {
            day: 1,
            date: "2026-06-02".to_string(),
            hit_rate: 0.60,
            prudential_hits: 2,
            axa_hits: 2,
            thai_health_hits: 2,
            total_queries: 10,
        };

        monitor.record_day(metric);

        let alerts = monitor.get_alerts();
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::WarningZone));
    }

    #[test]
    fn test_monitor_sudden_degradation() {
        let mut monitor = Monitor::new(10);

        // Day 1: Good
        monitor.record_day(DailyMetric {
            day: 1,
            date: "2026-06-02".to_string(),
            hit_rate: 0.85,
            prudential_hits: 3,
            axa_hits: 3,
            thai_health_hits: 3,
            total_queries: 10,
        });

        // Day 2: Sudden drop
        monitor.record_day(DailyMetric {
            day: 2,
            date: "2026-06-03".to_string(),
            hit_rate: 0.70,
            prudential_hits: 2,
            axa_hits: 2,
            thai_health_hits: 3,
            total_queries: 10,
        });

        let alerts = monitor.get_alerts();
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == AlertType::SuddenDegradation));
    }

    #[test]
    fn test_monitor_recent_average() {
        let mut monitor = Monitor::new(10);

        for day in 1..=5 {
            monitor.record_day(DailyMetric {
                day,
                date: format!("2026-06-{:02}", day + 1),
                hit_rate: 0.80,
                prudential_hits: 3,
                axa_hits: 3,
                thai_health_hits: 3,
                total_queries: 10,
            });
        }

        let avg = monitor.recent_average(3);
        assert!((avg - 0.80).abs() < 0.001);
    }

    #[test]
    fn test_monitor_company_underperformance() {
        let mut monitor = Monitor::new(10);

        monitor.record_day(DailyMetric {
            day: 1,
            date: "2026-06-02".to_string(),
            hit_rate: 0.70,
            prudential_hits: 1, // Low
            axa_hits: 3,
            thai_health_hits: 3,
            total_queries: 10,
        });

        let alerts = monitor.get_alerts();
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == AlertType::CompanyUnderperformance));
    }

    #[test]
    fn test_monitor_report_generation() {
        let mut monitor = Monitor::new(10);

        monitor.record_day(DailyMetric {
            day: 1,
            date: "2026-06-02".to_string(),
            hit_rate: 0.82,
            prudential_hits: 3,
            axa_hits: 2,
            thai_health_hits: 3,
            total_queries: 10,
        });

        let report = monitor.generate_daily_report(1);
        assert!(report.contains("DAY 1"));
        assert!(report.contains("82.0%"));
    }
}
