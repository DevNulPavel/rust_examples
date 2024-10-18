use crate::engine::all::MatchAllEngine;
use crate::engine::andor::{AndEngine, OrEngine};
use crate::engine::exact::{ExactEngine, ExactMatchingParam};
use crate::engine::fuzzy::{FuzzyAlgorithm, FuzzyEngine};
use crate::engine::regexp::RegexEngine;
use crate::{CaseMatching, MatchEngine, MatchEngineFactory};
use regex::Regex;

lazy_static! {
    static ref RE_AND: Regex = Regex::new(r"([^ |]+( +\| +[^ |]*)+)|( +)").unwrap();
    static ref RE_OR: Regex = Regex::new(r" +\| +").unwrap();
}
//------------------------------------------------------------------------------
// Exact engine factory
pub struct ExactOrFuzzyEngineFactory {
    exact_mode: bool,
    fuzzy_algorithm: FuzzyAlgorithm,
}

impl ExactOrFuzzyEngineFactory {
    pub fn builder() -> Self {
        Self {
            exact_mode: false,
            fuzzy_algorithm: FuzzyAlgorithm::SkimV2,
        }
    }

    pub fn exact_mode(mut self, exact_mode: bool) -> Self {
        self.exact_mode = exact_mode;
        self
    }

    pub fn fuzzy_algorithm(mut self, fuzzy_algorithm: FuzzyAlgorithm) -> Self {
        self.fuzzy_algorithm = fuzzy_algorithm;
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

impl MatchEngineFactory for ExactOrFuzzyEngineFactory {
    fn create_engine_with_case(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        // 'abc => match exact "abc"
        // ^abc => starts with "abc"
        // abc$ => ends with "abc"
        // ^abc$ => match exact "abc"
        // !^abc => items not starting with "abc"
        // !abc$ => items not ending with "abc"
        // !^abc$ => not "abc"

        let mut query = query;
        let mut exact = false;
        let mut param = ExactMatchingParam::default();
        param.case = case;

        if query.starts_with('\'') {
            if self.exact_mode {
                return Box::new(
                    FuzzyEngine::builder()
                        .query(&query[1..])
                        .algorithm(self.fuzzy_algorithm)
                        .case(case)
                        .build(),
                );
            } else {
                exact = true;
                query = &query[1..];
            }
        }

        if query.starts_with('!') {
            query = &query[1..];
            exact = true;
            param.inverse = true;
        }

        if query.is_empty() {
            // if only "!" was provided, will still show all items
            return Box::new(MatchAllEngine::builder().build());
        }

        if query.starts_with('^') {
            query = &query[1..];
            exact = true;
            param.prefix = true;
        }

        if query.ends_with('$') {
            query = &query[..(query.len() - 1)];
            exact = true;
            param.postfix = true;
        }

        if self.exact_mode {
            exact = true;
        }

        if exact {
            Box::new(ExactEngine::builder(query, param).build())
        } else {
            Box::new(
                FuzzyEngine::builder()
                    .query(query)
                    .algorithm(self.fuzzy_algorithm)
                    .case(case)
                    .build(),
            )
        }
    }
}

//------------------------------------------------------------------------------
pub struct AndOrEngineFactory {
    inner: Box<dyn MatchEngineFactory>,
}

impl AndOrEngineFactory {
    pub fn new(factory: impl MatchEngineFactory + 'static) -> Self {
        Self {
            inner: Box::new(factory),
        }
    }

    fn parse_or(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        if query.trim().is_empty() {
            self.inner.create_engine_with_case(query, case)
        } else {
            Box::new(
                OrEngine::builder()
                    .engines(RE_OR.split(query).map(|q| self.parse_and(q, case)).collect())
                    .build(),
            )
        }
    }

    fn parse_and(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        let query_trim = query.trim_matches(|c| c == ' ' || c == '|');
        let mut engines = vec![];
        let mut last = 0;
        for mat in RE_AND.find_iter(query_trim) {
            let (start, end) = (mat.start(), mat.end());
            let term = query_trim[last..start].trim_matches(|c| c == ' ' || c == '|');
            if !term.is_empty() {
                engines.push(self.inner.create_engine_with_case(term, case));
            }

            if !mat.as_str().trim().is_empty() {
                engines.push(self.parse_or(mat.as_str().trim(), case));
            }
            last = end;
        }

        let term = query_trim[last..].trim_matches(|c| c == ' ' || c == '|');
        if !term.is_empty() {
            engines.push(self.inner.create_engine_with_case(term, case));
        }
        Box::new(AndEngine::builder().engines(engines).build())
    }
}

impl MatchEngineFactory for AndOrEngineFactory {
    fn create_engine_with_case(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        self.parse_or(query, case)
    }
}

//------------------------------------------------------------------------------
pub struct RegexEngineFactory {}

impl RegexEngineFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl MatchEngineFactory for RegexEngineFactory {
    fn create_engine_with_case(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        Box::new(RegexEngine::builder(query, case).build())
    }
}

mod test {
    #[test]
    fn test_engine_factory() {
        use super::*;
        let exact_or_fuzzy = ExactOrFuzzyEngineFactory::builder().build();
        let x = exact_or_fuzzy.create_engine("'abc");
        assert_eq!(format!("{}", x), "(Exact|(?i)abc)");

        let x = exact_or_fuzzy.create_engine("^abc");
        assert_eq!(format!("{}", x), "(Exact|(?i)^abc)");

        let x = exact_or_fuzzy.create_engine("abc$");
        assert_eq!(format!("{}", x), "(Exact|(?i)abc$)");

        let x = exact_or_fuzzy.create_engine("^abc$");
        assert_eq!(format!("{}", x), "(Exact|(?i)^abc$)");

        let x = exact_or_fuzzy.create_engine("!abc");
        assert_eq!(format!("{}", x), "(Exact|!(?i)abc)");

        let x = exact_or_fuzzy.create_engine("!^abc");
        assert_eq!(format!("{}", x), "(Exact|!(?i)^abc)");

        let x = exact_or_fuzzy.create_engine("!abc$");
        assert_eq!(format!("{}", x), "(Exact|!(?i)abc$)");

        let x = exact_or_fuzzy.create_engine("!^abc$");
        assert_eq!(format!("{}", x), "(Exact|!(?i)^abc$)");

        let regex_factory = RegexEngineFactory::new();
        let and_or_factory = AndOrEngineFactory::new(exact_or_fuzzy);

        let x = and_or_factory.create_engine("'abc | def ^gh ij | kl mn");
        assert_eq!(
            format!("{}", x),
            "(Or: (And: (Exact|(?i)abc)), (And: (Fuzzy: def), (Exact|(?i)^gh), (Fuzzy: ij)), (And: (Fuzzy: kl), (Fuzzy: mn)))"
        );

        let x = regex_factory.create_engine("'abc | def ^gh ij | kl mn");
        assert_eq!(format!("{}", x), "(Regex: 'abc | def ^gh ij | kl mn)");
    }
}
