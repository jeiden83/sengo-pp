use crate::math::*;
use rosu_pp::Beatmap;
use rosu_pp::model::hit_object::HitObjectKind;
use rosu_map::section::hit_objects::PathType;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct LazerDifficultyHitObject {
    pub index: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub delta_time: f64,
    pub adjusted_delta_time: f64,
    pub last_object_end_delta_time: f64,
    pub jump_distance: f64,
    pub lazy_jump_distance: f64,
    pub min_jump_distance: f64,
    pub min_jump_time: f64,
    pub travel_distance: f64,
    pub travel_time: f64,
    pub lazy_travel_distance: f64,
    pub lazy_travel_time: f64,
    pub angle: Option<f64>,
    pub normalised_vector_angle: Option<f64>,
    pub small_circle_bonus: f64,
    pub is_slider: bool,
    pub is_spinner: bool,
    pub hit_window_great: f64,
    pub overall_difficulty: f64,
    pub pos: (f64, f64),
    pub head_pos: (f64, f64),
    pub tail_pos: (f64, f64),
    pub lazy_end_pos: Option<(f64, f64)>,
    pub radius: f64,
    pub base_start_time: f64,
    pub preempt: f64,
    pub base_preempt: f64,
    pub base_fade_in: f64,
}

impl LazerDifficultyHitObject {
    #[inline]
    pub fn calculate_double_tap_feasibility(&self, next_obj: Option<&LazerDifficultyHitObject>) -> f64 {
        if let Some(next) = next_obj {
            let dt = self.delta_time.max(1.0);
            let next_dt = next.delta_time.max(1.0);
            let diff = (next_dt - dt).abs();
            let x = dt / dt.max(diff);
            let num2 = (dt / self.hit_window_great).min(1.0).powi(5);
            let num3 = reverse_lerp(self.lazy_jump_distance, 100.0, 50.0).powi(2);
            1.0 - x.powf(num3 * (1.0 - num2))
        } else {
            0.0
        }
    }

    #[inline]
    pub fn opacity_at(&self, time: f64, hidden: bool) -> f64 {
        if time > self.base_start_time {
            return 0.0;
        }
        let num = self.base_start_time - self.base_preempt;
        let num2 = 400.0 * (self.base_preempt / 450.0).min(1.0);
        if hidden {
            let num3 = self.base_start_time - self.base_preempt + self.base_fade_in;
            let num4 = self.base_preempt * 0.3;
            let term1 = ((time - num) / num2).clamp(0.0, 1.0);
            let term2 = 1.0 - ((time - num3) / num4).clamp(0.0, 1.0);
            term1.min(term2)
        } else {
            ((time - num) / num2).clamp(0.0, 1.0)
        }
    }
}

pub struct SliderPath {
    pub calculated_path: Vec<(f64, f64)>,
    pub cumulative_lengths: Vec<f64>,
    pub total_length: f64,
}

impl SliderPath {
    pub fn new(control_points: &[rosu_map::section::hit_objects::PathControlPoint], expected_dist: Option<f64>) -> Self {
        if control_points.is_empty() {
            return Self {
                calculated_path: vec![(0.0, 0.0)],
                cumulative_lengths: vec![0.0],
                total_length: 0.0,
            };
        }

        let mut calculated_path = Vec::new();
        let mut i = 0;
        let mut last_type = PathType::LINEAR;

        while i < control_points.len() {
            let start = i;
            if let Some(kind) = control_points[start].path_type {
                last_type = kind;
            }
            i += 1;
            while i < control_points.len() && control_points[i].path_type.is_none() {
                i += 1;
            }
            let sub_points: Vec<(f64, f64)> = control_points[start..i]
                .iter()
                .map(|p| (p.pos.x as f64, p.pos.y as f64))
                .collect();

            if sub_points.is_empty() {
                continue;
            }

            let mut sub_path = match last_type {
                PathType::LINEAR => Self::create_linear(&sub_points),
                PathType::PERFECT_CURVE => {
                    if sub_points.len() == 3 {
                        Self::create_arc(&sub_points)
                    } else {
                        Self::create_bezier(&sub_points)
                    }
                }
                PathType::CATMULL => Self::create_catmull(&sub_points),
                _ => Self::create_bezier(&sub_points),
            };

            if calculated_path.is_empty() {
                calculated_path.append(&mut sub_path);
            } else if !sub_path.is_empty() {
                calculated_path.extend_from_slice(&sub_path[1..]);
            }
        }

        if calculated_path.is_empty() {
            calculated_path.push((control_points[0].pos.x as f64, control_points[0].pos.y as f64));
        }

        let mut cumulative_lengths = Vec::with_capacity(calculated_path.len());
        cumulative_lengths.push(0.0);
        let mut curr_len = 0.0;
        for j in 1..calculated_path.len() {
            let dx = calculated_path[j].0 - calculated_path[j - 1].0;
            let dy = calculated_path[j].1 - calculated_path[j - 1].1;
            curr_len += (dx * dx + dy * dy).sqrt();
            cumulative_lengths.push(curr_len);
        }

        let total_length = expected_dist.unwrap_or(curr_len);

        Self {
            calculated_path,
            cumulative_lengths,
            total_length,
        }
    }

    fn create_linear(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        points.to_vec()
    }

    fn create_arc(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        let (a, b, c) = (points[0], points[1], points[2]);
        let d = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
        if d.abs() < 1e-4 {
            return points.to_vec();
        }

        let a_sq = a.0 * a.0 + a.1 * a.1;
        let b_sq = b.0 * b.0 + b.1 * b.1;
        let c_sq = c.0 * c.0 + c.1 * c.1;

        let ox = (a_sq * (b.1 - c.1) + b_sq * (c.1 - a.1) + c_sq * (a.1 - b.1)) / d;
        let oy = (a_sq * (c.0 - b.0) + b_sq * (a.0 - c.0) + c_sq * (b.0 - a.0)) / d;
        let r = ((a.0 - ox).powi(2) + (a.1 - oy).powi(2)).sqrt();

        if r > 1000.0 || r.is_nan() || r.is_infinite() {
            return points.to_vec();
        }

        let a1 = (a.1 - oy).atan2(a.0 - ox);
        let a2 = (b.1 - oy).atan2(b.0 - ox);
        let a3 = (c.1 - oy).atan2(c.0 - ox);

        if a1.is_nan() || a2.is_nan() || a3.is_nan() {
            return points.to_vec();
        }

        let angle_diff = (a2 - a1).rem_euclid(2.0 * PI);
        let arc_diff = (a3 - a1).rem_euclid(2.0 * PI);
        let da = if angle_diff > arc_diff {
            arc_diff - 2.0 * PI
        } else {
            arc_diff
        };

        const CIRCULAR_ARC_TOLERANCE: f64 = 0.1;
        let d_theta = 2.0 * (1.0 - (CIRCULAR_ARC_TOLERANCE / r).min(1.0)).acos();
        let segments = if d_theta <= 0.0 { 1 } else { (da.abs() / d_theta).ceil().max(1.0) as usize };
        let mut path = Vec::with_capacity(segments + 1);

        for step in 0..=segments {
            let t = step as f64 / segments as f64;
            let ang = a1 + t * da;
            path.push((ox + r * ang.cos(), oy + r * ang.sin()));
        }

        path
    }

    fn bezier_is_flat_enough(control_points: &[(f64, f64)]) -> bool {
        const BEZIER_TOLERANCE_SQ: f64 = 0.25 * 0.25 * 4.0;
        for i in 1..control_points.len() - 1 {
            let dx = control_points[i - 1].0 - 2.0 * control_points[i].0 + control_points[i + 1].0;
            let dy = control_points[i - 1].1 - 2.0 * control_points[i].1 + control_points[i + 1].1;
            if dx * dx + dy * dy > BEZIER_TOLERANCE_SQ {
                return false;
            }
        }
        true
    }

    fn bezier_subdivide(
        control_points: &[(f64, f64)],
        l: &mut [(f64, f64)],
        r: &mut [(f64, f64)],
        subdivision_buffer: &mut [(f64, f64)],
        count: usize,
    ) {
        subdivision_buffer[..count].copy_from_slice(&control_points[..count]);

        for i in 0..count {
            l[i] = subdivision_buffer[0];
            r[count - i - 1] = subdivision_buffer[count - i - 1];

            for j in 0..count - i - 1 {
                subdivision_buffer[j].0 = (subdivision_buffer[j].0 + subdivision_buffer[j + 1].0) * 0.5;
                subdivision_buffer[j].1 = (subdivision_buffer[j].1 + subdivision_buffer[j + 1].1) * 0.5;
            }
        }
    }

    fn bezier_approximate(
        control_points: &[(f64, f64)],
        output: &mut Vec<(f64, f64)>,
        subdivision_buffer1: &mut [(f64, f64)],
        subdivision_buffer2: &mut [(f64, f64)],
        count: usize,
    ) {
        let mut l = subdivision_buffer2.to_vec();
        let mut r = subdivision_buffer1.to_vec();

        Self::bezier_subdivide(control_points, &mut l, &mut r, subdivision_buffer1, count);

        for i in 0..count - 1 {
            l[count + i] = r[i + 1];
        }

        output.push(control_points[0]);

        for i in 1..count - 1 {
            let index = 2 * i;
            let p = (
                0.25 * (l[index - 1].0 + 2.0 * l[index].0 + l[index + 1].0),
                0.25 * (l[index - 1].1 + 2.0 * l[index].1 + l[index + 1].1),
            );
            output.push(p);
        }
    }

    fn create_bezier(control_points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        if control_points.len() < 2 {
            return control_points.to_vec();
        }
        let degree = control_points.len() - 1;
        let mut output = Vec::new();
        let mut to_flatten = vec![control_points.to_vec()];
        let mut free_buffers: Vec<Vec<(f64, f64)>> = Vec::new();

        let mut sub_buf1 = vec![(0.0, 0.0); degree + 1];
        let mut sub_buf2 = vec![(0.0, 0.0); degree * 2 + 1];

        while let Some(mut parent) = to_flatten.pop() {
            if Self::bezier_is_flat_enough(&parent) {
                Self::bezier_approximate(&parent, &mut output, &mut sub_buf1, &mut sub_buf2, degree + 1);
                free_buffers.push(parent);
                continue;
            }

            let mut right_child = free_buffers.pop().unwrap_or_else(|| vec![(0.0, 0.0); degree + 1]);
            let mut left_child = vec![(0.0, 0.0); degree + 1];

            Self::bezier_subdivide(&parent, &mut left_child, &mut right_child, &mut sub_buf1, degree + 1);

            for i in 0..degree + 1 {
                parent[i] = left_child[i];
            }

            to_flatten.push(right_child);
            to_flatten.push(parent);
        }

        output.push(*control_points.last().unwrap());
        output
    }

    fn create_catmull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        if points.len() <= 1 {
            return points.to_vec();
        }
        let mut path = Vec::new();
        for i in 0..points.len() - 1 {
            let p0 = if i > 0 { points[i - 1] } else { points[i] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() { points[i + 2] } else { p2 };

            for step in 0..10 {
                let t = step as f64 / 10.0;
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5 * ((2.0 * p1.0) + (-p0.0 + p2.0) * t + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2 + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
                let y = 0.5 * ((2.0 * p1.1) + (-p0.1 + p2.1) * t + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2 + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
                path.push((x, y));
            }
        }
        path.push(*points.last().unwrap());
        path
    }

    pub fn position_at(&self, progress: f64) -> (f64, f64) {
        if self.calculated_path.is_empty() {
            return (0.0, 0.0);
        }
        let target_dist = (progress * self.total_length).clamp(0.0, self.total_length);
        match self.cumulative_lengths.binary_search_by(|len| len.partial_cmp(&target_dist).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(idx) => self.calculated_path[idx],
            Err(idx) => {
                if idx == 0 {
                    self.calculated_path[0]
                } else if idx >= self.calculated_path.len() {
                    *self.calculated_path.last().unwrap()
                } else {
                    let d0 = self.cumulative_lengths[idx - 1];
                    let d1 = self.cumulative_lengths[idx];
                    let t = if d1 > d0 { (target_dist - d0) / (d1 - d0) } else { 0.0 };
                    let p0 = self.calculated_path[idx - 1];
                    let p1 = self.calculated_path[idx];
                    (p0.0 + (p1.0 - p0.0) * t, p0.1 + (p1.1 - p0.1) * t)
                }
            }
        }
    }
}

pub enum NestedObjectKind {
    SliderHead,
    SliderTick,
    SliderRepeat,
    SliderTail,
}

pub struct NestedHitObject {
    pub start_time: f64,
    pub pos: (f64, f64),
    pub kind: NestedObjectKind,
}

pub fn build_lazer_difficulty_hit_objects(
    map: &Beatmap,
    ar: f64,
    cs: f64,
    od: f64,
    clock_rate: f64,
    has_hidden: bool,
) -> Vec<LazerDifficultyHitObject> {
    if map.hit_objects.is_empty() {
        return Vec::new();
    }

    let base_preempt = if ar < 5.0 {
        1800.0 - 120.0 * ar
    } else {
        1200.0 - 150.0 * (ar - 5.0)
    };
    let preempt = base_preempt / clock_rate;
    let base_fade_in = if has_hidden {
        base_preempt * 0.4
    } else {
        400.0 * (base_preempt / 450.0).min(1.0)
    };
    let great_window = (79.5 - 6.0 * od) / clock_rate;
    let overall_diff = (79.5 - great_window) / 6.0;
    let radius = (54.4 - 4.48 * cs).max(1.0);
    let small_circle_bonus = (1.0 + (30.0 - radius) / 70.0).max(1.0);

    let base_sv = map.slider_multiplier as f64;

    struct ProcessedBaseObject {
        start_time: f64,
        end_time: f64,
        pos: (f64, f64),
        stacked_pos: (f64, f64),
        tail_pos: (f64, f64),
        is_slider: bool,
        is_spinner: bool,
        nested_objects: Vec<NestedHitObject>,
        lazy_travel_time: f64,
        lazy_travel_distance: f64,
        travel_time: f64,
        travel_distance: f64,
        lazy_end_pos: Option<(f64, f64)>,
    }

    let mut base_objects = Vec::with_capacity(map.hit_objects.len());

    for h in map.hit_objects.iter() {
        match &h.kind {
            HitObjectKind::Circle => {
                base_objects.push(ProcessedBaseObject {
                    start_time: h.start_time,
                    end_time: h.start_time,
                    pos: (h.pos.x as f64, h.pos.y as f64),
                    stacked_pos: (h.pos.x as f64, h.pos.y as f64),
                    tail_pos: (h.pos.x as f64, h.pos.y as f64),
                    is_slider: false,
                    is_spinner: false,
                    nested_objects: Vec::new(),
                    lazy_travel_time: 0.0,
                    lazy_travel_distance: 0.0,
                    travel_time: 0.0,
                    travel_distance: 0.0,
                    lazy_end_pos: None,
                });
            }
            HitObjectKind::Slider(s) => {
                let pixel_length = s.expected_dist.unwrap_or(0.0) as f64;
                let repeat_count = s.repeats as u32;
                let span_count = repeat_count + 1;

                let beat_len = map.timing_points
                    .iter()
                    .rev()
                    .find(|tp| tp.time <= h.start_time)
                    .map(|tp| tp.beat_len)
                    .unwrap_or_else(|| map.timing_points.first().map(|tp| tp.beat_len).unwrap_or(500.0));

                let sv = map.difficulty_points
                    .iter()
                    .rev()
                    .find(|dp| dp.time <= h.start_time)
                    .map(|dp| dp.slider_velocity)
                    .unwrap_or(1.0);

                let pixels_per_beat = (base_sv * 100.0 * sv).max(1e-4);
                let duration = ((pixel_length * span_count as f64) / pixels_per_beat) * beat_len;
                let span_duration = duration / span_count as f64;

                let slider_path = SliderPath::new(&s.control_points, s.expected_dist.map(|d| d as f64));
                let head_pos = (h.pos.x as f64, h.pos.y as f64);
                let tail_pos_local = slider_path.position_at(if span_count % 2 == 1 { 1.0 } else { 0.0 });
                let tail_pos = (head_pos.0 + tail_pos_local.0 - slider_path.calculated_path[0].0, head_pos.1 + tail_pos_local.1 - slider_path.calculated_path[0].1);

                let mut nested_objects = Vec::new();
                nested_objects.push(NestedHitObject {
                    start_time: h.start_time,
                    pos: head_pos,
                    kind: NestedObjectKind::SliderHead,
                });

                let tick_interval = (pixels_per_beat / (map.slider_tick_rate as f64).max(1.0)).max(1.0);
                let min_dist_from_end = tick_interval / 8.0;
                if pixel_length > 0.0 && duration > 0.0 && span_duration > 0.0 {
                    for span_idx in 0..span_count {
                        let span_start = h.start_time + (span_idx as f64) * span_duration;
                        let is_reverse = span_idx % 2 == 1;

                        let mut d = tick_interval;
                        while d <= pixel_length - min_dist_from_end {
                            let progress = if is_reverse { 1.0 - d / pixel_length } else { d / pixel_length };
                            let tick_time = span_start + (d / pixel_length) * span_duration;
                            let p = slider_path.position_at(progress);
                            let tick_pos = (head_pos.0 + p.0 - slider_path.calculated_path[0].0, head_pos.1 + p.1 - slider_path.calculated_path[0].1);
                            nested_objects.push(NestedHitObject {
                                start_time: tick_time,
                                pos: tick_pos,
                                kind: NestedObjectKind::SliderTick,
                            });
                            d += tick_interval;
                        }

                        if span_idx < span_count - 1 {
                            let repeat_time = span_start + span_duration;
                            let progress = if is_reverse { 0.0 } else { 1.0 };
                            let p = slider_path.position_at(progress);
                            let repeat_pos = (head_pos.0 + p.0 - slider_path.calculated_path[0].0, head_pos.1 + p.1 - slider_path.calculated_path[0].1);
                            nested_objects.push(NestedHitObject {
                                start_time: repeat_time,
                                pos: repeat_pos,
                                kind: NestedObjectKind::SliderRepeat,
                            });
                        }
                    }
                }

                nested_objects.push(NestedHitObject {
                    start_time: h.start_time + duration,
                    pos: tail_pos,
                    kind: NestedObjectKind::SliderTail,
                });

                // computeSliderCursorPosition
                let mut num = (h.start_time + duration - 36.0).max(h.start_time + duration / 2.0);
                let last_tick_idx = nested_objects.iter().rposition(|n| matches!(n.kind, NestedObjectKind::SliderTick));
                if let Some(tick_idx) = last_tick_idx {
                    if nested_objects[tick_idx].start_time > num {
                        num = nested_objects[tick_idx].start_time;
                        let tick = nested_objects.remove(tick_idx);
                        nested_objects.push(tick);
                    }
                }

                let lazy_travel_time = num - h.start_time;
                let mut num2 = if span_duration > 0.0 { lazy_travel_time / span_duration } else { 0.0 };
                num2 = if num2 % 2.0 >= 1.0 { 1.0 - num2 % 1.0 } else { num2 % 1.0 };

                let lazy_end_pos_local = slider_path.position_at(num2);
                let mut lazy_end_pos = (head_pos.0 + lazy_end_pos_local.0 - slider_path.calculated_path[0].0, head_pos.1 + lazy_end_pos_local.1 - slider_path.calculated_path[0].1);

                let mut curr_cursor = head_pos;
                let scaling = (50.0 / radius) as f32;
                let mut lazy_travel_dist = 0.0f32;

                for (idx, nested) in nested_objects.iter().enumerate().skip(1) {
                    let mut val = nested.pos;
                    let dx = (val.0 - curr_cursor.0) as f32;
                    let dy = (val.1 - curr_cursor.1) as f32;
                    let raw_dist = (dx * dx + dy * dy).sqrt();
                    let mut num4 = raw_dist * scaling;
                    let mut num5 = 90.0f32;

                    if idx == nested_objects.len() - 1 {
                        let end_dx = (lazy_end_pos.0 - curr_cursor.0) as f32;
                        let end_dy = (lazy_end_pos.1 - curr_cursor.1) as f32;
                        let end_dist = (end_dx * end_dx + end_dy * end_dy).sqrt();
                        if end_dist < raw_dist {
                            val = lazy_end_pos;
                            num4 = end_dist * scaling;
                        }
                    } else if matches!(nested.kind, NestedObjectKind::SliderRepeat) {
                        num5 = 50.0f32;
                    }

                    if num4 > num5 {
                        let factor = (num4 - num5) / num4;
                        let vector_x = (val.0 - curr_cursor.0) as f32 * factor;
                        let vector_y = (val.1 - curr_cursor.1) as f32 * factor;
                        curr_cursor.0 += vector_x as f64;
                        curr_cursor.1 += vector_y as f64;
                        num4 -= num5;
                        lazy_travel_dist += num4;
                    }

                    if idx == nested_objects.len() - 1 {
                        lazy_end_pos = curr_cursor;
                    }
                }

                let travel_dist = ((lazy_travel_dist as f64 * (repeat_count as f64).powf(0.3).max(1.0)) as f32) as f64;
                let lazy_travel_dist = lazy_travel_dist as f64;
                let travel_t = (lazy_travel_time / clock_rate).max(25.0);

                base_objects.push(ProcessedBaseObject {
                    start_time: h.start_time,
                    end_time: h.start_time + duration,
                    pos: head_pos,
                    stacked_pos: head_pos,
                    tail_pos,
                    is_slider: true,
                    is_spinner: false,
                    nested_objects,
                    lazy_travel_time,
                    lazy_travel_distance: lazy_travel_dist,
                    travel_time: travel_t,
                    travel_distance: travel_dist,
                    lazy_end_pos: Some(lazy_end_pos),
                });
            }
            HitObjectKind::Spinner(sp) => {
                base_objects.push(ProcessedBaseObject {
                    start_time: h.start_time,
                    end_time: h.start_time + sp.duration as f64,
                    pos: (256.0, 192.0),
                    stacked_pos: (256.0, 192.0),
                    tail_pos: (256.0, 192.0),
                    is_slider: false,
                    is_spinner: true,
                    nested_objects: Vec::new(),
                    lazy_travel_time: 0.0,
                    lazy_travel_distance: 0.0,
                    travel_time: 0.0,
                    travel_distance: 0.0,
                    lazy_end_pos: None,
                });
            }
            _ => {}
        }
    }

    let count = base_objects.len();
    let mut stack_heights = vec![0_i32; count];
    let stack_threshold = (base_preempt as f32) * (map.stack_leniency as f32);

    #[inline]
    fn v2_dist(p1: (f64, f64), p2: (f64, f64)) -> f32 {
        let dx = (p1.0 - p2.0) as f32;
        let dy = (p1.1 - p2.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    if count > 0 {
        if map.version >= 6 {
            let start_index = 0;
            let end_index = count - 1;
            let num = end_index;

            let mut num5 = start_index;
            for num6 in (start_index + 1..=num).rev() {
                let mut num7 = num6;
                let mut obj3_idx = num6;
                if stack_heights[num6] == 0 && !base_objects[num6].is_spinner {
                    let num8 = stack_threshold;
                    if !base_objects[num6].is_slider {
                        // HitCircle
                        while num7 > 0 {
                            num7 -= 1;
                            if base_objects[num7].is_spinner {
                                continue;
                            }
                            let end_time2 = base_objects[num7].end_time;
                            if (base_objects[num6].start_time as i32 - end_time2 as i32) as f32 > num8 {
                                break;
                            }
                            if num7 < num5 {
                                stack_heights[num7] = 0;
                                num5 = num7;
                            }
                            if base_objects[num7].is_slider && v2_dist(base_objects[num7].tail_pos, base_objects[num6].pos) < 3.0 {
                                let num9 = stack_heights[num6] - stack_heights[num7] + 1;
                                for j in num7 + 1..=num6 {
                                    if v2_dist(base_objects[num7].tail_pos, base_objects[j].pos) < 3.0 {
                                        stack_heights[j] -= num9;
                                    }
                                }
                                break;
                            }
                            if v2_dist(base_objects[num7].pos, base_objects[obj3_idx].pos) < 3.0 {
                                stack_heights[num7] = stack_heights[obj3_idx] + 1;
                                obj3_idx = num7;
                            }
                        }
                    } else {
                        // Slider
                        while num7 > start_index {
                            num7 -= 1;
                            if !base_objects[num7].is_spinner {
                                if base_objects[num6].start_time - base_objects[num7].start_time > num8 as f64 {
                                    break;
                                }
                                if v2_dist(base_objects[num7].tail_pos, base_objects[obj3_idx].pos) < 3.0 {
                                    stack_heights[num7] = stack_heights[obj3_idx] + 1;
                                    obj3_idx = num7;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // applyStackingOld
            for i in 0..count {
                if stack_heights[i] != 0 && !base_objects[i].is_slider {
                    continue;
                }
                let mut start_time = base_objects[i].end_time;
                let mut num2 = 0;
                for j in i + 1..count {
                    let num3 = stack_threshold;
                    if (base_objects[j].start_time - num3 as f64) > start_time {
                        break;
                    }
                    let end_pos = if base_objects[i].is_slider { base_objects[i].tail_pos } else { base_objects[i].pos };
                    if v2_dist(base_objects[j].pos, base_objects[i].pos) < 3.0 {
                        stack_heights[i] += 1;
                        start_time = base_objects[j].start_time;
                    } else if v2_dist(base_objects[j].pos, end_pos) < 3.0 {
                        num2 += 1;
                        stack_heights[j] -= num2;
                        start_time = base_objects[j].start_time;
                    }
                }
            }
        }
    }

    let scale_f32 = (1.0f32 - 0.7f32 * (cs as f32 - 5.0f32) / 5.0f32) / 2.0f32;
    for (i, obj) in base_objects.iter_mut().enumerate() {
        let offset = (stack_heights[i] as f32 * scale_f32 * -6.4) as f64;
        obj.stacked_pos = (obj.pos.0 + offset, obj.pos.1 + offset);
        obj.tail_pos = (obj.tail_pos.0 + offset, obj.tail_pos.1 + offset);
        if let Some(ref mut lep) = obj.lazy_end_pos {
            *lep = (lep.0 + offset, lep.1 + offset);
        }
        for nested in obj.nested_objects.iter_mut() {
            nested.pos = (nested.pos.0 + offset, nested.pos.1 + offset);
        }
    }

    let mut difficulty_objects: Vec<LazerDifficultyHitObject> = Vec::with_capacity(base_objects.len().saturating_sub(1));

    for i in 1..base_objects.len() {
        let curr = &base_objects[i];
        let last = &base_objects[i - 1];

        let dt = (curr.start_time - last.start_time).max(0.0) / clock_rate;
        let adj_dt = dt.max(25.0);

        let last_end_dt = if i > 1 {
            let prev_diff = &difficulty_objects[i - 2];
            (curr.start_time / clock_rate - prev_diff.end_time).max(25.0)
        } else {
            adj_dt
        };

        let mut diff_obj = LazerDifficultyHitObject {
            index: i - 1,
            start_time: curr.start_time / clock_rate,
            end_time: curr.end_time / clock_rate,
            delta_time: dt,
            adjusted_delta_time: adj_dt,
            last_object_end_delta_time: last_end_dt,
            jump_distance: 0.0,
            lazy_jump_distance: 0.0,
            min_jump_distance: 0.0,
            min_jump_time: adj_dt,
            travel_distance: curr.travel_distance,
            travel_time: curr.travel_time,
            lazy_travel_distance: curr.lazy_travel_distance,
            lazy_travel_time: curr.lazy_travel_time,
            angle: None,
            normalised_vector_angle: None,
            small_circle_bonus,
            is_slider: curr.is_slider,
            is_spinner: curr.is_spinner,
            hit_window_great: 2.0 * great_window,
            overall_difficulty: overall_diff,
            pos: curr.stacked_pos,
            head_pos: curr.stacked_pos,
            tail_pos: curr.tail_pos,
            lazy_end_pos: curr.lazy_end_pos,
            radius,
            base_start_time: curr.start_time,
            preempt,
            base_preempt,
            base_fade_in,
        };

        if !curr.is_spinner && !last.is_spinner {
            let scaling = 50.0 / radius;
            let end_cursor = if i > 1 {
                let prev_diff = &difficulty_objects[i - 2];
                prev_diff.lazy_end_pos.unwrap_or(last.stacked_pos)
            } else {
                last.stacked_pos
            };

            let dx = curr.stacked_pos.0 - last.stacked_pos.0;
            let dy = curr.stacked_pos.1 - last.stacked_pos.1;
            let jump_dist = (dx * dx + dy * dy).sqrt() * scaling;
            diff_obj.jump_distance = jump_dist;

            let lazy_dx = curr.stacked_pos.0 - end_cursor.0;
            let lazy_dy = curr.stacked_pos.1 - end_cursor.1;
            let lazy_jump_dist = (lazy_dx * lazy_dx + lazy_dy * lazy_dy).sqrt() * scaling;
            diff_obj.lazy_jump_distance = lazy_jump_dist;
            diff_obj.min_jump_distance = lazy_jump_dist;

            if last.is_slider && i > 1 {
                let prev_diff = &difficulty_objects[i - 2];
                let travel_t = (prev_diff.lazy_travel_time / clock_rate).max(25.0);
                diff_obj.min_jump_time = (adj_dt - travel_t).max(25.0);
                let tail_dx = last.tail_pos.0 - curr.stacked_pos.0;
                let tail_dy = last.tail_pos.1 - curr.stacked_pos.1;
                let tail_dist = (tail_dx * tail_dx + tail_dy * tail_dy).sqrt() * scaling;
                diff_obj.min_jump_distance = (lazy_jump_dist - 30.0).min(tail_dist - 120.0).max(0.0);
            }

            if i >= 3 && !base_objects[i - 2].is_spinner {
                let prev_diff = &difficulty_objects[i - 2];
                let prev2_diff = &difficulty_objects[i - 3];

                let mut v = prev_diff.lazy_end_pos.unwrap_or(last.stacked_pos);
                if prev_diff.is_slider && prev_diff.travel_distance > 0.0 {
                    v = last.stacked_pos;
                }

                let end_cursor2 = prev2_diff.lazy_end_pos.unwrap_or(base_objects[i - 2].stacked_pos);
                let val = calculate_angle(curr.stacked_pos, v, end_cursor2);

                let mut last_last_cursor = end_cursor2;
                if prev_diff.is_slider && prev_diff.travel_distance > 0.0 && last.nested_objects.len() >= 2 {
                    last_last_cursor = last.nested_objects[last.nested_objects.len() - 2].pos;
                }
                let end_cursor = prev_diff.lazy_end_pos.unwrap_or(last.stacked_pos);
                let val2 = calculate_angle(curr.stacked_pos, end_cursor, last_last_cursor);

                let vec_dx = (curr.stacked_pos.0 - v.0).abs();
                let vec_dy = (curr.stacked_pos.1 - v.1).abs();
                diff_obj.normalised_vector_angle = Some(vec_dy.atan2(vec_dx));
                diff_obj.angle = Some(val.min(val2));
            }
        }

        difficulty_objects.push(diff_obj);
    }

    difficulty_objects
}

#[inline]
fn calculate_angle(curr: (f64, f64), last: (f64, f64), last_last: (f64, f64)) -> f64 {
    let left = (last_last.0 - last.0, last_last.1 - last.1);
    let right = (curr.0 - last.0, curr.1 - last.1);
    let dot = left.0 * right.0 + left.1 * right.1;
    let cross = left.0 * right.1 - left.1 * right.0;
    cross.atan2(dot).abs()
}

#[derive(Clone, Debug)]
pub struct Island {
    pub delta: i32,
    pub delta_count: i32,
    pub occurrences: i32,
}

impl Island {
    pub fn new(delta: i32) -> Self {
        Self {
            delta: delta.max(25),
            delta_count: 1,
            occurrences: 1,
        }
    }

    pub fn add_delta(&mut self, delta: i32) {
        if self.delta == i32::MAX {
            self.delta = delta.max(25);
        }
        self.delta_count += 1;
    }

    pub fn is_similar_polarity(&self, other: &Island, epsilon: f64) -> bool {
        if self.delta_count <= 1 || other.delta_count <= 1 {
            return false;
        }
        if ((self.delta - other.delta).abs() as f64) < epsilon {
            return (self.delta_count % 2) == (other.delta_count % 2);
        }
        false
    }

    pub fn almost_equals(&self, other: &Island, epsilon: f64) -> bool {
        if ((self.delta - other.delta).abs() as f64) < epsilon {
            return self.delta_count == other.delta_count;
        }
        false
    }
}

pub struct SnapAimEvaluator;

impl SnapAimEvaluator {
    #[inline]
    pub fn calc_angle_acuteness(angle: f64) -> f64 {
        smoothstep(angle, (140.0_f64).to_radians(), (40.0_f64).to_radians())
    }

    #[inline]
    pub fn calc_angle_wideness(angle: f64) -> f64 {
        smoothstep(angle, (40.0_f64).to_radians(), (140.0_f64).to_radians())
    }

    #[inline]
    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.03_f64.powf((ms / 1000.0).powf(0.65)))
    }

    fn vector_angle_repetition(objects: &[LazerDifficultyHitObject], index: usize) -> f64 {
        let curr = &objects[index];
        let prev = &objects[index - 1];
        if curr.angle.is_none() || prev.angle.is_none() {
            return 1.0;
        }
        let mut num = 0.0;
        for i in 0..6 {
            if index <= i {
                break;
            }
            let obj = &objects[index - 1 - i];
            let max_dt = curr.adjusted_delta_time.max(obj.adjusted_delta_time);
            let min_dt = curr.adjusted_delta_time.min(obj.adjusted_delta_time);
            if max_dt > 1.1 * min_dt {
                break;
            }
            if let (Some(curr_v), Some(obj_v)) = (curr.normalised_vector_angle, obj.normalised_vector_angle) {
                let val = (curr_v - obj_v).abs();
                num += (8.0 * (11.25_f64.to_radians()).min(val)).cos();
            }
        }
        let num2 = (0.5 / num.max(0.0001)).min(1.0).powi(2);
        let num3 = smootherstep(curr.lazy_jump_distance, 0.0, 100.0);
        let angle = curr.angle.unwrap();
        let prev_angle = prev.angle.unwrap();
        let num4 = (2.0 * (45.0_f64.to_radians()).min((angle - prev_angle).abs() * num3)).cos();
        let num5 = 1.0 - 0.15 * Self::calc_angle_acuteness(prev_angle) * num4;
        (num5 + (1.0 - num5) * num2 * 0.5 * num3).powi(2)
    }

    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
        with_slider_travel: bool,
    ) -> f64 {
        if index <= 1 || index >= objects.len() {
            return 0.0;
        }
        let curr = &objects[index];
        let prev = &objects[index - 1];
        if curr.is_spinner || prev.is_spinner {
            return 0.0;
        }

        let num = if with_slider_travel { curr.lazy_jump_distance } else { curr.jump_distance };
        let mut num2 = num / curr.adjusted_delta_time;
        if prev.is_slider && with_slider_travel {
            let num3 = prev.lazy_travel_distance + curr.lazy_jump_distance;
            num2 = num2.max(num3 / curr.adjusted_delta_time);
        }

        let num4 = if with_slider_travel { prev.lazy_jump_distance } else { prev.jump_distance };
        let num5 = num4 / prev.adjusted_delta_time;
        let mut num6 = num2;
        num6 *= Self::vector_angle_repetition(objects, index);

        if let (Some(angle), Some(prev_angle)) = (curr.angle, prev.angle) {
            let num7 = num2.min(num5);
            let mut num8 = 0.0;
            let max_dt = curr.adjusted_delta_time.max(prev.adjusted_delta_time);
            let min_dt = curr.adjusted_delta_time.min(prev.adjusted_delta_time);

            if max_dt < 1.25 * min_dt {
                num8 = Self::calc_angle_acuteness(angle);
                num8 *= 0.08 + 0.92 * (1.0 - num8.min(Self::calc_angle_acuteness(prev_angle).powi(3)));
                num8 *= num7 * smootherstep(milliseconds_to_bpm(curr.adjusted_delta_time, 2.0), 300.0, 400.0) * smootherstep(num, 0.0, 200.0);
            }

            let mut num9 = Self::calc_angle_wideness(angle);
            num9 *= 0.25 + 0.75 * (1.0 - num9.min(Self::calc_angle_wideness(prev_angle).powi(3)));
            let mut val = num / curr.adjusted_delta_time.powf(1.45);
            let val2 = num4 / prev.adjusted_delta_time.powf(1.45);
            if prev.is_slider && with_slider_travel {
                let num10 = prev.lazy_travel_distance + curr.lazy_jump_distance;
                val = val.max(num10 / curr.adjusted_delta_time.powf(1.45));
            }
            num9 *= val.min(val2);

            if index >= 3 {
                let prev3 = &objects[index - 3];
                let dx = (prev.pos.0 - prev3.pos.0) as f32;
                let dy = (prev.pos.1 - prev3.pos.1) as f32;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 1.0 {
                    num9 *= 1.0 - 0.55 * (1.0 - len as f64);
                }
            }

            num6 += (num8 * 2.41).max(num9 * 9.67);

            let num11 = num7 * smootherstep(num, 50.0, 100.0) * reverse_lerp(num, 300.0, 100.0).powf(1.8) * smootherstep(angle, (110.0_f64).to_radians(), (60.0_f64).to_radians())
                * smootherstep(num4, 50.0, 100.0) * reverse_lerp(num4, 300.0, 100.0).powf(1.8) * smootherstep(prev_angle, (110.0_f64).to_radians(), (60.0_f64).to_radians());
            num6 += num11 * 1.02;
        }

        if num5.max(num2) != 0.0 {
            if with_slider_travel {
                num2 = num / curr.adjusted_delta_time;
            }
            let num12 = smoothstep((num5 - num2).abs() / num5.max(num2), 0.0, 1.0);
            let min_dt = curr.adjusted_delta_time.min(prev.adjusted_delta_time);
            let max_dt = curr.adjusted_delta_time.max(prev.adjusted_delta_time);
            let mut num13 = (125.0 / min_dt).min((num5 - num2).abs()) * num12;
            num13 *= (min_dt / max_dt).powi(2);
            num6 += num13 * 0.9;
        }

        if curr.is_slider && with_slider_travel && curr.travel_time > 0.0 {
            let num14 = curr.travel_distance / curr.travel_time;
            num6 += if num14 < 1.0 { num14 } else { num14.powf(0.75) } * 1.5;
        }

        num6 * curr.small_circle_bonus * Self::high_bpm_bonus(curr.adjusted_delta_time)
    }
}

pub struct FlowAimEvaluator;

impl FlowAimEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
        with_slider_travel: bool,
    ) -> f64 {
        if index <= 1 || index >= objects.len() {
            return 0.0;
        }
        let curr = &objects[index];
        let prev = &objects[index - 1];
        if curr.is_spinner || prev.is_spinner {
            return 0.0;
        }

        let num = if with_slider_travel { curr.lazy_jump_distance } else { curr.jump_distance };
        let num2 = if with_slider_travel { prev.lazy_jump_distance } else { prev.jump_distance };
        let mut num3 = num / curr.adjusted_delta_time;
        if prev.is_slider && with_slider_travel {
            let num4 = prev.lazy_travel_distance + curr.lazy_jump_distance;
            num3 = num3.max(num4 / curr.adjusted_delta_time);
        }
        let num5 = num2 / prev.adjusted_delta_time;

        let mut num6 = num3 * curr.small_circle_bonus.sqrt();
        let max_dt = curr.adjusted_delta_time.max(prev.adjusted_delta_time);
        let min_dt = curr.adjusted_delta_time.min(prev.adjusted_delta_time);
        num6 *= 1.0 + 0.25_f64.min(((max_dt - min_dt) / 50.0).powi(4));

        if let (Some(angle), Some(prev_angle)) = (curr.angle, prev.angle) {
            let num7 = ((angle - prev_angle).abs() / 2.0).sin() * 180.0 / (curr.adjusted_delta_time * 0.1);
            num6 *= 0.8 + (num7 / 270.0).sqrt();
        }

        let mut num8 = 1.0;
        if index > 2 {
            let prev2 = &objects[index - 2];
            let num9 = Self::calculate_overlap_factor(curr, prev);
            let num10 = Self::calculate_overlap_factor(curr, prev2);
            let num11 = Self::calculate_overlap_factor(prev, prev2);
            num8 = 1.0 - num9 * num10 * num11;
        }

        if let Some(angle) = curr.angle {
            num6 += num3 * SnapAimEvaluator::calc_angle_acuteness(angle) * num8;
        }

        if num5.max(num3) != 0.0 {
            if with_slider_travel {
                num3 = num / curr.adjusted_delta_time;
            }
            let num12 = smoothstep((num5 - num3).abs() / num5.max(num3), 0.0, 1.0);
            let num13 = (125.0 / min_dt).min((num5 - num3).abs());
            num6 += num13 * num12 * num8 * 0.52;
        }

        if curr.is_slider && with_slider_travel && curr.travel_time > 0.0 {
            num6 += curr.travel_distance / curr.travel_time;
        }

        num6.powf(1.45) * smootherstep(num, 0.0, 50.0)
    }

    fn calculate_overlap_factor(first: &LazerDifficultyHitObject, second: &LazerDifficultyHitObject) -> f64 {
        let dx = first.pos.0 - second.pos.0;
        let dy = first.pos.1 - second.pos.1;
        let dist = (dx * dx + dy * dy).sqrt();
        let radius = first.radius.max(1.0);
        (1.0 - ((dist - radius).max(0.0) / radius).powi(2)).clamp(0.0, 1.0)
    }
}

pub struct AgilityEvaluator;

impl AgilityEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
    ) -> f64 {
        let curr = &objects[index];
        if curr.is_spinner {
            return 0.0;
        }
        let prev_travel = if index > 0 { objects[index - 1].lazy_travel_distance } else { 0.0 };
        let jump = (prev_travel + curr.lazy_jump_distance).min(120.0);
        let bpm_bonus = 1.0 / (1.0 - 0.2_f64.powf(curr.adjusted_delta_time / 1000.0));
        (jump / 120.0) * (1000.0 / curr.adjusted_delta_time) * curr.small_circle_bonus.powf(1.5) * bpm_bonus
    }
}

pub struct SpeedEvaluator;

impl SpeedEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
    ) -> f64 {
        let curr = &objects[index];
        if curr.is_spinner {
            return 0.0;
        }
        let next_obj = objects.get(index + 1);
        let double_tap = 1.0 - curr.calculate_double_tap_feasibility(next_obj);

        let mut adj_dt = curr.adjusted_delta_time;
        let window_ratio = (adj_dt / curr.hit_window_great / 0.93).clamp(0.92, 1.0);
        adj_dt /= window_ratio;

        let mut high_bpm_bonus = 0.0;
        if milliseconds_to_bpm(adj_dt, 4.0) > 200.0 {
            high_bpm_bonus = 0.75 * ((bpm_to_milliseconds(200.0, 4.0) - adj_dt) / 40.0).powi(2);
        }
        let bpm_term = 1.0 / (1.0 - 0.3_f64.powf(curr.adjusted_delta_time / 1000.0));
        (1.0 + high_bpm_bonus) * (1000.0 / adj_dt) * bpm_term * double_tap
    }
}

pub struct RhythmEvaluator;

impl RhythmEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
    ) -> f64 {
        let curr = &objects[index];
        if curr.is_spinner {
            return 0.0;
        }
        if index == 0 {
            return 1.0;
        }

        let num2 = curr.hit_window_great * 0.3;
        let mut island = Island::new(i32::MAX);
        let mut island2 = Island::new(i32::MAX);
        let mut list: Vec<Island> = Vec::new();
        let mut num3 = 0.0;
        let mut flag = false;
        let num4 = index.min(32);

        let mut i = 0;
        while i < num4.saturating_sub(2) && index >= i + 1 && (curr.start_time - objects[index - 1 - i].start_time) < 5000.0 {
            i += 1;
        }

        if index < i + 1 {
            return 1.0;
        }

        let mut osu_diff_obj = &objects[index - 1 - i];
        let mut osu_diff_obj2 = if index >= i + 2 { &objects[index - 2 - i] } else { osu_diff_obj };

        let mut num = 0.0;

        for num5 in (1..=i).rev() {
            let target_idx = index - num5;
            let osu_diff_obj3 = &objects[target_idx];
            if osu_diff_obj3.is_spinner {
                continue;
            }

            let val = ((5000.0 - (curr.start_time - osu_diff_obj3.start_time)) / 5000.0).max(0.0);
            let num6 = ((num4 - num5) as f64 / num4 as f64).min(val);
            let num7 = osu_diff_obj3.delta_time.max(1e-7);
            let num8 = osu_diff_obj.delta_time.max(1e-7);
            let num9 = (num8 - num7).abs();

            if island.delta == i32::MAX {
                island = Island::new(num7 as i32);
            }

            let num10 = num8.max(num7) / num8.min(num7);
            let num11 = (2.0 - num10 / 8.0).clamp(0.0, 1.0);
            let num12 = ((num9 - num2) / num2).clamp(0.0, 1.0);
            let mut num13 = Self::get_effective_difficulty(num10) * num12 * num11;

            if osu_diff_obj.is_slider {
                let min_jump_time = osu_diff_obj3.min_jump_time;
                let delta_diff_ratio = min_jump_time.max(num7) / min_jump_time.min(num7);
                let last_end_dt = osu_diff_obj3.last_object_end_delta_time;
                let term1 = Self::get_effective_difficulty(delta_diff_ratio);
                let term2 = Self::get_effective_difficulty(last_end_dt.max(num7) / last_end_dt.min(num7));
                num13 = num13.min(term1.min(term2));
            }

            if num9 < num2 {
                island.add_delta(num7 as i32);
            }

            if flag {
                if num9 > num2 {
                    if osu_diff_obj3.is_slider {
                        num13 *= 0.5;
                    }
                    if island.is_similar_polarity(&island2, num2) {
                        num13 *= 0.5;
                    }
                    if osu_diff_obj2.delta_time.max(1e-7) > num8 + num2 && num8 > num7 + num2 {
                        num13 *= 0.125;
                    }
                    if island2.delta_count == island.delta_count {
                        num13 *= 0.5;
                    }
                    if num8 > num7 + num2 {
                        num13 *= 0.65;
                    }

                    let mut flag2 = false;
                    for item in list.iter_mut() {
                        if item.almost_equals(&island, num2) {
                            if island2.almost_equals(&island, num2) {
                                item.occurrences += 1;
                            }
                            let exponent = logistic(island.delta as f64, 58.33, 0.24, 2.75);
                            num13 *= (3.0 / item.occurrences as f64).min((1.0 / item.occurrences as f64).powf(exponent));
                            flag2 = true;
                            break;
                        }
                    }

                    if !flag2 && island.delta_count > 0 {
                        list.push(island.clone());
                    }

                    num13 *= 1.0 - osu_diff_obj.calculate_double_tap_feasibility(Some(osu_diff_obj3)) * 0.75;
                    num = if island.delta_count <= 1 {
                        num + 0.7 * num6
                    } else {
                        num + (num13 * num3).sqrt() * num6
                    };

                    num3 = num13;
                    if num8 + num2 < num7 {
                        flag = false;
                    }
                    island2 = island;
                    island = Island::new(num7 as i32);
                }
            } else if num8 > num7 + num2 {
                flag = true;
                if osu_diff_obj3.is_slider {
                    num13 *= 0.6;
                }
                if osu_diff_obj.is_slider {
                    num13 *= 0.6;
                }
                num3 = num13;
                island = Island::new(num7 as i32);
            }

            osu_diff_obj2 = osu_diff_obj;
            osu_diff_obj = osu_diff_obj3;
        }

        num *= reverse_lerp(island.delta_count as f64, 22.0, 3.0);
        ((4.0 + num * 0.95).sqrt()) / 2.0
    }

    #[inline]
    fn get_effective_difficulty(delta_difference_ratio: f64) -> f64 {
        let x = delta_difference_ratio - delta_difference_ratio.floor();
        1.0 + 26.0 * (0.5_f64.min(smoothstep_bell_curve_unit(x)))
    }
}

pub struct ReadingEvaluator;

impl ReadingEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
        has_hidden: bool,
    ) -> f64 {
        if index == 0 || index >= objects.len() {
            return 0.0;
        }
        let curr = &objects[index];
        if curr.is_spinner {
            return 0.0;
        }

        let next_obj = objects.get(index + 1);
        let velocity = (curr.lazy_jump_distance / curr.adjusted_delta_time).max(1.0);
        let current_visible_density = Self::retrieve_current_visible_object_density(objects, index);
        let past_influence = Self::get_past_object_difficulty_influence(objects, index);
        let angle_nerf = Self::get_constant_angle_nerf_factor(objects, index);

        let density_diff = Self::calculate_density_difficulty(
            next_obj,
            velocity,
            angle_nerf,
            past_influence,
            current_visible_density,
        );

        let hidden_diff = if has_hidden {
            Self::calculate_hidden_difficulty(
                objects,
                index,
                past_influence,
                current_visible_density,
                velocity,
                angle_nerf,
            )
        } else {
            0.0
        };

        let preempt_diff = Self::calculate_preempt_difficulty(velocity, angle_nerf, curr.preempt);
        let combined = norm(1.5, &[preempt_diff, hidden_diff, density_diff]);
        combined * Self::high_bpm_bonus(curr.adjusted_delta_time)
    }

    fn calculate_density_difficulty(
        next_obj: Option<&LazerDifficultyHitObject>,
        velocity: f64,
        angle_nerf: f64,
        past_influence: f64,
        visible_density: f64,
    ) -> f64 {
        let mut num = visible_density.sqrt();
        if let Some(next) = next_obj {
            num *= smootherstep(next.lazy_jump_distance, 15.0, 150.0);
        }
        let mut num2 = (past_influence + num).powf(1.7) * 0.4 * angle_nerf * velocity;
        num2 = (num2 - 2.5).max(0.0);
        num2.powf(0.45) * 2.4
    }

    fn calculate_preempt_difficulty(velocity: f64, angle_nerf: f64, preempt: f64) -> f64 {
        let p_factor = (500.0 - preempt).max(0.0);
        p_factor.powf(2.5) / 140000.0 * (angle_nerf * velocity)
    }

    fn calculate_hidden_difficulty(
        objects: &[LazerDifficultyHitObject],
        index: usize,
        past_influence: f64,
        visible_density: f64,
        velocity: f64,
        angle_nerf: f64,
    ) -> f64 {
        let curr = &objects[index];
        let num = curr.preempt.powf(2.2) * 0.01;
        let num2 = (visible_density + past_influence).powf(3.3) * 3.0;
        let mut x = (num + num2) * angle_nerf * velocity * 0.01;
        x = x.powf(0.4) * 0.28;

        if index > 0 {
            let prev = &objects[index - 1];
            if curr.lazy_jump_distance == 0.0
                && curr.opacity_at(prev.base_start_time, true) == 0.0
                && prev.start_time > curr.start_time - curr.preempt
            {
                x += 700.0 / curr.adjusted_delta_time.powf(1.5);
            }
        }

        x
    }

    fn get_past_object_difficulty_influence(objects: &[LazerDifficultyHitObject], index: usize) -> f64 {
        let curr = &objects[index];
        let mut sum = 0.0;
        for i in 0..index {
            let prev = &objects[index - 1 - i];
            if curr.start_time - prev.start_time > 3000.0 || prev.start_time < curr.start_time - curr.preempt {
                break;
            }
            let mut num2 = curr.opacity_at(prev.base_start_time, false);
            num2 *= smootherstep(prev.lazy_jump_distance, 15.0, 150.0);
            num2 *= Self::get_time_nerf_factor(curr.start_time - prev.start_time);
            sum += num2;
        }
        sum
    }

    fn retrieve_current_visible_object_density(objects: &[LazerDifficultyHitObject], index: usize) -> f64 {
        let curr = &objects[index];
        let mut sum = 0.0;
        for next in objects.iter().skip(index + 1) {
            if next.start_time - curr.start_time > 3000.0 || curr.start_time < next.start_time - next.preempt {
                break;
            }
            let time_nerf = Self::get_time_nerf_factor(next.start_time - curr.start_time);
            sum += next.opacity_at(curr.base_start_time, false) * time_nerf;
        }
        sum
    }

    fn get_constant_angle_nerf_factor(objects: &[LazerDifficultyHitObject], index: usize) -> f64 {
        let curr = &objects[index];
        let mut num = 0.0;
        let mut num2 = 0;
        let mut num3 = 0.0;
        let mut obj = curr;
        let mut obj2: Option<&LazerDifficultyHitObject> = None;
        let mut obj3: Option<&LazerDifficultyHitObject> = None;

        while num3 < 2000.0 && num2 < index {
            let obj4 = &objects[index - 1 - num2];
            let num4 = 1.0 - reverse_lerp(obj4.adjusted_delta_time, 200.0, 2000.0);

            if let (Some(curr_angle), Some(obj4_angle)) = (curr.angle, obj4.angle) {
                let val = (curr_angle - obj4_angle).abs();
                let mut val2 = PI;

                if let (Some(o2), Some(o3)) = (obj2, obj3) {
                    if let (Some(a_obj), Some(a_o2), Some(a_o3)) = (obj.angle, o2.angle, o3.angle) {
                        val2 = (a_o2 - obj4_angle).abs() + (a_o3 - a_obj).abs();
                        let mut num5 = 1.0;
                        num5 *= reverse_lerp(obj4_angle.min(a_obj) * 180.0 / PI, 20.0, 5.0);
                        num5 *= reverse_lerp(obj4_angle.max(a_obj) * 180.0 / PI, 60.0, 120.0);
                        val2 = PI + (0.1 * val2 - PI) * num5;
                    }
                }

                let num6 = smootherstep(obj4.lazy_jump_distance, 0.0, 50.0);
                num += (3.0 * (30.0_f64.to_radians()).min(val.min(val2) * num6)).cos() * num4;
            }

            num3 = curr.start_time - obj4.start_time;
            num2 += 1;
            obj3 = obj2;
            obj2 = Some(obj);
            obj = obj4;
        }

        (2.0 / num).clamp(0.2, 1.0)
    }

    #[inline]
    fn get_time_nerf_factor(delta_time: f64) -> f64 {
        (2.0 - delta_time / 1500.0).clamp(0.0, 1.0)
    }

    #[inline]
    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.8_f64.powf(ms / 1000.0))
    }
}

pub struct FlashlightEvaluator;

impl FlashlightEvaluator {
    pub fn evaluate_difficulty_of(
        objects: &[LazerDifficultyHitObject],
        index: usize,
        has_hidden: bool,
    ) -> f64 {
        let curr = &objects[index];
        if curr.is_spinner || index == 0 {
            return 0.0;
        }
        let scaling = 52.0 / curr.radius;
        let mut num2 = 1.0;
        let mut num3 = 0.0;
        let mut num4 = 0.0;
        let mut osu_diff_obj2 = curr;
        let mut num5 = 0.0;

        let lookback = index.min(10);
        for i in 0..lookback {
            let osu_diff_obj3 = &objects[index - 1 - i];
            num3 += osu_diff_obj2.adjusted_delta_time;
            if !osu_diff_obj3.is_spinner {
                let dx = curr.pos.0 - osu_diff_obj3.tail_pos.0;
                let dy = curr.pos.1 - osu_diff_obj3.tail_pos.1;
                let num6 = (dx * dx + dy * dy).sqrt();
                if i == 0 {
                    num2 = (num6 / 75.0).min(1.0);
                }
                let num7 = (osu_diff_obj3.lazy_jump_distance / scaling / 25.0).min(1.0);
                let num8 = 1.0 + 0.4 * (1.0 - curr.opacity_at(osu_diff_obj3.base_start_time, has_hidden));
                num4 += num7 * num8 * scaling * num6 / num3;
                if let (Some(a3), Some(a_curr)) = (osu_diff_obj3.angle, curr.angle) {
                    if (a3 - a_curr).abs() < 0.02 {
                        num5 += (1.0 - 0.1 * i as f64).max(0.0);
                    }
                }
            }
            osu_diff_obj2 = osu_diff_obj3;
        }

        num4 = (num2 * num4).powi(2);
        if has_hidden {
            num4 *= 1.2;
        }
        num4 *= 0.2 + 0.8 / (num5 + 1.0);

        let mut num9 = 0.0;
        if curr.is_slider && curr.travel_time > 0.0 {
            let num10 = curr.lazy_travel_distance / scaling;
            num9 = ((num10 / curr.travel_time - 0.5).max(0.0)).sqrt() * num10;
        }

        num4 + num9 * 1.3
    }
}
