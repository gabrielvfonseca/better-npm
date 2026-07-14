use serde::{Deserialize, Serialize};

const DEFAULT_WEIGHTS: [f64; 6] = [0.25, 0.15, 0.20, 0.20, 0.10, 0.10];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSignal {
    pub name: String,
    pub weight: f64,
    pub value: f64,
    pub source: String,
}

impl ReputationSignal {
    pub fn new(name: &str, weight: f64, value: f64, source: &str) -> Self {
        Self {
            name: name.to_string(),
            weight,
            value: value.clamp(0.0, 1.0),
            source: source.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    pub package_id: String,
    pub signals: Vec<ReputationSignal>,
    pub overall: f64,
}

impl ReputationScore {
    pub fn new(package_id: &str) -> Self {
        Self { package_id: package_id.to_string(), signals: Vec::new(), overall: 0.0 }
    }

    pub fn with_signals(package_id: &str, signals: Vec<ReputationSignal>) -> Self {
        let overall = compute_weighted_average(&signals);
        Self { package_id: package_id.to_string(), signals, overall }
    }

    pub fn add_signal(&mut self, signal: ReputationSignal) {
        self.signals.push(signal);
        self.overall = compute_weighted_average(&self.signals);
    }

    pub fn score_human_readable(&self) -> u64 {
        (self.overall * 1000.0).round().clamp(0.0, 1000.0) as u64
    }
}

pub fn provenance_strength(has_provenance: bool, slsa_level: u8) -> f64 {
    if !has_provenance {
        return 0.0;
    }
    (slsa_level as f64).min(4.0) / 4.0
}

pub fn maintenance_cadence(release_count: u64, months_active: f64) -> f64 {
    if months_active <= 0.0 || release_count == 0 {
        return 0.0;
    }
    let releases_per_month = release_count as f64 / months_active;
    (releases_per_month / 4.0).min(1.0)
}

pub fn security_history(cve_count: u64) -> f64 {
    if cve_count == 0 {
        return 1.0;
    }
    (1.0 / (1.0 + cve_count as f64)).max(0.0)
}

pub fn semver_honesty(api_breaking_count: u64, total_releases: u64) -> f64 {
    if total_releases == 0 {
        return 0.5;
    }
    if api_breaking_count == 0 {
        return 1.0;
    }
    let breaking_ratio = api_breaking_count as f64 / total_releases as f64;
    (1.0 - breaking_ratio).max(0.0)
}

pub fn verified_production_use(graph_roots_containing_package: u64) -> f64 {
    (graph_roots_containing_package as f64 / 100.0).min(1.0)
}

pub fn capability_footprint(subprocess: u64, net_count: u64) -> f64 {
    let total = subprocess.saturating_add(net_count);
    if total == 0 {
        return 1.0;
    }
    (1.0 / (1.0 + total as f64)).max(0.0)
}

fn compute_weighted_average(signals: &[ReputationSignal]) -> f64 {
    if signals.is_empty() {
        return 0.0;
    }
    let total_weight: f64 = signals.iter().map(|s| s.weight).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    let weighted_sum: f64 = signals.iter().map(|s| s.value * s.weight).sum();
    weighted_sum / total_weight
}

pub fn compute_overall(scores: &[ReputationScore], weights: Option<&[f64]>) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let w = weights.unwrap_or(&DEFAULT_WEIGHTS);
    let count = scores.len().min(6);
    let total_weight: f64 = w.iter().take(count).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    let weighted_sum: f64 = scores
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, s)| {
            let wgt = w.get(i).copied().unwrap_or(1.0);
            s.overall * wgt
        })
        .sum();
    weighted_sum / total_weight
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherHistory {
    pub publisher: String,
    pub account_age_days: u64,
    pub package_count: u64,
    pub verified_identity: bool,
    pub has_2fa: bool,
    pub past_incidents: u64,
}

pub fn check_sybil_signals(history: &PublisherHistory) -> bool {
    let mut suspicious_signals = 0u64;

    if history.account_age_days < 30 {
        suspicious_signals += 1;
    }
    if history.package_count > 50 && history.account_age_days < 365 {
        suspicious_signals += 1;
    }
    if !history.verified_identity {
        suspicious_signals += 1;
    }
    if !history.has_2fa {
        suspicious_signals += 1;
    }
    if history.past_incidents > 3 {
        suspicious_signals += 1;
    }
    if history.package_count == 0 {
        suspicious_signals += 1;
    }

    suspicious_signals >= 3
}

#[allow(clippy::too_many_arguments)]
pub fn compute_full_score(
    package_id: &str,
    has_provenance: bool,
    slsa_level: u8,
    release_count: u64,
    months_active: f64,
    cve_count: u64,
    api_breaking_count: u64,
    total_releases: u64,
    graph_roots: u64,
    subprocess_caps: u64,
    net_caps: u64,
) -> ReputationScore {
    let signals = vec![
        ReputationSignal::new(
            "provenance_strength",
            DEFAULT_WEIGHTS[0],
            provenance_strength(has_provenance, slsa_level),
            "provenance attestation",
        ),
        ReputationSignal::new(
            "maintenance_cadence",
            DEFAULT_WEIGHTS[1],
            maintenance_cadence(release_count, months_active),
            "registry release history",
        ),
        ReputationSignal::new(
            "security_history",
            DEFAULT_WEIGHTS[2],
            security_history(cve_count),
            "CVE database",
        ),
        ReputationSignal::new(
            "semver_honesty",
            DEFAULT_WEIGHTS[3],
            semver_honesty(api_breaking_count, total_releases),
            "type diff analysis",
        ),
        ReputationSignal::new(
            "verified_production_use",
            DEFAULT_WEIGHTS[4],
            verified_production_use(graph_roots),
            "org dependency graphs",
        ),
        ReputationSignal::new(
            "capability_footprint",
            DEFAULT_WEIGHTS[5],
            capability_footprint(subprocess_caps, net_caps),
            "manifest declared capabilities",
        ),
    ];
    ReputationScore::with_signals(package_id, signals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_strength_no_provenance() {
        assert!((provenance_strength(false, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provenance_strength_slsa4() {
        assert!((provenance_strength(true, 4) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provenance_strength_slsa2() {
        assert!((provenance_strength(true, 2) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_maintenance_cadence_regular() {
        let score = maintenance_cadence(24, 12.0);
        assert!(score > 0.0 && score <= 1.0);
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_maintenance_cadence_none() {
        assert!((maintenance_cadence(0, 12.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_security_history_no_cves() {
        assert!((security_history(0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_security_history_with_cves() {
        let score = security_history(3);
        assert!(score < 1.0 && score > 0.0);
        assert!((score - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_semver_honesty_perfect() {
        assert!((semver_honesty(0, 10) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_semver_honesty_some_breaking() {
        let score = semver_honesty(2, 10);
        assert!((score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_semver_honesty_all_breaking() {
        assert!((semver_honesty(10, 10) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_verified_production_use() {
        assert!((verified_production_use(50) - 0.5).abs() < 0.01);
        assert!((verified_production_use(200) - 1.0).abs() < f64::EPSILON);
        assert!((verified_production_use(0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_capability_footprint_none() {
        assert!((capability_footprint(0, 0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_capability_footprint_some() {
        let score = capability_footprint(2, 1);
        assert!((score - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_known_good_scores_higher_than_known_bad() {
        let good = compute_full_score("@good/trusted", true, 4, 48, 24.0, 0, 0, 20, 85, 0, 0);
        let bad = compute_full_score("@bad/suspicious", false, 0, 1, 0.5, 5, 8, 10, 0, 5, 3);

        assert!(good.overall > bad.overall);
        assert!(good.score_human_readable() > bad.score_human_readable());
    }

    #[test]
    fn test_reputation_score_human_readable() {
        let score = compute_full_score("@acme/pkg", true, 4, 24, 12.0, 0, 0, 10, 50, 0, 0);
        let hr = score.score_human_readable();
        assert!(hr <= 1000);
        assert!(hr > 0);
    }

    #[test]
    fn test_sybil_detection_new_account() {
        let history = PublisherHistory {
            publisher: "did:dash:user:suspicious".into(),
            account_age_days: 5,
            package_count: 100,
            verified_identity: false,
            has_2fa: false,
            past_incidents: 0,
        };
        assert!(check_sybil_signals(&history));
    }

    #[test]
    fn test_sybil_detection_legitimate() {
        let history = PublisherHistory {
            publisher: "did:dash:user:trusted".into(),
            account_age_days: 800,
            package_count: 5,
            verified_identity: true,
            has_2fa: true,
            past_incidents: 0,
        };
        assert!(!check_sybil_signals(&history));
    }

    #[test]
    fn test_sybil_detection_many_incidents() {
        let history = PublisherHistory {
            publisher: "did:dash:user:risky".into(),
            account_age_days: 20,
            package_count: 10,
            verified_identity: false,
            has_2fa: false,
            past_incidents: 5,
        };
        assert!(check_sybil_signals(&history));
    }

    #[test]
    fn test_sybil_detection_unverified_with_incidents() {
        let history = PublisherHistory {
            publisher: "did:dash:user:risky".into(),
            account_age_days: 500,
            package_count: 10,
            verified_identity: false,
            has_2fa: false,
            past_incidents: 5,
        };
        assert!(check_sybil_signals(&history));
    }

    #[test]
    fn test_sybil_detection_no_packages() {
        let history = PublisherHistory {
            publisher: "did:dash:user:empty".into(),
            account_age_days: 100,
            package_count: 0,
            verified_identity: false,
            has_2fa: false,
            past_incidents: 0,
        };
        assert!(check_sybil_signals(&history));
    }

    #[test]
    fn test_compute_overall_uses_weights() {
        let s1 = ReputationScore::with_signals(
            "a",
            vec![ReputationSignal::new("signal", 1.0, 1.0, "test")],
        );
        let s2 = ReputationScore::with_signals(
            "b",
            vec![ReputationSignal::new("signal", 1.0, 0.0, "test")],
        );
        let overall = compute_overall(&[s1, s2], None);
        let expected = (1.0 * 0.25 + 0.0 * 0.15) / (0.25 + 0.15);
        assert!((overall - expected).abs() < 0.01);
    }

    #[test]
    fn test_compute_overall_empty() {
        assert!((compute_overall(&[], None) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reputation_signal_clamped() {
        let signal = ReputationSignal::new("test", 1.0, 1.5, "source");
        assert!((signal.value - 1.0).abs() < f64::EPSILON);
        let signal = ReputationSignal::new("test", 1.0, -0.5, "source");
        assert!((signal.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reputation_score_with_signals() {
        let signals = vec![ReputationSignal::new("a", 0.5, 1.0, "s1")];
        let score = ReputationScore::with_signals("@test/pkg", signals);
        assert!((score.overall - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add_signal_updates_overall() {
        let mut score = ReputationScore::new("@test/pkg");
        assert!((score.overall - 0.0).abs() < f64::EPSILON);
        score.add_signal(ReputationSignal::new("a", 1.0, 1.0, "s1"));
        assert!((score.overall - 1.0).abs() < f64::EPSILON);
        score.add_signal(ReputationSignal::new("b", 1.0, 0.0, "s2"));
        assert!((score.overall - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_serde_roundtrip() {
        let score = compute_full_score("@acme/pkg", true, 4, 24, 12.0, 0, 0, 10, 50, 0, 0);
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: ReputationScore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.package_id, score.package_id);
        assert_eq!(deserialized.signals.len(), score.signals.len());
        assert!((deserialized.overall - score.overall).abs() < 0.01);
    }
}
