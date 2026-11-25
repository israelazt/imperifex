//! Summing up some samples. Percentiles are by nearest rank (decision D10 of
//! the plan): the rank is `ceil(p / 100 * n)`, counted from 1, with no
//! interpolation, so anyone can reproduce them.

use serde::{Deserialize, Serialize};

/// What is said of a set of samples, in the samples' own unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Summary {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    /// Of the whole population (divided by `count`).
    pub stddev: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Leaves out NaN and the infinities; `None` if nothing is left.
pub fn summarize(samples: &[f64]) -> Option<Summary> {
    let mut sorted: Vec<f64> = samples.iter().copied().filter(|s| s.is_finite()).collect();
    sorted.sort_by(f64::total_cmp);
    let (&min, &max) = (sorted.first()?, sorted.last()?);
    let count = sorted.len();
    let mean = sorted.iter().sum::<f64>() / count as f64;
    let variance = sorted.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / count as f64;
    Some(Summary {
        count,
        min,
        max,
        mean,
        stddev: variance.sqrt(),
        p50: percentile(&sorted, 50.0),
        p95: percentile(&sorted, 95.0),
        p99: percentile(&sorted, 99.0),
    })
}

/// The nearest rank percentile of samples that are sorted and not empty.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    // Multiplying before dividing keeps whole ranks whole.
    let rank = (p * sorted.len() as f64 / 100.0).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_to_a_hundred() -> Vec<f64> {
        (1..=100).map(f64::from).collect()
    }

    #[test]
    fn one_to_a_hundred_has_the_expected_figures() {
        let s = summarize(&one_to_a_hundred()).unwrap();
        assert_eq!((s.count, s.min, s.max, s.mean), (100, 1.0, 100.0, 50.5));
        assert_eq!((s.p50, s.p95, s.p99), (50.0, 95.0, 99.0));
    }

    #[test]
    fn the_deviation_is_of_the_whole_population() {
        let s = summarize(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert_eq!(s.stddev, 2.0);
    }

    #[test]
    fn a_percentile_between_two_ranks_takes_the_higher_one() {
        let s = summarize(&[10.0, 20.0, 30.0]).unwrap();
        assert_eq!((s.p50, s.p95), (20.0, 30.0));
    }

    #[test]
    fn a_single_sample_is_everything() {
        let s = summarize(&[7.5]).unwrap();
        assert_eq!([s.min, s.max, s.mean, s.p50, s.p95, s.p99], [7.5; 6]);
        assert_eq!((s.count, s.stddev), (1, 0.0));
    }

    #[test]
    fn the_order_of_the_samples_does_not_matter() {
        let mut reversed = one_to_a_hundred();
        reversed.reverse();
        assert_eq!(summarize(&reversed), summarize(&one_to_a_hundred()));
    }

    #[test]
    fn what_is_not_a_finite_number_does_not_count() {
        let s = summarize(&[f64::NAN, 3.0, f64::INFINITY, f64::NEG_INFINITY]).unwrap();
        assert_eq!((s.count, s.mean), (1, 3.0));
    }

    #[test]
    fn no_samples_have_no_summary() {
        assert_eq!(summarize(&[]), None);
        assert_eq!(summarize(&[f64::NAN]), None);
    }
}
