use std::collections::{HashMap, HashSet};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;
use rust_decimal::Decimal;

/// Validation utility functions
#[derive(Debug, Clone)]
pub struct ValidationUtils;


impl ValidationUtils {
    // String validations
    pub fn is_empty_string(s: &str) -> bool {
        s.is_empty()
    }

    pub fn is_blank_string(s: &str) -> bool {
        s.trim().is_empty()
    }

    pub fn has_space_string(s: &str) -> bool {
        s.chars().any(|c| c.is_whitespace())
    }

    pub fn matches_regex(s: &str, pattern: &Regex) -> bool {
        pattern.is_match(s)
    }

    pub fn string_length(s: &str) -> usize {
        s.chars().count()
    }

    // Collection validations
    pub fn is_empty_vec<T>(vec: &[T]) -> bool {
        vec.is_empty()
    }

    pub fn vec_length<T>(vec: &[T]) -> usize {
        vec.len()
    }

    pub fn is_empty_hashset<T>(set: &HashSet<T>) -> bool {
        set.is_empty()
    }

    pub fn hashset_length<T>(set: &HashSet<T>) -> usize {
        set.len()
    }

    pub fn is_empty_hashmap<K, V>(map: &HashMap<K, V>) -> bool {
        map.is_empty()
    }

    pub fn hashmap_length<K, V>(map: &HashMap<K, V>) -> usize {
        map.len()
    }

    // Numeric range validations
    pub fn in_range_i8(value: i8, min: Option<i8>, max: Option<i8>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_i16(value: i16, min: Option<i16>, max: Option<i16>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_i32(value: i32, min: Option<i32>, max: Option<i32>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_i64(value: i64, min: Option<i64>, max: Option<i64>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_i128(value: i128, min: Option<i128>, max: Option<i128>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_u8(value: u8, min: Option<u8>, max: Option<u8>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_u16(value: u16, min: Option<u16>, max: Option<u16>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_u32(value: u32, min: Option<u32>, max: Option<u32>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_u64(value: u64, min: Option<u64>, max: Option<u64>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_u128(value: u128, min: Option<u128>, max: Option<u128>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_f32(value: f32, min: Option<f32>, max: Option<f32>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_f64(value: f64, min: Option<f64>, max: Option<f64>) -> bool {
        Self::in_range_generic(&value, min.as_ref(), max.as_ref())
    }

    pub fn in_range_decimal(value: &Decimal, min: Option<&Decimal>, max: Option<&Decimal>) -> bool {
        Self::in_range_generic(value, min, max)
    }

    // DateTime validations
    pub fn in_range_naive_date_time(value: &NaiveDateTime, min: Option<&NaiveDateTime>, max: Option<&NaiveDateTime>) -> bool {
        Self::in_range_generic(value, min, max)
    }

    pub fn in_range_date_time<T: chrono::TimeZone>(value: &DateTime<T>, min: Option<&DateTime<T>>, max: Option<&DateTime<T>>) -> bool {
        Self::in_range_generic(value, min, max)
    }

    pub fn in_range_naive_date(value: &NaiveDate, min: Option<&NaiveDate>, max: Option<&NaiveDate>) -> bool {
        Self::in_range_generic(value, min, max)
    }

    pub fn in_range_naive_time(value: &NaiveTime, min: Option<&NaiveTime>, max: Option<&NaiveTime>) -> bool {
        Self::in_range_generic(value, min, max)
    }

    // Generic validations
    pub fn in_range_generic<T: PartialOrd>(value: &T, min: Option<&T>, max: Option<&T>) -> bool {
        if let Some(min_val) = min {
            if value < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if value > max_val {
                return false;
            }
        }
        true
    }

    pub fn is_within<T: PartialEq>(value: &T, list: &[T]) -> bool {
        list.contains(value)
    }

    pub fn is_excluded<T: PartialEq>(value: &T, list: &[T]) -> bool {
        !list.contains(value)
    }

    pub fn call_validator_func<T, F>(value: &T, validator: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        validator(value)
    }

    // Helper methods for validation logic
    pub fn validate_size(value: &str, min: Option<u64>, max: Option<u64>) -> bool {
        let len = value.len() as u64;
        if let Some(min_val) = min {
            if len < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if len > max_val {
                return false;
            }
        }
        true
    }

    pub fn validate_collection_size<T>(collection: &[T], min: Option<u64>, max: Option<u64>) -> bool {
        let len = collection.len() as u64;
        if let Some(min_val) = min {
            if len < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if len > max_val {
                return false;
            }
        }
        true
    }

    pub fn validate_hashset_size<T>(set: &HashSet<T>, min: Option<u64>, max: Option<u64>) -> bool {
        let len = set.len() as u64;
        if let Some(min_val) = min {
            if len < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if len > max_val {
                return false;
            }
        }
        true
    }

    pub fn validate_hashmap_size<K, V>(map: &HashMap<K, V>, min: Option<u64>, max: Option<u64>) -> bool {
        let len = map.len() as u64;
        if let Some(min_val) = min {
            if len < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if len > max_val {
                return false;
            }
        }
        true
    }
}