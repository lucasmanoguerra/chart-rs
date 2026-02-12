use chrono::{DateTime, FixedOffset, Timelike, Utc};

use super::label_cache::TimeLabelPattern;
use super::{
    AxisLabelLocale, PriceAxisDisplayMode, PriceAxisLabelConfig, PriceAxisLabelPolicy,
    TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeAxisSessionConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResolvedTimeLabelPattern {
    LogicalDecimal { precision: u8 },
    Utc { pattern: TimeLabelPattern },
}

pub(super) fn resolve_time_label_pattern(
    policy: TimeAxisLabelPolicy,
    visible_span_abs: f64,
) -> ResolvedTimeLabelPattern {
    match policy {
        TimeAxisLabelPolicy::LogicalDecimal { precision } => {
            ResolvedTimeLabelPattern::LogicalDecimal { precision }
        }
        TimeAxisLabelPolicy::UtcDateTime { show_seconds } => {
            let pattern = if show_seconds {
                TimeLabelPattern::DateSecond
            } else {
                TimeLabelPattern::DateMinute
            };
            ResolvedTimeLabelPattern::Utc { pattern }
        }
        TimeAxisLabelPolicy::UtcAdaptive => {
            let pattern = if visible_span_abs <= 600.0 {
                TimeLabelPattern::DateSecond
            } else if visible_span_abs <= 172_800.0 {
                TimeLabelPattern::DateMinute
            } else {
                TimeLabelPattern::Date
            };
            ResolvedTimeLabelPattern::Utc { pattern }
        }
    }
}

pub(super) fn quantize_logical_time_millis(logical_time: f64) -> i64 {
    if !logical_time.is_finite() {
        return 0;
    }
    let millis = (logical_time * 1_000.0).round();
    if millis > (i64::MAX as f64) {
        i64::MAX
    } else if millis < (i64::MIN as f64) {
        i64::MIN
    } else {
        millis as i64
    }
}

pub(super) fn quantize_price_label_value(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    let nanos = (value * 1_000_000_000.0).round();
    if nanos > (i64::MAX as f64) {
        i64::MAX
    } else if nanos < (i64::MIN as f64) {
        i64::MIN
    } else {
        nanos as i64
    }
}

pub(super) fn format_time_axis_label(
    logical_time: f64,
    config: TimeAxisLabelConfig,
    visible_span_abs: f64,
) -> String {
    if !logical_time.is_finite() {
        return "nan".to_owned();
    }

    match resolve_time_label_pattern(config.policy, visible_span_abs) {
        ResolvedTimeLabelPattern::LogicalDecimal { precision } => {
            format_axis_decimal(logical_time, usize::from(precision), config.locale)
        }
        ResolvedTimeLabelPattern::Utc { pattern } => {
            let seconds = logical_time.round() as i64;
            let Some(dt) = DateTime::<Utc>::from_timestamp(seconds, 0) else {
                return format_axis_decimal(logical_time, 2, config.locale);
            };
            let local_dt = dt.with_timezone(&config.timezone.fixed_offset());
            let pattern = resolve_session_time_label_pattern(pattern, config.session, local_dt);

            let pattern = match (config.locale, pattern) {
                (AxisLabelLocale::EnUs, TimeLabelPattern::Date) => "%Y-%m-%d",
                (AxisLabelLocale::EnUs, TimeLabelPattern::DateMinute) => "%Y-%m-%d %H:%M",
                (AxisLabelLocale::EnUs, TimeLabelPattern::DateSecond) => "%Y-%m-%d %H:%M:%S",
                (AxisLabelLocale::EnUs, TimeLabelPattern::TimeMinute) => "%H:%M",
                (AxisLabelLocale::EnUs, TimeLabelPattern::TimeSecond) => "%H:%M:%S",
                (AxisLabelLocale::EsEs, TimeLabelPattern::Date) => "%d/%m/%Y",
                (AxisLabelLocale::EsEs, TimeLabelPattern::DateMinute) => "%d/%m/%Y %H:%M",
                (AxisLabelLocale::EsEs, TimeLabelPattern::DateSecond) => "%d/%m/%Y %H:%M:%S",
                (AxisLabelLocale::EsEs, TimeLabelPattern::TimeMinute) => "%H:%M",
                (AxisLabelLocale::EsEs, TimeLabelPattern::TimeSecond) => "%H:%M:%S",
            };
            local_dt.format(pattern).to_string()
        }
    }
}

pub(super) fn format_time_axis_label_with_precision(
    logical_time: f64,
    config: TimeAxisLabelConfig,
    visible_span_abs: f64,
    precision: u8,
) -> String {
    if matches!(config.policy, TimeAxisLabelPolicy::LogicalDecimal { .. }) {
        return format_axis_decimal(logical_time, usize::from(precision), config.locale);
    }
    format_time_axis_label(logical_time, config, visible_span_abs)
}

fn resolve_session_time_label_pattern(
    pattern: TimeLabelPattern,
    session: Option<TimeAxisSessionConfig>,
    local_dt: DateTime<FixedOffset>,
) -> TimeLabelPattern {
    let Some(session) = session else {
        return pattern;
    };

    // Session mode keeps boundary timestamps explicit while reducing in-session
    // noise to time-only labels for intraday readability.
    let minute_of_day = (local_dt.hour() * 60 + local_dt.minute()) as u16;
    if !session.contains_local_minute(minute_of_day) {
        return pattern;
    }
    if session.is_boundary(minute_of_day, local_dt.second()) {
        return pattern;
    }

    match pattern {
        TimeLabelPattern::DateMinute => TimeLabelPattern::TimeMinute,
        TimeLabelPattern::DateSecond => TimeLabelPattern::TimeSecond,
        other => other,
    }
}

pub(super) fn is_major_time_tick(logical_time: f64, config: TimeAxisLabelConfig) -> bool {
    if !logical_time.is_finite() {
        return false;
    }
    if matches!(config.policy, TimeAxisLabelPolicy::LogicalDecimal { .. }) {
        return false;
    }

    let seconds = logical_time.round() as i64;
    let Some(dt) = DateTime::<Utc>::from_timestamp(seconds, 0) else {
        return false;
    };
    let local_dt = dt.with_timezone(&config.timezone.fixed_offset());
    let minute_of_day = (local_dt.hour() * 60 + local_dt.minute()) as u16;

    if let Some(session) = config.session {
        if session.is_boundary(minute_of_day, local_dt.second()) {
            return true;
        }
    }

    local_dt.hour() == 0 && local_dt.minute() == 0 && local_dt.second() == 0
}

fn resolved_price_display_base(mode: PriceAxisDisplayMode, fallback_base_price: f64) -> f64 {
    let explicit_base = match mode {
        PriceAxisDisplayMode::Normal => None,
        PriceAxisDisplayMode::Percentage { base_price }
        | PriceAxisDisplayMode::IndexedTo100 { base_price } => base_price,
    };

    let base = explicit_base.unwrap_or(fallback_base_price);
    if !base.is_finite() || base == 0.0 {
        1.0
    } else {
        base
    }
}

pub(super) fn map_price_to_display_value(
    raw_price: f64,
    mode: PriceAxisDisplayMode,
    fallback_base_price: f64,
) -> f64 {
    if !raw_price.is_finite() {
        return raw_price;
    }

    match mode {
        PriceAxisDisplayMode::Normal => raw_price,
        PriceAxisDisplayMode::Percentage { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            ((raw_price / base) - 1.0) * 100.0
        }
        PriceAxisDisplayMode::IndexedTo100 { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            (raw_price / base) * 100.0
        }
    }
}

pub(super) fn map_price_step_to_display_value(
    raw_step_abs: f64,
    mode: PriceAxisDisplayMode,
    fallback_base_price: f64,
) -> f64 {
    if !raw_step_abs.is_finite() || raw_step_abs <= 0.0 {
        return raw_step_abs;
    }

    match mode {
        PriceAxisDisplayMode::Normal => raw_step_abs,
        PriceAxisDisplayMode::Percentage { .. } | PriceAxisDisplayMode::IndexedTo100 { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            (raw_step_abs / base).abs() * 100.0
        }
    }
}

pub(super) fn price_display_mode_suffix(mode: PriceAxisDisplayMode) -> &'static str {
    match mode {
        PriceAxisDisplayMode::Percentage { .. } => "%",
        PriceAxisDisplayMode::Normal | PriceAxisDisplayMode::IndexedTo100 { .. } => "",
    }
}

pub(super) fn format_price_axis_label(
    value: f64,
    config: PriceAxisLabelConfig,
    tick_step_abs: f64,
) -> String {
    if !value.is_finite() {
        return "nan".to_owned();
    }

    match config.policy {
        PriceAxisLabelPolicy::FixedDecimals { precision } => {
            format_axis_decimal(value, usize::from(precision), config.locale)
        }
        PriceAxisLabelPolicy::MinMove {
            min_move,
            trim_trailing_zeros,
        } => {
            let precision = precision_from_step(min_move);
            let snapped = if min_move.is_finite() && min_move > 0.0 {
                (value / min_move).round() * min_move
            } else {
                value
            };
            let text = format_axis_decimal(snapped, precision, config.locale);
            if trim_trailing_zeros {
                trim_axis_decimal(text, config.locale)
            } else {
                text
            }
        }
        PriceAxisLabelPolicy::Adaptive => {
            let nice_step = normalize_step_for_precision(tick_step_abs);
            let precision = precision_from_step(nice_step);
            format_axis_decimal(value, precision, config.locale)
        }
    }
}

pub(super) fn format_price_axis_label_with_precision(
    value: f64,
    config: PriceAxisLabelConfig,
    tick_step_abs: f64,
    precision: u8,
) -> String {
    if !value.is_finite() {
        return "nan".to_owned();
    }
    if precision <= 12 {
        return format_axis_decimal(value, usize::from(precision), config.locale);
    }
    format_price_axis_label(value, config, tick_step_abs)
}

fn normalize_step_for_precision(step_abs: f64) -> f64 {
    if !step_abs.is_finite() || step_abs <= 0.0 {
        return 0.01;
    }

    let magnitude = 10.0_f64.powf(step_abs.log10().floor());
    if !magnitude.is_finite() || magnitude <= 0.0 {
        return step_abs;
    }

    let normalized = step_abs / magnitude;
    let nice = if normalized < 1.5 {
        1.0
    } else if normalized < 3.0 {
        2.0
    } else if normalized < 7.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn precision_from_step(step: f64) -> usize {
    if !step.is_finite() || step <= 0.0 {
        return 2;
    }
    let text = format!("{:.12}", step.abs());
    let Some((_, fraction)) = text.split_once('.') else {
        return 0;
    };
    fraction.trim_end_matches('0').len().clamp(0, 12)
}

fn trim_axis_decimal(mut text: String, locale: AxisLabelLocale) -> String {
    let separator = match locale {
        AxisLabelLocale::EnUs => '.',
        AxisLabelLocale::EsEs => ',',
    };

    if let Some(index) = text.find(separator) {
        let mut trim_start = text.len();
        for (idx, ch) in text.char_indices().rev() {
            if idx <= index {
                break;
            }
            if ch != '0' {
                break;
            }
            trim_start = idx;
        }
        if trim_start < text.len() {
            text.truncate(trim_start);
        }
        if text.ends_with(separator) {
            text.pop();
        }
    }

    if text == "-0" { "0".to_owned() } else { text }
}

fn format_axis_decimal(value: f64, precision: usize, locale: AxisLabelLocale) -> String {
    let text = format!("{value:.precision$}");
    match locale {
        AxisLabelLocale::EnUs => text,
        AxisLabelLocale::EsEs => text.replace('.', ","),
    }
}
