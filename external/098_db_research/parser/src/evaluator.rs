use crate::ast::{Comparator, Expr};
use serde_json::Value;

pub type UserMeta = Value;

#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    pub is_match: bool,
    pub wake_up_at: Option<u64>,
}

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(expr: &Expr, meta: &UserMeta, now_ts: u64) -> EvalResult {
        match expr {
            Expr::Empty => EvalResult {
                is_match: true,
                wake_up_at: None,
            },

            Expr::Any(path, inner_expr) => {
                let mut any_match = false;
                let mut max_true_wakeup: Option<u64> = None;
                let mut min_false_wakeup: Option<u64> = None;
                let mut all_true_have_wakeup = true;

                if let Some(Value::Array(arr)) = Self::get_nested(meta, path) {
                    for item in arr {
                        let res = Self::evaluate(inner_expr, item, now_ts);
                        if res.is_match {
                            any_match = true;
                            if let Some(wake_up_at) = res.wake_up_at
                                && all_true_have_wakeup
                            {
                                max_true_wakeup =
                                    Some(std::cmp::max(max_true_wakeup.unwrap_or(0), wake_up_at));
                            } else {
                                all_true_have_wakeup = false;
                            }
                        } else {
                            min_false_wakeup = match (min_false_wakeup, res.wake_up_at) {
                                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            };
                        }
                    }
                }

                if any_match {
                    EvalResult {
                        is_match: true,
                        wake_up_at: if all_true_have_wakeup {
                            max_true_wakeup
                        } else {
                            None
                        },
                    }
                } else {
                    EvalResult {
                        is_match: false,
                        wake_up_at: min_false_wakeup,
                    }
                }
            }

            Expr::All(path, inner_expr) => {
                let mut all_match = true;
                let mut min_true_wakeup: Option<u64> = None;
                let mut max_false_wakeup: Option<u64> = None;
                let mut all_false_have_wakeup = true;
                let mut is_array = false;

                if let Some(Value::Array(arr)) = Self::get_nested(meta, path) {
                    is_array = true;
                    for item in arr {
                        let res = Self::evaluate(inner_expr, item, now_ts);
                        if !res.is_match {
                            all_match = false;
                            if let Some(wake_up_at) = res.wake_up_at
                                && all_false_have_wakeup
                            {
                                max_false_wakeup =
                                    Some(std::cmp::max(max_false_wakeup.unwrap_or(0), wake_up_at));
                            } else {
                                all_false_have_wakeup = false;
                            }
                        } else {
                            min_true_wakeup = match (min_true_wakeup, res.wake_up_at) {
                                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            };
                        }
                    }
                }

                if !is_array {
                    return EvalResult {
                        is_match: false,
                        wake_up_at: None,
                    };
                }

                if all_match {
                    EvalResult {
                        is_match: true,
                        wake_up_at: min_true_wakeup,
                    }
                } else {
                    EvalResult {
                        is_match: false,
                        wake_up_at: if all_false_have_wakeup {
                            max_false_wakeup
                        } else {
                            None
                        },
                    }
                }
            }

            Expr::LenComp(path, comp, expected_len) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(Value::String(s)) => {
                        Self::compare_f64(s.chars().count() as f64, comp, *expected_len as f64)
                    }
                    Some(Value::Array(arr)) => {
                        Self::compare_f64(arr.len() as f64, comp, *expected_len as f64)
                    }
                    Some(Value::Object(obj)) => {
                        Self::compare_f64(obj.len() as f64, comp, *expected_len as f64)
                    }
                    _ => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::BitComp(path, comp, mask, expected) => {
                let is_match = match Self::extract_timestamp(meta, path) {
                    Some(val) => {
                        let bit_result = val & mask;
                        Self::compare_f64(bit_result as f64, comp, *expected as f64)
                    }
                    None => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::Flag(path) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(Value::Bool(b)) => *b,
                    _ => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::Exists(path) => {
                let exists = Self::get_nested(meta, path).is_some_and(|v| !v.is_null());
                EvalResult {
                    is_match: exists,
                    wake_up_at: None,
                }
            }

            Expr::NotExists(path) => {
                let not_exists = Self::get_nested(meta, path).is_none_or(|v| v.is_null());
                EvalResult {
                    is_match: not_exists,
                    wake_up_at: None,
                }
            }

            Expr::Cmp(path, comp, target_val) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(actual_val) => Self::compare_json(actual_val, comp, target_val),
                    None => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::In(path, list) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(Value::Array(arr)) => arr.iter().any(|item| list.contains(item)),
                    Some(val) => list.contains(val),
                    None => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::AgeComp(path, comp, duration) => {
                let ts = match Self::extract_timestamp(meta, path) {
                    Some(val) => val,
                    None => {
                        return EvalResult {
                            is_match: false,
                            wake_up_at: None,
                        };
                    }
                };
                let age = now_ts.saturating_sub(ts);
                let is_match = Self::compare_f64(age as f64, comp, *duration as f64);

                let wake_up_at = if !is_match
                    && (*comp == Comparator::Greater || *comp == Comparator::GreaterOrEq)
                {
                    // Ложно сейчас, но станет истинным, когда пройдет достаточно времени
                    let target = ts + duration + if *comp == Comparator::Greater { 1 } else { 0 };
                    if target > now_ts { Some(target) } else { None }
                } else if is_match && (*comp == Comparator::Less || *comp == Comparator::LessOrEq) {
                    // Истинно сейчас, но перестанет быть истинным (протухнет) в будущем
                    let target = ts + duration + if *comp == Comparator::Less { 0 } else { 1 };
                    if target > now_ts { Some(target) } else { None }
                } else {
                    None
                };

                EvalResult {
                    is_match,
                    wake_up_at,
                }
            }

            Expr::TimeCompNow(path, comp) => {
                let ts = match Self::extract_timestamp(meta, path) {
                    Some(val) => val,
                    None => {
                        return EvalResult {
                            is_match: false,
                            wake_up_at: None,
                        };
                    }
                };
                let is_match = Self::compare_f64(ts as f64, comp, now_ts as f64);

                let wake_up_at =
                    if !is_match && (*comp == Comparator::Less || *comp == Comparator::LessOrEq) {
                        // Сейчас False. Станет True, когда NOW() догонит ts.
                        Some(ts + if *comp == Comparator::Less { 1 } else { 0 })
                    } else if is_match
                        && (*comp == Comparator::Greater || *comp == Comparator::GreaterOrEq)
                    {
                        // Сейчас True. Протухнет (False), когда NOW() догонит ts.
                        Some(ts + if *comp == Comparator::Greater { 0 } else { 1 })
                    } else {
                        None
                    };

                EvalResult {
                    is_match,
                    wake_up_at,
                }
            }

            Expr::ModComp(path, comp, divisor, remainder) => {
                let is_match = match Self::extract_timestamp(meta, path) {
                    Some(val) => {
                        let mod_result = val % divisor;
                        Self::compare_f64(mod_result as f64, comp, *remainder as f64)
                    }
                    None => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::Not(inner) => {
                let res = Self::evaluate(inner, meta, now_ts);
                EvalResult {
                    is_match: !res.is_match,
                    wake_up_at: res.wake_up_at, // Если перевернется внутреннее, перевернется и результат NOT
                }
            }

            Expr::And(left, right) => {
                let l = Self::evaluate(left, meta, now_ts);

                // Безопасное раннее прерывание: если ложь и никогда не станет правдой
                if !l.is_match && l.wake_up_at.is_none() {
                    return EvalResult {
                        is_match: false,
                        wake_up_at: None,
                    };
                }

                let r = Self::evaluate(right, meta, now_ts);

                match (l.is_match, r.is_match) {
                    (true, true) => EvalResult {
                        is_match: true,
                        wake_up_at: match (l.wake_up_at, r.wake_up_at) {
                            (Some(a), Some(b)) => Some(std::cmp::min(a, b)), // Протухнет, когда протухнет самое ближайшее
                            (Some(a), None) => Some(a),
                            (None, Some(b)) => Some(b),
                            (None, None) => None,
                        },
                    },
                    (true, false) => EvalResult {
                        is_match: false,
                        wake_up_at: r.wake_up_at,
                    },
                    (false, true) => EvalResult {
                        is_match: false,
                        wake_up_at: l.wake_up_at,
                    },
                    (false, false) => {
                        let wake_up_at = match (l.wake_up_at, r.wake_up_at) {
                            (Some(a), Some(b)) => Some(std::cmp::max(a, b)), // Станет истинным, когда И левое И правое станут истинными
                            _ => None,
                        };
                        EvalResult {
                            is_match: false,
                            wake_up_at,
                        }
                    }
                }
            }

            Expr::Or(left, right) => {
                let l = Self::evaluate(left, meta, now_ts);

                // Безопасное раннее прерывание: если истина и НИКОГДА не протухнет
                if l.is_match && l.wake_up_at.is_none() {
                    return EvalResult {
                        is_match: true,
                        wake_up_at: None,
                    };
                }

                let r = Self::evaluate(right, meta, now_ts);

                match (l.is_match, r.is_match) {
                    (true, true) => EvalResult {
                        is_match: true,
                        wake_up_at: match (l.wake_up_at, r.wake_up_at) {
                            (Some(a), Some(b)) => Some(std::cmp::max(a, b)), // Протухнет только когда ОБА перестанут быть истинными
                            _ => None,
                        },
                    },
                    (true, false) => EvalResult {
                        is_match: true,
                        wake_up_at: l.wake_up_at,
                    },
                    (false, true) => EvalResult {
                        is_match: true,
                        wake_up_at: r.wake_up_at,
                    },
                    (false, false) => {
                        let wake_up_at = match (l.wake_up_at, r.wake_up_at) {
                            (Some(a), Some(b)) => Some(std::cmp::min(a, b)), // Станет истинным, когда хотя бы ОДНО станет истинным
                            (Some(a), None) => Some(a),
                            (None, Some(b)) => Some(b),
                            (None, None) => None,
                        };
                        EvalResult {
                            is_match: false,
                            wake_up_at,
                        }
                    }
                }
            }
            Expr::Contains(path, substring) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(Value::String(s)) => s.contains(substring),
                    _ => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }

            Expr::Icontains(path, substring) => {
                let is_match = match Self::get_nested(meta, path) {
                    Some(Value::String(s)) => s.to_lowercase().contains(&substring.to_lowercase()),
                    _ => false,
                };
                EvalResult {
                    is_match,
                    wake_up_at: None,
                }
            }
        }
    }

    fn get_nested<'a>(meta: &'a UserMeta, path: &[String]) -> Option<&'a Value> {
        let mut current = meta;
        for key in path {
            current = current.get(key)?;
        }
        Some(current)
    }

    fn extract_timestamp(meta: &UserMeta, path: &[String]) -> Option<u64> {
        match Self::get_nested(meta, path) {
            Some(Value::Number(n)) => n.as_u64().or_else(|| n.as_f64().map(|f| f.max(0.0) as u64)),
            _ => None,
        }
    }

    fn compare_json(actual: &Value, comp: &Comparator, target: &Value) -> bool {
        match (actual, target) {
            (Value::Number(a), Value::Number(b)) => {
                Self::compare_f64(a.as_f64().unwrap_or(0.0), comp, b.as_f64().unwrap_or(0.0))
            }
            (Value::String(a), Value::String(b)) => match comp {
                Comparator::Eq => a == b,
                Comparator::NotEq => a != b,
                Comparator::Greater => a > b,
                Comparator::Less => a < b,
                _ => false,
            },
            (Value::Bool(a), Value::Bool(b)) => match comp {
                Comparator::Eq => a == b,
                Comparator::NotEq => a != b,
                _ => false,
            },
            (Value::Null, Value::Null) => matches!(
                comp,
                Comparator::Eq | Comparator::LessOrEq | Comparator::GreaterOrEq
            ),
            _ => false,
        }
    }

    fn compare_f64(a: f64, comp: &Comparator, b: f64) -> bool {
        match comp {
            Comparator::Less => a < b,
            Comparator::LessOrEq => a <= b,
            Comparator::Greater => a > b,
            Comparator::GreaterOrEq => a >= b,
            Comparator::Eq => (a - b).abs() < f64::EPSILON,
            Comparator::NotEq => (a - b).abs() >= f64::EPSILON,
        }
    }
}
