use std::fmt::{Display, Error, Formatter};
use std::sync::Arc;

use fuzzy_matcher::clangd::ClangdMatcher;
use fuzzy_matcher::skim::{SkimMatcher, SkimMatcherV2};
use fuzzy_matcher::FuzzyMatcher;

use crate::item::{MatchedItem, MatchedRange, Rank};
use crate::SkimItem;
use crate::{CaseMatching, MatchEngine};

//------------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum FuzzyAlgorithm {
    SkimV1,
    SkimV2,
    Clangd,
}

impl FuzzyAlgorithm {
    pub fn of(algorithm: &str) -> Self {
        match algorithm.to_ascii_lowercase().as_ref() {
            "skim_v1" => FuzzyAlgorithm::SkimV1,
            "skim_v2" | "skim" => FuzzyAlgorithm::SkimV2,
            "clangd" => FuzzyAlgorithm::Clangd,
            _ => FuzzyAlgorithm::SkimV2,
        }
    }
}

impl Default for FuzzyAlgorithm {
    fn default() -> Self {
        FuzzyAlgorithm::SkimV2
    }
}

const BYTES_1M: usize = 1024 * 1024 * 1024;

//------------------------------------------------------------------------------
// Fuzzy engine
#[derive(Default)]
pub struct FuzzyEngineBuilder {
    query: String,
    case: CaseMatching,
    algorithm: FuzzyAlgorithm,
}

impl FuzzyEngineBuilder {
    pub fn query(mut self, query: &str) -> Self {
        self.query = query.to_string();
        self
    }

    pub fn case(mut self, case: CaseMatching) -> Self {
        self.case = case;
        self
    }

    pub fn algorithm(mut self, algorithm: FuzzyAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn build(self) -> FuzzyEngine {
        let matcher: Box<dyn FuzzyMatcher> = match self.algorithm {
            FuzzyAlgorithm::SkimV1 => Box::new(SkimMatcher::default()),
            FuzzyAlgorithm::SkimV2 => {
                let matcher = SkimMatcherV2::default().element_limit(BYTES_1M);
                let matcher = match self.case {
                    CaseMatching::Respect => matcher.respect_case(),
                    CaseMatching::Ignore => matcher.ignore_case(),
                    CaseMatching::Smart => matcher.smart_case(),
                };
                Box::new(matcher)
            }
            FuzzyAlgorithm::Clangd => {
                let matcher = ClangdMatcher::default();
                let matcher = match self.case {
                    CaseMatching::Respect => matcher.respect_case(),
                    CaseMatching::Ignore => matcher.ignore_case(),
                    CaseMatching::Smart => matcher.smart_case(),
                };
                Box::new(matcher)
            }
        };

        FuzzyEngine {
            matcher,
            query: self.query,
        }
    }
}

pub struct FuzzyEngine {
    query: String,
    matcher: Box<dyn FuzzyMatcher>,
}

impl FuzzyEngine {
    pub fn builder() -> FuzzyEngineBuilder {
        FuzzyEngineBuilder::default()
    }

    fn fuzzy_match(&self, choice: &str, pattern: &str) -> Option<(i64, Vec<usize>)> {
        if pattern.is_empty() {
            return Some((0, Vec::new()));
        } else if choice.is_empty() {
            return None;
        }

        self.matcher.fuzzy_indices(choice, pattern)
    }
}

impl MatchEngine for FuzzyEngine {
    fn match_item(&self, item: Arc<dyn SkimItem>) -> Option<MatchedItem> {
        // iterate over all matching fields:
        let mut matched_result = None;
        let item_text = item.text();
        let default_range = [(0, item_text.len())];
        for &(start, end) in item.get_matching_ranges().unwrap_or(&default_range) {
            matched_result = self.fuzzy_match(&item.text()[start..end], &self.query).map(|(s, vec)| {
                if start != 0 {
                    let start_char = &item_text[..start].chars().count();
                    (s, vec.iter().map(|x| x + start_char).collect())
                } else {
                    (s, vec)
                }
            });

            if matched_result.is_some() {
                break;
            }
        }

        if matched_result == None {
            return None;
        }

        let (score, matched_range) = matched_result.unwrap();

        let begin = *matched_range.get(0).unwrap_or(&0) as i64;
        let end = *matched_range.last().unwrap_or(&0) as i64;

        let rank = Rank {
            score: -score,
            begin,
            end,
        };

        Some(
            MatchedItem::builder(item)
                .rank(rank)
                .matched_range(MatchedRange::Chars(matched_range))
                .build(),
        )
    }
}

impl Display for FuzzyEngine {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "(Fuzzy: {})", self.query)
    }
}
