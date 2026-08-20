use crate::math::*;
use crate::lazer_engine::*;

#[derive(Clone, Debug)]
pub struct StrainPeak {
    pub value: f64,
    pub section_length: f64,
}

impl StrainPeak {
    pub fn new(value: f64, section_length: f64) -> Self {
        Self {
            value,
            section_length: section_length.round(),
        }
    }
}

pub struct LazerAimSkill {
    pub include_sliders: bool,
    pub current_strain: f64,
    pub current_section_peak: f64,
    pub current_section_begin: f64,
    pub current_section_end: f64,
    pub strain_peaks: Vec<StrainPeak>,
    pub queued_strains: Vec<(f64, f64)>,
    pub total_length: f64,
    pub max_stored_length: f64,
    pub max_section_length: f64,
    pub decay_weight: f64,
    pub slider_strains: Vec<f64>,
    pub difficulties: Vec<f64>,
    pub peaks_finalised: bool,
}

impl LazerAimSkill {
    pub fn new(include_sliders: bool) -> Self {
        let decay_weight = 0.9;
        let max_section_length = 400.0;
        let max_stored_length = 11.0 / (1.0 - decay_weight);
        Self {
            include_sliders,
            current_strain: 0.0,
            current_section_peak: 0.0,
            current_section_begin: 0.0,
            current_section_end: 0.0,
            strain_peaks: Vec::new(),
            queued_strains: Vec::new(),
            total_length: 0.0,
            max_stored_length,
            max_section_length,
            decay_weight,
            slider_strains: Vec::new(),
            difficulties: Vec::new(),
            peaks_finalised: false,
        }
    }

    #[inline]
    fn strain_decay(ms: f64) -> f64 {
        0.2_f64.powf(ms / 1000.0)
    }

    #[inline]
    fn calculate_initial_strain(&self, time: f64, prev_time: f64) -> f64 {
        self.current_strain * Self::strain_decay(time - prev_time)
    }

    fn save_current_peak(&mut self, section_length: f64) {
        if section_length <= 0.0 {
            return;
        }
        let peak = StrainPeak::new(self.current_section_peak, section_length);
        self.strain_peaks.push(peak);
        self.strain_peaks.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));
        self.total_length += section_length;

        while self.total_length > self.max_stored_length * self.max_section_length {
            if let Some(removed) = self.strain_peaks.pop() {
                self.total_length -= removed.section_length;
            } else {
                break;
            }
        }
    }

    fn backfill_peaks(&mut self, current_time: f64, prev_time: f64) {
        while current_time > self.current_section_end {
            let section_len = self.current_section_end - self.current_section_begin;
            self.save_current_peak(section_len);
            self.current_section_begin = self.current_section_end;

            if !self.queued_strains.is_empty() {
                let (val, start_time) = self.queued_strains.remove(0);
                self.current_section_end = start_time + self.max_section_length;
                self.current_section_peak = self.calculate_initial_strain(self.current_section_begin, prev_time).max(val);
            } else {
                self.current_section_end = self.current_section_begin + self.max_section_length;
                self.current_section_peak = self.calculate_initial_strain(self.current_section_begin, prev_time);
            }
        }
    }

    pub fn process_objects(&mut self, objects: &[LazerDifficultyHitObject]) {
        for (i, obj) in objects.iter().enumerate() {
            if i == 0 {
                let decay = Self::strain_decay(obj.adjusted_delta_time);
                self.current_strain *= decay;

                let snap_diff = SnapAimEvaluator::evaluate_difficulty_of(objects, 0, self.include_sliders) * 70.9;
                let agility_diff = AgilityEvaluator::evaluate_difficulty_of(objects, 0) * 2.35;
                let flow_diff = FlowAimEvaluator::evaluate_difficulty_of(objects, 0, self.include_sliders) * 242.0;

                let snap_agil = norm(1.2, &[snap_diff, agility_diff]);
                let ratio = if snap_agil == 0.0 { 0.0 } else { flow_diff / snap_agil };
                let snap_flow_prob = if ratio == 0.0 {
                    0.0
                } else if ratio.is_nan() {
                    1.0
                } else {
                    logistic_simple(-7.27 * ratio.ln(), 1.0)
                };
                let mut total_aim = (snap_agil * snap_flow_prob + flow_diff * (1.0 - snap_flow_prob)) * 1.12;

                let od_term = (obj.overall_difficulty.max(0.0)).powi(2) / 4000.0;
                total_aim *= 0.985 + od_term;

                self.current_strain += total_aim * (1.0 - decay);

                if obj.is_slider {
                    self.slider_strains.push(self.current_strain);
                }
                self.difficulties.push(self.current_strain);

                self.current_section_begin = obj.start_time;
                self.current_section_end = self.current_section_begin + self.max_section_length;
                self.current_section_peak = self.current_strain;
                continue;
            }

            let prev_time = objects[i - 1].start_time;
            self.backfill_peaks(obj.start_time, prev_time);

            let decay = Self::strain_decay(obj.adjusted_delta_time);
            self.current_strain *= decay;

            let snap_diff = SnapAimEvaluator::evaluate_difficulty_of(objects, i, self.include_sliders) * 70.9;
            let agility_diff = AgilityEvaluator::evaluate_difficulty_of(objects, i) * 2.35;
            let flow_diff = FlowAimEvaluator::evaluate_difficulty_of(objects, i, self.include_sliders) * 242.0;

            let snap_agil = norm(1.2, &[snap_diff, agility_diff]);
            let ratio = if snap_agil == 0.0 { 0.0 } else { flow_diff / snap_agil };
            let snap_flow_prob = if ratio == 0.0 {
                0.0
            } else if ratio.is_nan() {
                1.0
            } else {
                logistic_simple(-7.27 * ratio.ln(), 1.0)
            };
            let mut total_aim = (snap_agil * snap_flow_prob + flow_diff * (1.0 - snap_flow_prob)) * 1.12;

            let od_term = (obj.overall_difficulty.max(0.0)).powi(2) / 4000.0;
            total_aim *= 0.985 + od_term;

            self.current_strain += total_aim * (1.0 - decay);

            if obj.is_slider {
                self.slider_strains.push(self.current_strain);
            }
            self.difficulties.push(self.current_strain);

            let strain_val = self.current_strain;

            if strain_val > self.current_section_peak {
                self.queued_strains.clear();
                let section_len = obj.start_time - self.current_section_begin;
                self.save_current_peak(section_len);
                self.current_section_begin = obj.start_time;
                self.current_section_end = self.current_section_begin + self.max_section_length;
                self.current_section_peak = strain_val;
            } else {
                while let Some(&(last_strain, _)) = self.queued_strains.last() {
                    if last_strain < strain_val {
                        self.queued_strains.pop();
                    } else {
                        break;
                    }
                }
                self.queued_strains.push((strain_val, obj.start_time));
            }
        }
    }

    fn get_current_strain_peaks(&mut self) -> Vec<StrainPeak> {
        if !self.peaks_finalised {
            let section_len = self.current_section_end - self.current_section_begin;
            self.save_current_peak(section_len);
            self.peaks_finalised = true;
        }
        self.strain_peaks.clone()
    }

    fn get_reduced_strain_peaks(&mut self) -> Vec<StrainPeak> {
        let mut list: Vec<StrainPeak> = self.get_current_strain_peaks().into_iter().filter(|p| p.value > 0.0).collect();
        let reduced_section_time = 4000.0;
        let mut num = 0.0;
        let mut i = 0;

        while i < list.len() && num < reduced_section_time {
            let peak = list[i].clone();
            let mut num2 = 0.0;
            while num2 < peak.section_length {
                let progress = ((num + num2) / reduced_section_time).clamp(0.0, 1.0);
                let lerp_target = 1.0 + 9.0 * progress;
                let amount = lerp_target.log10();
                let scale = 0.727 + (1.0 - 0.727) * amount;
                let piece_len = (peak.section_length - num2).min(20.0);
                list.push(StrainPeak::new(peak.value * scale, piece_len));
                num2 += 20.0;
            }
            num += peak.section_length;
            i += 1;
        }

        let mut remaining: Vec<StrainPeak> = list.into_iter().skip(i).collect();
        remaining.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));
        remaining
    }

    pub fn difficulty_value(&mut self) -> f64 {
        let peaks = self.get_reduced_strain_peaks();
        let mut num = 0.0;
        let mut num2 = 0.0;
        for p in peaks {
            let exponent = num2;
            let num3 = num2 + p.section_length / self.max_section_length;
            let num4 = self.decay_weight.powf(exponent) - self.decay_weight.powf(num3);
            num += p.value * num4;
            num2 = num3;
        }
        num / (1.0 - self.decay_weight)
    }

    pub fn calculate_rating(&mut self) -> f64 {
        let val = self.difficulty_value();
        val.powf(0.63) * 0.02275
    }

    pub fn count_top_weighted_strains(&self, diff_val: f64) -> f64 {
        if self.difficulties.is_empty() {
            return 0.0;
        }
        let consistent_top = diff_val * (1.0 - self.decay_weight);
        if consistent_top == 0.0 {
            return self.difficulties.len() as f64;
        }
        self.difficulties.iter().map(|&s| 1.1 / (1.0 + (-10.0 * (s / consistent_top - 0.88)).exp())).sum()
    }

    pub fn count_top_weighted_sliders(&self, diff_val: f64) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }
        let consistent_top = diff_val * (1.0 - self.decay_weight);
        if consistent_top == 0.0 {
            return 0.0;
        }
        self.slider_strains.iter().map(|&s| logistic(s / consistent_top, 0.88, 10.0, 1.1)).sum()
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }
        let max_slider_strain = self.slider_strains.iter().cloned().fold(0.0_f64, f64::max);
        if max_slider_strain == 0.0 {
            return 0.0;
        }
        self.slider_strains.iter().map(|&s| 1.0 / (1.0 + (-(s / max_slider_strain * 12.0 - 6.0)).exp())).sum()
    }
}

pub struct LazerSpeedSkill {
    pub current_strain: f64,
    pub slider_strains: Vec<f64>,
    pub difficulties: Vec<f64>,
    pub object_weight_sum: f64,
}

impl LazerSpeedSkill {
    pub fn new() -> Self {
        Self {
            current_strain: 0.0,
            slider_strains: Vec::new(),
            difficulties: Vec::new(),
            object_weight_sum: 0.0,
        }
    }

    #[inline]
    fn strain_decay(ms: f64) -> f64 {
        0.3_f64.powf(ms / 1000.0)
    }

    pub fn process_objects(&mut self, objects: &[LazerDifficultyHitObject]) {
        for (i, obj) in objects.iter().enumerate() {
            let decay = Self::strain_decay(obj.adjusted_delta_time);
            self.current_strain *= decay;

            let raw_speed = SpeedEvaluator::evaluate_difficulty_of(objects, i);
            self.current_strain += raw_speed * (1.0 - decay) * 1.16;

            let rhythm = RhythmEvaluator::evaluate_difficulty_of(objects, i);
            let combined = self.current_strain * rhythm;

            if obj.is_slider {
                self.slider_strains.push(combined);
            }
            self.difficulties.push(combined);
        }
    }

    pub fn difficulty_value(&mut self) -> f64 {
        if self.difficulties.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.difficulties.iter().copied().filter(|&v| v > 0.0).collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let mut sum = 0.0;
        self.object_weight_sum = 0.0;
        for (i, &item) in sorted.iter().enumerate() {
            let scale_term = 1.0 + 20.0 / (1.0 + i as f64);
            let weight = scale_term / ((i as f64).powf(0.9) + scale_term);
            self.object_weight_sum += weight;
            sum += item * weight;
        }
        sum
    }

    pub fn calculate_rating(&mut self) -> f64 {
        let val = self.difficulty_value();
        val.sqrt() * 0.0675
    }

    pub fn relevant_object_count(&self) -> f64 {
        if self.difficulties.is_empty() {
            return 0.0;
        }
        let max_strain = self.difficulties.iter().cloned().fold(0.0_f64, f64::max);
        if max_strain == 0.0 {
            return 0.0;
        }
        self.difficulties.iter().map(|&s| 1.0 / (1.0 + (-(s / max_strain * 12.0 - 6.0)).exp())).sum()
    }

    pub fn count_top_weighted_object_difficulties(&self, diff_val: f64) -> f64 {
        if self.difficulties.is_empty() || self.object_weight_sum == 0.0 {
            return 0.0;
        }
        let consistent_top = diff_val / self.object_weight_sum;
        if consistent_top == 0.0 {
            return 0.0;
        }
        self.difficulties.iter().map(|&d| logistic(d / consistent_top, 0.88, 10.0, 1.1)).sum()
    }

    pub fn count_top_weighted_sliders(&self, diff_val: f64) -> f64 {
        if self.slider_strains.is_empty() || self.object_weight_sum == 0.0 {
            return 0.0;
        }
        let consistent_top = diff_val / self.object_weight_sum;
        if consistent_top == 0.0 {
            return 0.0;
        }
        self.slider_strains.iter().map(|&s| logistic(s / consistent_top, 0.88, 10.0, 1.1)).sum()
    }
}

pub struct LazerReadingSkill {
    pub has_hidden: bool,
    pub current_strain: f64,
    pub object_list_start_times: Vec<f64>,
    pub difficulties: Vec<f64>,
    pub object_weight_sum: f64,
}

impl LazerReadingSkill {
    pub fn new(has_hidden: bool) -> Self {
        Self {
            has_hidden,
            current_strain: 0.0,
            object_list_start_times: Vec::new(),
            difficulties: Vec::new(),
            object_weight_sum: 0.0,
        }
    }

    #[inline]
    fn strain_decay(ms: f64) -> f64 {
        0.8_f64.powf(ms / 1000.0)
    }

    pub fn process_objects(&mut self, objects: &[LazerDifficultyHitObject]) {
        for (i, obj) in objects.iter().enumerate() {
            self.object_list_start_times.push(obj.start_time);
            let decay = Self::strain_decay(obj.delta_time);
            self.current_strain *= decay;

            let mut raw_reading = ReadingEvaluator::evaluate_difficulty_of(objects, i, self.has_hidden);
            raw_reading *= 0.825 + (obj.overall_difficulty.max(0.0)).powf(2.2) / 1125.0;

            self.current_strain += raw_reading * (1.0 - decay) * 2.5;
            self.difficulties.push(self.current_strain);
        }
    }

    fn calculate_reduced_note_count(&self) -> usize {
        if self.object_list_start_times.is_empty() {
            return 0;
        }
        let threshold = self.object_list_start_times[0] + 60000.0;
        self.object_list_start_times.iter().take_while(|&&t| t <= threshold).count()
    }

    fn get_transformed_difficulties(&self) -> Vec<f64> {
        let mut diffs: Vec<f64> = self.difficulties.iter().copied().filter(|&v| v > 0.0).collect();
        let num = self.calculate_reduced_note_count();
        if num > 0 {
            for i in 0..diffs.len().min(num) {
                let progress = (i as f64 / num as f64).clamp(0.0, 1.0);
                let lerp_target = 1.0 + 9.0 * progress;
                let amount = lerp_target.log10();
                diffs[i] *= amount;
            }
        }
        diffs.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        diffs
    }

    pub fn difficulty_value(&mut self) -> f64 {
        if self.difficulties.is_empty() {
            return 0.0;
        }
        let sorted = self.get_transformed_difficulties();

        let mut sum = 0.0;
        self.object_weight_sum = 0.0;
        for (i, &item) in sorted.iter().enumerate() {
            let scale_term = 1.0 + 1.0 / (1.0 + i as f64);
            let weight = scale_term / ((i as f64).powf(0.9) + scale_term);
            self.object_weight_sum += weight;
            sum += item * weight;
        }
        sum
    }

    pub fn calculate_rating(&mut self) -> f64 {
        let val = self.difficulty_value();
        val.sqrt() * 0.0675
    }

    pub fn count_top_weighted_object_difficulties(&self, diff_val: f64) -> f64 {
        if self.difficulties.is_empty() || self.object_weight_sum == 0.0 {
            return 0.0;
        }
        let consistent_top = diff_val / self.object_weight_sum;
        if consistent_top == 0.0 {
            return 0.0;
        }
        self.difficulties.iter().map(|&d| logistic(d / consistent_top, 1.15, 5.0, 1.1)).sum()
    }
}

pub struct LazerFlashlightSkill {
    pub current_strain: f64,
    pub strain_peaks: Vec<f64>,
    pub current_section_peak: f64,
    pub current_section_end: f64,
    pub total_objects: usize,
    pub has_hidden: bool,
}

impl LazerFlashlightSkill {
    pub fn new(total_objects: usize, has_hidden: bool) -> Self {
        Self {
            current_strain: 0.0,
            strain_peaks: Vec::new(),
            current_section_peak: 0.0,
            current_section_end: 400.0,
            total_objects,
            has_hidden,
        }
    }

    #[inline]
    fn strain_decay(ms: f64) -> f64 {
        0.15_f64.powf(ms / 1000.0)
    }

    pub fn process_objects(&mut self, objects: &[LazerDifficultyHitObject]) {
        for (i, obj) in objects.iter().enumerate() {
            if i == 0 {
                self.current_section_end = (obj.start_time / 400.0).ceil() * 400.0;
            }

            while obj.start_time > self.current_section_end {
                self.strain_peaks.push(self.current_section_peak);
                self.current_section_end += 400.0;
                self.current_section_peak = self.current_strain * Self::strain_decay(self.current_section_end - obj.start_time);
            }

            let decay = Self::strain_decay(obj.delta_time);
            self.current_strain *= decay;

            let mut raw_fl = FlashlightEvaluator::evaluate_difficulty_of(objects, i, self.has_hidden);
            raw_fl *= 0.985 + (obj.overall_difficulty.max(0.0)).powi(2) / 4000.0;

            self.current_strain += raw_fl * 0.058;
            self.current_section_peak = self.current_section_peak.max(self.current_strain);
        }
        self.strain_peaks.push(self.current_section_peak);
    }

    pub fn difficulty_value(&self) -> f64 {
        let sum: f64 = self.strain_peaks.iter().sum();
        let total = self.total_objects as f64;
        let scale = 0.7 + 0.1 * (total / 200.0).min(1.0)
            + if total > 200.0 { 0.2 * ((total - 200.0) / 200.0).min(1.0) } else { 0.0 };
        sum * scale
    }

    pub fn calculate_rating(&self) -> f64 {
        let val = self.difficulty_value();
        val.sqrt() * 0.0675
    }
}

pub fn sum_cognition_difficulty(reading: f64, flashlight: f64) -> f64 {
    if reading <= 0.0 {
        flashlight
    } else if flashlight <= 0.0 {
        reading
    } else {
        norm(1.1, &[reading, flashlight * (flashlight / reading).clamp(0.25, 1.0)])
    }
}

pub fn calculate_star_rating(base_performance: f64) -> f64 {
    (base_performance * 1.12).cbrt()
}

pub struct LazerDifficultyResult {
    pub star_rating: f64,
    pub aim_difficulty: f64,
    pub speed_difficulty: f64,
    pub reading_difficulty: f64,
    pub flashlight_difficulty: f64,
    pub slider_factor: f64,
    pub aim_difficult_strain_count: f64,
    pub speed_difficult_strain_count: f64,
    pub reading_difficult_note_count: f64,
    pub speed_note_count: f64,
    pub aim_top_weighted_slider_factor: f64,
    pub speed_top_weighted_slider_factor: f64,
    pub aim_difficult_slider_count: f64,
    pub max_combo: u32,
    pub hit_circle_count: u32,
    pub slider_count: u32,
    pub spinner_count: u32,
}

pub struct LazerPerformanceResult {
    pub pp: f64,
    pub aim_pp: f64,
    pub speed_pp: f64,
    pub accuracy_pp: f64,
    pub reading_pp: f64,
    pub flashlight_pp: f64,
    pub speed_deviation: Option<f64>,
}

pub fn calculate_lazer_performance(
    diff: &LazerDifficultyResult,
    accuracy: f64,
    combo: Option<u32>,
    misses: u32,
    od: f64,
    ar: f64,
    clock_rate: f64,
    _has_hidden: bool,
    has_flashlight: bool,
) -> LazerPerformanceResult {
    let total_hits = (diff.hit_circle_count + diff.slider_count + diff.spinner_count) as f64;
    let max_combo = diff.max_combo as f64;
    let score_max_combo = combo.unwrap_or(diff.max_combo) as f64;
    let effective_misses = misses as f64;
    let acc = (accuracy / 100.0).clamp(0.0, 1.0);

    let (count_great, count_ok, count_meh) = {
        let total_result_count = total_hits;
        let relevant_result_count = (total_result_count - effective_misses).max(1.0);
        let relevant_accuracy = (acc * total_result_count / relevant_result_count).clamp(0.0, 1.0);

        if relevant_accuracy >= 0.25 {
            let ratio50_to_100 = (1.0 - (relevant_accuracy - 0.25) / 0.75).powi(2);
            let count100_est = 6.0 * relevant_result_count * (1.0 - relevant_accuracy) / (5.0 * ratio50_to_100 + 4.0);
            let count50_est = count100_est * ratio50_to_100;
            let ok = count100_est.round();
            let meh = (count100_est + count50_est).round() - ok;
            let great = (total_result_count - ok - meh - effective_misses).max(0.0);
            (great, ok, meh)
        } else if relevant_accuracy >= 1.0 / 6.0 {
            let count100_est = 6.0 * relevant_result_count * relevant_accuracy - relevant_result_count;
            let count50_est = relevant_result_count - count100_est;
            let ok = count100_est.round();
            let meh = (count100_est + count50_est).round() - ok;
            let great = (total_result_count - ok - meh - effective_misses).max(0.0);
            (great, ok, meh)
        } else {
            let count50_est = 6.0 * relevant_result_count * relevant_accuracy;
            let meh = count50_est.round();
            let ok = 0.0;
            let great = 0.0;
            (great, ok, meh)
        }
    };

    let great_window = (79.5 - 6.0 * od) / clock_rate;
    let ok_window = (139.5 - 8.0 * od) / clock_rate;
    let meh_window = (199.5 - 10.0 * od) / clock_rate;
    let overall_difficulty = (79.5 - great_window) / 6.0;

    // 1. Aim PP
    let mut aim_diff = diff.aim_difficulty;
    let slider_count = diff.slider_count as f64;
    let dropped_sliders: f64 = 0.0;
    if slider_count > 0.0 && diff.aim_difficult_slider_count > 0.0 {
        let num2 = dropped_sliders.clamp(0.0, diff.aim_difficult_slider_count);
        let num3 = (1.0 - diff.slider_factor) * (1.0_f64 - num2 / diff.aim_difficult_slider_count).powi(3) + diff.slider_factor;
        aim_diff *= num3;
    }
    let mut aim_pp = 4.0 * aim_diff.powi(3);
    let length_bonus = 0.95 + 0.35 * (total_hits / 2000.0).min(1.0)
        + if total_hits > 2000.0 { (total_hits / 2000.0).log10() * 0.5 } else { 0.0 };
    aim_pp *= length_bonus;
    if effective_misses > 0.0 {
        let count = diff.aim_difficult_strain_count.max(1.0);
        aim_pp *= 0.93 / (effective_misses / (4.0 * count.ln().max(1.0)) + 1.0);
    }
    aim_pp *= acc;

    // 2. Speed PP & Deviation
    let mut speed_pp = 4.0 * diff.speed_difficulty.powi(3);
    let speed_note_count = diff.speed_note_count + (total_hits - diff.speed_note_count) * 0.1;
    let s_miss = effective_misses.min(speed_note_count);
    let s_meh = count_meh.min((speed_note_count - s_miss).max(0.0));
    let s_ok = count_ok.min((speed_note_count - s_miss - s_meh).max(0.0));
    let s_great = (speed_note_count - s_miss - s_meh - s_ok).max(0.0);

    let speed_deviation = calculate_deviation(s_great, s_ok, s_meh, great_window, ok_window, meh_window);
    if let Some(dev) = speed_deviation {
        let num2_nerf = 100.0 + 220.0 * (22.0 / dev).powf(6.5);
        let speed_high_nerf = if speed_pp <= num2_nerf {
            1.0
        } else {
            let val = 50.0 * (((speed_pp - num2_nerf) / 50.0 + 1.0).ln() + num2_nerf / 50.0);
            let amount = 1.0 - reverse_lerp(dev, 22.0, 27.0);
            (val + (speed_pp - val) * amount) / speed_pp
        };
        speed_pp *= speed_high_nerf;

        let erf_val = erf(20.0 * (4.0 / diff.speed_difficulty.max(0.1)).powf(0.35) / dev);
        speed_pp *= erf_val * erf_val;
    }
    if effective_misses > 0.0 {
        let count = diff.speed_difficult_strain_count.max(1.0);
        speed_pp *= 0.93 / (effective_misses / (4.0 * count.ln().max(1.0)) + 1.0);
    }

    // 3. Accuracy PP
    let num_acc_objects = (diff.hit_circle_count + diff.slider_count) as f64;
    let circle_acc = if num_acc_objects <= 0.0 {
        0.0
    } else {
        let max_sub = (total_hits - num_acc_objects).max(0.0);
        let adjusted_great = (count_great - max_sub).max(0.0);
        ((adjusted_great * 6.0 + count_ok * 2.0 + count_meh) / (num_acc_objects * 6.0)).clamp(0.0, 1.0)
    };

    let mut accuracy_pp = 1.52163_f64.powf(overall_difficulty) * circle_acc.powi(24) * 2.83;
    let circle_bonus = if num_acc_objects < 1000.0 {
        (num_acc_objects / 1000.0).powf(0.3)
    } else {
        (num_acc_objects / 1000.0).powf(0.1)
    };
    accuracy_pp *= circle_bonus;

    // 4. Reading PP
    let mut reading_pp = 4.0 * diff.reading_difficulty.powi(3);
    if effective_misses > 0.0 {
        let count = diff.reading_difficult_note_count.max(1.0);
        reading_pp *= 0.93 / (effective_misses / (4.0 * count.ln().max(1.0)) + 1.0);
    }
    reading_pp *= acc.powi(3);

    // 5. Flashlight PP
    let mut flashlight_pp = if has_flashlight { 25.0 * diff.flashlight_difficulty.powi(2) } else { 0.0 };
    if has_flashlight && effective_misses > 0.0 {
        flashlight_pp *= 0.97 * (1.0 - (effective_misses / total_hits).powf(0.775)).powf(effective_misses.powf(0.875));
    }
    flashlight_pp *= (score_max_combo / max_combo).powf(0.8).min(1.0);
    flashlight_pp *= 0.5 + acc / 2.0;

    let cognition = sum_cognition_difficulty(reading_pp, flashlight_pp);
    let total = norm(1.1, &[aim_pp, speed_pp, accuracy_pp, cognition]) * 1.12;

    LazerPerformanceResult {
        pp: total,
        aim_pp,
        speed_pp,
        accuracy_pp,
        reading_pp,
        flashlight_pp,
        speed_deviation,
    }
}
