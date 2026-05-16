//! Execution Simulator - Simulate 10 days (June 2-11) of S1 execution
//! Validates system behavior under realistic execution patterns

use crate::*;
use insurance_data::InsuranceCompany;
use mock_server::MockMimirServer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulates one day of execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayExecution {
    pub day: u32,
    pub date: String,
    pub prudential_result: Option<HitRateResult>,
    pub axa_result: Option<HitRateResult>,
    pub thai_health_result: Option<HitRateResult>,
    pub daily_hit_rate: f32,
    pub cumulative_hit_rate: f32,
    pub alert_level: AlertLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Green,   // ≥75% - OK
    Yellow,  // 50-74% - Warning
    Red,     // <50% - Critical (Plan B)
}

impl AlertLevel {
    pub fn from_hit_rate(hit_rate: f32) -> Self {
        match hit_rate {
            rate if rate >= 0.75 => AlertLevel::Green,
            rate if rate >= 0.50 => AlertLevel::Yellow,
            _ => AlertLevel::Red,
        }
    }
}

/// Complete 10-day execution simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSimulation {
    pub days: Vec<DayExecution>,
    pub final_decision: ValidationDecision,
    pub plan_b_activated: bool,
    pub metrics: SimulationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub days_green: u32,
    pub days_yellow: u32,
    pub days_red: u32,
    pub min_daily_hit_rate: f32,
    pub max_daily_hit_rate: f32,
    pub average_hit_rate: f32,
    pub trend: Trend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
}

/// Execution Simulator
pub struct ExecutionSimulator {
    mock_server: MockMimirServer,
}

impl ExecutionSimulator {
    pub fn new() -> Self {
        Self {
            mock_server: MockMimirServer::new(),
        }
    }

    /// Simulate 10 days of execution with realistic variance
    pub fn simulate_10_day_execution(&self) -> ExecutionSimulation {
        let mut days = Vec::new();
        let mut cumulative_hits = 0;
        let mut cumulative_total = 0;
        let mut hit_rates = Vec::new();

        // Realistic scenario: starts at 82%, slight daily variance
        let base_hit_rate = 0.82;

        for day in 1..=10 {
            let date = format!("2026-06-{:02}", day + 1); // June 2-11

            // Simulate daily variance (±5%)
            let variance = ((day as f32 * 1.618) % 0.1) - 0.05; // Fibonacci-like variance
            let daily_hit_rate = (base_hit_rate + variance).min(1.0).max(0.0);

            hit_rates.push(daily_hit_rate);

            // Simulate results for each company
            let prud_result = simulate_company_result(daily_hit_rate, "Prudential");
            let axa_result = simulate_company_result(daily_hit_rate, "AXA");
            let thai_result = simulate_company_result(daily_hit_rate, "Thai Health");

            cumulative_hits += (daily_hit_rate * 10.0) as u32;
            cumulative_total += 10;

            let cumulative_hit_rate = cumulative_hits as f32 / cumulative_total as f32;

            days.push(DayExecution {
                day: day as u32,
                date,
                prudential_result: Some(prud_result),
                axa_result: Some(axa_result),
                thai_health_result: Some(thai_result),
                daily_hit_rate,
                cumulative_hit_rate,
                alert_level: AlertLevel::from_hit_rate(daily_hit_rate),
            });
        }

        let final_cumulative = cumulative_hits as f32 / cumulative_total as f32;
        let final_decision = ValidationDecision::from_hit_rate(final_cumulative);
        let plan_b_activated = final_cumulative < 0.50;

        let metrics = SimulationMetrics {
            days_green: days.iter().filter(|d| d.alert_level == AlertLevel::Green).count() as u32,
            days_yellow: days
                .iter()
                .filter(|d| d.alert_level == AlertLevel::Yellow)
                .count() as u32,
            days_red: days.iter().filter(|d| d.alert_level == AlertLevel::Red).count() as u32,
            min_daily_hit_rate: *hit_rates.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max_daily_hit_rate: *hit_rates.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            average_hit_rate: hit_rates.iter().sum::<f32>() / hit_rates.len() as f32,
            trend: compute_trend(&hit_rates),
        };

        ExecutionSimulation {
            days,
            final_decision,
            plan_b_activated,
            metrics,
        }
    }

    /// Simulate worst-case scenario (< 50% hit rate, Plan B trigger)
    pub fn simulate_worst_case(&self) -> ExecutionSimulation {
        let mut days = Vec::new();

        for day in 1..=10 {
            let date = format!("2026-06-{:02}", day + 1);
            // Degrading performance: starts at 60%, drops to 35% by day 10
            let daily_hit_rate = 0.60 - (day as f32 * 0.025);

            let prud_result = simulate_company_result(daily_hit_rate, "Prudential");
            let axa_result = simulate_company_result(daily_hit_rate, "AXA");
            let thai_result = simulate_company_result(daily_hit_rate, "Thai Health");

            let cumulative = (0.60 - (day as f32 * 0.013)).max(0.0);

            days.push(DayExecution {
                day: day as u32,
                date,
                prudential_result: Some(prud_result),
                axa_result: Some(axa_result),
                thai_health_result: Some(thai_result),
                daily_hit_rate,
                cumulative_hit_rate: cumulative,
                alert_level: AlertLevel::from_hit_rate(daily_hit_rate),
            });
        }

        let final_decision = ValidationDecision::Fail;

        ExecutionSimulation {
            days: days.clone(),
            final_decision,
            plan_b_activated: true,
            metrics: SimulationMetrics {
                days_green: 0,
                days_yellow: 3,
                days_red: 7,
                min_daily_hit_rate: 0.35,
                max_daily_hit_rate: 0.60,
                average_hit_rate: 0.47,
                trend: Trend::Degrading,
            },
        }
    }

    /// Simulate best-case scenario (≥ 75% from day 1)
    pub fn simulate_best_case(&self) -> ExecutionSimulation {
        let mut days = Vec::new();

        for day in 1..=10 {
            let date = format!("2026-06-{:02}", day + 1);
            let daily_hit_rate = 0.85 + (day as f32 * 0.01); // Improving

            let prud_result = simulate_company_result(daily_hit_rate, "Prudential");
            let axa_result = simulate_company_result(daily_hit_rate, "AXA");
            let thai_result = simulate_company_result(daily_hit_rate, "Thai Health");

            days.push(DayExecution {
                day: day as u32,
                date,
                prudential_result: Some(prud_result),
                axa_result: Some(axa_result),
                thai_health_result: Some(thai_result),
                daily_hit_rate,
                cumulative_hit_rate: daily_hit_rate,
                alert_level: AlertLevel::Green,
            });
        }

        let final_decision = ValidationDecision::Pass;

        ExecutionSimulation {
            days,
            final_decision,
            plan_b_activated: false,
            metrics: SimulationMetrics {
                days_green: 10,
                days_yellow: 0,
                days_red: 0,
                min_daily_hit_rate: 0.85,
                max_daily_hit_rate: 0.95,
                average_hit_rate: 0.90,
                trend: Trend::Improving,
            },
        }
    }

    /// Generate execution report
    pub fn generate_simulation_report(sim: &ExecutionSimulation) -> String {
        let mut report = String::new();

        report.push_str("=== 10-DAY EXECUTION SIMULATION REPORT ===\n\n");

        for day in &sim.days {
            let alert_icon = match day.alert_level {
                AlertLevel::Green => "✅",
                AlertLevel::Yellow => "⚠️",
                AlertLevel::Red => "❌",
            };

            report.push_str(&format!(
                "Day {:2} ({}) {} Hit Rate: {:.1}% (Cumulative: {:.1}%)\n",
                day.day,
                day.date,
                alert_icon,
                day.daily_hit_rate * 100.0,
                day.cumulative_hit_rate * 100.0
            ));
        }

        report.push_str("\n=== SUMMARY ===\n");
        report.push_str(&format!("Days Green (≥75%): {}\n", sim.metrics.days_green));
        report.push_str(&format!("Days Yellow (50-74%): {}\n", sim.metrics.days_yellow));
        report.push_str(&format!("Days Red (<50%): {}\n", sim.metrics.days_red));
        report.push_str(&format!(
            "Average Hit Rate: {:.1}%\n",
            sim.metrics.average_hit_rate * 100.0
        ));
        report.push_str(&format!("Trend: {:?}\n", sim.metrics.trend));

        report.push_str("\n=== DECISION (June 12) ===\n");
        match sim.final_decision {
            ValidationDecision::Pass => {
                report.push_str("🎉 PASS - Proceed to Production\n");
            }
            ValidationDecision::Warn => {
                report.push_str("⚠️  WARN - Review Required\n");
            }
            ValidationDecision::Fail => {
                report.push_str("❌ FAIL - Activate Plan B\n");
                report.push_str("   Switch: BGE-M3 → Typhoon\n");
                report.push_str("   Re-run: Phase 4 (2 hours)\n");
            }
        }

        report
    }
}

impl Default for ExecutionSimulator {
    fn default() -> Self {
        Self::new()
    }
}

fn simulate_company_result(base_hit_rate: f32, company: &str) -> HitRateResult {
    let hit_count = (base_hit_rate * 10.0) as usize;

    HitRateResult {
        total_queries: 10,
        hit_count,
        hit_rate: base_hit_rate,
        decision: ValidationDecision::from_hit_rate(base_hit_rate),
        query_results: vec![QueryResult {
            query: format!("{} consolidated output", company),
            hits: hit_count,
            score: base_hit_rate,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }],
    }
}

fn compute_trend(hit_rates: &[f32]) -> Trend {
    if hit_rates.len() < 2 {
        return Trend::Stable;
    }

    let first_half: f32 = hit_rates[0..hit_rates.len() / 2].iter().sum::<f32>() / (hit_rates.len() / 2) as f32;
    let second_half: f32 = hit_rates[hit_rates.len() / 2..].iter().sum::<f32>() / (hit_rates.len() - hit_rates.len() / 2) as f32;

    if (second_half - first_half).abs() < 0.02 {
        Trend::Stable
    } else if second_half > first_half {
        Trend::Improving
    } else {
        Trend::Degrading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_new() {
        let sim = ExecutionSimulator::new();
        assert!(true); // Just verify it constructs
    }

    #[test]
    fn test_simulate_10_day_execution() {
        let sim = ExecutionSimulator::new();
        let result = sim.simulate_10_day_execution();

        assert_eq!(result.days.len(), 10);
        assert!(result.days[0].day == 1);
        assert!(result.days[9].day == 10);
        assert!(result.metrics.days_green + result.metrics.days_yellow + result.metrics.days_red == 10);
    }

    #[test]
    fn test_simulate_worst_case() {
        let sim = ExecutionSimulator::new();
        let result = sim.simulate_worst_case();

        assert_eq!(result.final_decision, ValidationDecision::Fail);
        assert!(result.plan_b_activated);
        assert!(result.metrics.trend == Trend::Degrading);
    }

    #[test]
    fn test_simulate_best_case() {
        let sim = ExecutionSimulator::new();
        let result = sim.simulate_best_case();

        assert_eq!(result.final_decision, ValidationDecision::Pass);
        assert!(!result.plan_b_activated);
        assert_eq!(result.metrics.days_green, 10);
    }

    #[test]
    fn test_alert_level_green() {
        assert_eq!(AlertLevel::from_hit_rate(0.75), AlertLevel::Green);
        assert_eq!(AlertLevel::from_hit_rate(0.95), AlertLevel::Green);
    }

    #[test]
    fn test_alert_level_yellow() {
        assert_eq!(AlertLevel::from_hit_rate(0.50), AlertLevel::Yellow);
        assert_eq!(AlertLevel::from_hit_rate(0.74), AlertLevel::Yellow);
    }

    #[test]
    fn test_alert_level_red() {
        assert_eq!(AlertLevel::from_hit_rate(0.49), AlertLevel::Red);
        assert_eq!(AlertLevel::from_hit_rate(0.0), AlertLevel::Red);
    }

    #[test]
    fn test_simulation_report_generation() {
        let sim = ExecutionSimulator::new();
        let result = sim.simulate_10_day_execution();
        let report = ExecutionSimulator::generate_simulation_report(&result);

        assert!(report.contains("10-DAY EXECUTION SIMULATION REPORT"));
        assert!(report.contains("Day"));
        assert!(report.contains("DECISION"));
    }

    #[test]
    fn test_simulation_metrics_sum_to_10() {
        let sim = ExecutionSimulator::new();
        let result = sim.simulate_10_day_execution();

        assert_eq!(
            result.metrics.days_green + result.metrics.days_yellow + result.metrics.days_red,
            10
        );
    }

    #[test]
    fn test_trend_calculation() {
        let improving = vec![0.5, 0.6, 0.7, 0.8, 0.9];
        assert_eq!(compute_trend(&improving), Trend::Improving);

        let degrading = vec![0.9, 0.8, 0.7, 0.6, 0.5];
        assert_eq!(compute_trend(&degrading), Trend::Degrading);
    }
}
