//! Deterministic statistical calculations on Fixed64.

use crate::Fixed64;
use serde::{Deserialize, Serialize};

/// Summary statistics for a series of Fixed64 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesSummary {
    pub count: usize,
    pub min: Fixed64,
    pub p10: Fixed64,
    pub p25: Fixed64,
    pub median: Fixed64,
    pub p75: Fixed64,
    pub p90: Fixed64,
    pub max: Fixed64,
    pub mean: Fixed64,
    pub variance: Fixed64,
    pub std_dev: Fixed64,
}

/// Compute the deterministic arithmetic mean of a slice of Fixed64 numbers.
pub fn mean(values: &[Fixed64]) -> Fixed64 {
    if values.is_empty() {
        return Fixed64::ZERO;
    }
    let mut sum: i128 = 0;
    for v in values {
        sum += v.raw() as i128;
    }
    let avg = sum / values.len() as i128;
    Fixed64::from_raw(clamp_i128(avg))
}

/// Compute the population variance of a slice of Fixed64 numbers.
pub fn variance(values: &[Fixed64]) -> Fixed64 {
    if values.len() <= 1 {
        return Fixed64::ZERO;
    }
    let avg = mean(values);
    let mut sum_sq: i128 = 0;
    for v in values {
        let diff = *v - avg;
        // (diff * diff) in Q32.32
        let prod = (diff.raw() as i128 * diff.raw() as i128) >> 32;
        sum_sq += prod;
    }
    let var = sum_sq / values.len() as i128;
    Fixed64::from_raw(clamp_i128(var))
}

/// Compute the population standard deviation of a slice of Fixed64 numbers.
pub fn std_dev(values: &[Fixed64]) -> Fixed64 {
    variance(values).sqrt()
}

/// Extract a deterministic interpolated percentile from a PRE-SORTED slice of Fixed64.
/// `pct` is between 0 and 100 inclusive.
pub fn percentile(sorted: &[Fixed64], pct: u32) -> Fixed64 {
    if sorted.is_empty() {
        return Fixed64::ZERO;
    }
    if sorted.len() == 1 || pct == 0 {
        return sorted[0];
    }
    if pct >= 100 {
        return *sorted.last().unwrap();
    }

    let n = sorted.len();
    let rank = (pct as u64).saturating_mul((n - 1) as u64);
    let index = (rank / 100) as usize;
    let rem = (rank % 100) as i32;

    if rem == 0 || index + 1 >= n {
        sorted[index]
    } else {
        let low = sorted[index];
        let high = sorted[index + 1];
        let diff = high - low;
        low + (diff * Fixed64::from_ratio(rem, 100))
    }
}

/// Calculate a full summary of statistics for a slice of values.
pub fn summarize(values: &[Fixed64]) -> Option<SeriesSummary> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort();

    let count = sorted.len();
    let min = sorted[0];
    let max = *sorted.last().unwrap();
    let p10 = percentile(&sorted, 10);
    let p25 = percentile(&sorted, 25);
    let median = percentile(&sorted, 50);
    let p75 = percentile(&sorted, 75);
    let p90 = percentile(&sorted, 90);
    let m = mean(&sorted);
    let var = variance(&sorted);
    let sd = var.sqrt();

    Some(SeriesSummary {
        count,
        min,
        p10,
        p25,
        median,
        p75,
        p90,
        max,
        mean: m,
        variance: var,
        std_dev: sd,
    })
}

/// Compute the population covariance of two paired slices.
pub fn covariance(x: &[Fixed64], y: &[Fixed64]) -> Fixed64 {
    let n = x.len().min(y.len());
    if n <= 1 {
        return Fixed64::ZERO;
    }
    let mean_x = mean(&x[..n]);
    let mean_y = mean(&y[..n]);

    let mut sum_prod: i128 = 0;
    for i in 0..n {
        let diff_x = x[i] - mean_x;
        let diff_y = y[i] - mean_y;
        let prod = (diff_x.raw() as i128 * diff_y.raw() as i128) >> 32;
        sum_prod += prod;
    }
    let cov = sum_prod / n as i128;
    Fixed64::from_raw(clamp_i128(cov))
}

/// Compute the Pearson correlation coefficient between two series in [-1.0, 1.0].
pub fn correlation(x: &[Fixed64], y: &[Fixed64]) -> Fixed64 {
    let n = x.len().min(y.len());
    if n <= 1 {
        return Fixed64::ZERO;
    }
    let std_x = std_dev(&x[..n]);
    let std_y = std_dev(&y[..n]);
    if std_x.is_zero() || std_y.is_zero() {
        return Fixed64::ZERO;
    }
    let cov = covariance(&x[..n], &y[..n]);
    let denom = std_x * std_y;
    if denom.is_zero() {
        return Fixed64::ZERO;
    }
    let r = cov / denom;
    r.max(Fixed64::NEG_ONE).min(Fixed64::ONE)
}

const fn clamp_i128(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_and_variance() {
        let values = [
            Fixed64::from_int(2),
            Fixed64::from_int(4),
            Fixed64::from_int(4),
            Fixed64::from_int(4),
            Fixed64::from_int(5),
            Fixed64::from_int(5),
            Fixed64::from_int(7),
            Fixed64::from_int(9),
        ];
        // Mean = 40 / 8 = 5
        let m = mean(&values);
        assert_eq!(m, Fixed64::from_int(5));

        // Variance = ((9 + 1 + 1 + 1 + 0 + 0 + 4 + 16) = 32) / 8 = 4
        let v = variance(&values);
        assert_eq!(v, Fixed64::from_int(4));

        // Std dev = sqrt(4) = 2
        let sd = std_dev(&values);
        assert_eq!(sd, Fixed64::from_int(2));
    }

    #[test]
    fn test_percentiles() {
        let mut data: Vec<Fixed64> = (1..=100).map(Fixed64::from_int).collect();
        data.sort();

        let p10 = percentile(&data, 10);
        let p50 = percentile(&data, 50);
        let p90 = percentile(&data, 90);

        // Linear interpolation on 1..100: rank = pct * 99 / 100
        assert_eq!(p50.to_int(), 50);
        assert!(p10 >= Fixed64::from_int(10) && p10 <= Fixed64::from_int(11));
        assert!(p90 >= Fixed64::from_int(90) && p90 <= Fixed64::from_int(91));
    }

    #[test]
    fn test_correlation() {
        let x: Vec<Fixed64> = (1..=10).map(Fixed64::from_int).collect();
        let y: Vec<Fixed64> = (1..=10).map(|v| Fixed64::from_int(v * 2)).collect();

        let r = correlation(&x, &y);
        assert_eq!(r, Fixed64::ONE);

        let neg_y: Vec<Fixed64> = (1..=10).map(|v| Fixed64::from_int(100 - v)).collect();
        let r_neg = correlation(&x, &neg_y);
        assert_eq!(r_neg, Fixed64::NEG_ONE);
    }
}
