pub const SQRT2: f64 = 1.414213562373095;

#[inline]
pub fn bpm_to_milliseconds(bpm: f64, delimiter: f64) -> f64 {
    60000.0 / delimiter / bpm
}

#[inline]
pub fn milliseconds_to_bpm(ms: f64, delimiter: f64) -> f64 {
    60000.0 / (ms * delimiter)
}

#[inline]
pub fn logistic(x: f64, midpoint_offset: f64, multiplier: f64, max_value: f64) -> f64 {
    max_value / (1.0 + (multiplier * (midpoint_offset - x)).exp())
}

#[inline]
pub fn logistic_simple(exponent: f64, max_value: f64) -> f64 {
    max_value / (1.0 + exponent.exp())
}

#[inline]
pub fn norm(p: f64, values: &[f64]) -> f64 {
    let mut sum = 0.0;
    for &x in values {
        sum += x.powf(p);
    }
    sum.powf(1.0 / p)
}

#[inline]
pub fn bell_curve(x: f64, mean: f64, width: f64, multiplier: f64) -> f64 {
    multiplier * (std::f64::consts::E * (-((x - mean) / width).powi(2))).exp()
}

#[inline]
pub fn smoothstep_bell_curve(x: f64, mean: f64, width: f64) -> f64 {
    let shifted = x - mean;
    let dist = if shifted > 0.0 { width - shifted } else { width + shifted };
    smoothstep(dist, 0.0, width)
}

#[inline]
pub fn smoothstep_bell_curve_unit(x: f64) -> f64 {
    let mut val = 0.5 - (x - 0.5).abs();
    val = (val * 2.0).clamp(0.0, 1.0);
    val * val * (3.0 - 2.0 * val)
}

#[inline]
pub fn smoothstep(x: f64, start: f64, end: f64) -> f64 {
    let t = ((x - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn smootherstep(x: f64, start: f64, end: f64) -> f64 {
    let t = ((x - start) / (end - start)).clamp(0.0, 1.0);
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

#[inline]
pub fn reverse_lerp(x: f64, start: f64, end: f64) -> f64 {
    ((x - start) / (end - start)).clamp(0.0, 1.0)
}

#[inline]
pub fn erf(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { -1.0 };
    }
    if x.is_nan() {
        return f64::NAN;
    }
    let ax = x.abs();
    let num = 1.0 / (1.0 + 0.3275911 * ax);
    let num2 = num * (0.254829592 + num * (-0.284496736 + num * (1.421413741 + num * (-1.453152027 + num * 1.061405429))));
    let num3 = 1.0 - num2 * (-ax * ax).exp();
    if x >= 0.0 {
        num3
    } else {
        -num3
    }
}

#[inline]
pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

#[inline]
pub fn erf_inv(x: f64) -> f64 {
    if x <= -1.0 {
        return f64::NEG_INFINITY;
    }
    if x >= 1.0 {
        return f64::INFINITY;
    }
    if x == 0.0 {
        return 0.0;
    }
    let sign = if x > 0.0 { 1.0 } else { -1.0 };
    let ax = x.abs();
    let num2 = (1.0 - ax * ax).ln();
    let num3 = 4.330746750799873 + num2 / 2.0;
    let num4 = num2 / 0.147;
    let d = (num3 * num3 - num4).sqrt() - num3;
    let num5 = if ax >= 0.85 { ((ax - 0.85) / 0.293).powi(8) } else { 0.0 };
    sign * (d.sqrt() + num5)
}

#[inline]
pub fn erfc_inv(x: f64) -> f64 {
    erf_inv(1.0 - x)
}

pub fn calculate_deviation(
    relevant_great: f64,
    relevant_ok: f64,
    relevant_meh: f64,
    great_hit_window: f64,
    ok_hit_window: f64,
    meh_hit_window: f64,
) -> Option<f64> {
    let total = relevant_great + relevant_ok + relevant_meh;
    if total <= 0.0 {
        return None;
    }
    let num = (relevant_great + relevant_ok).max(1.0);
    let num2 = relevant_great / num;
    let term1 = (num * num2 + 2.7059472155252142) / (num + 5.4118944310504284);
    let term2 = 2.32634787404 / (num + 5.4118944310504284) * (num * num2 * (1.0 - num2) + 1.3529736077626071).sqrt();
    let num3 = num2.min(term1 - term2);

    let num4 = if num3 > 0.01 {
        let mut dev = great_hit_window / (1.414213562373095 * erf_inv(num3));
        let num5 = (2.0 / std::f64::consts::PI).sqrt() * ok_hit_window * (-0.5 * (ok_hit_window / dev).powi(2)).exp()
            / (dev * erf(ok_hit_window / (1.414213562373095 * dev)));
        dev *= (1.0 - num5).max(0.0).sqrt();
        dev
    } else {
        ok_hit_window / 3.0_f64.sqrt()
    };

    let num6 = (meh_hit_window * meh_hit_window + ok_hit_window * meh_hit_window + ok_hit_window * ok_hit_window) / 3.0;
    Some((((relevant_great + relevant_ok) * num4.powi(2) + relevant_meh * num6) / total).sqrt())
}

