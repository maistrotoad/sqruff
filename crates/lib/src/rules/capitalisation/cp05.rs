use ahash::AHashMap;
use regex::Regex;
use sqruff_lib_core::dialects::syntax::{SyntaxKind, SyntaxSet};

use super::cp01::{handle_segment, RuleCP01};
use crate::core::config::Value;
use crate::core::rules::base::{Erased, ErasedRule, LintResult, Rule, RuleGroups};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{Crawler, SegmentSeekerCrawler};

#[derive(Debug, Clone)]
pub struct RuleCP05 {
    base: RuleCP01,
}

impl Default for RuleCP05 {
    fn default() -> Self {
        Self {
            base: RuleCP01 {
                skip_literals: false,
                exclude_parent_types: &[],
                ..Default::default()
            },
        }
    }
}

impl Rule for RuleCP05 {
    fn load_from_config(&self, config: &AHashMap<String, Value>) -> Result<ErasedRule, String> {
        Ok(RuleCP05 {
            base: RuleCP01 {
                capitalisation_policy: config["extended_capitalisation_policy"]
                    .as_string()
                    .unwrap()
                    .to_string(),
                description_elem: "Type names",
                ignore_words: config["ignore_words"]
                    .map(|it| {
                        it.as_array()
                            .unwrap()
                            .iter()
                            .map(|it| it.as_string().unwrap().to_lowercase())
                            .collect()
                    })
                    .unwrap_or_default(),
                ignore_words_regex: config["ignore_words_regex"]
                    .map(|it| {
                        it.as_array()
                            .unwrap()
                            .iter()
                            .map(|it| Regex::new(it.as_string().unwrap()).unwrap())
                            .collect()
                    })
                    .unwrap_or_default(),

                ..Default::default()
            },
        }
        .erased())
    }

    fn name(&self) -> &'static str {
        "capitalisation.types"
    }

    fn description(&self) -> &'static str {
        "Inconsistent capitalisation of datatypes."
    }

    fn long_description(&self) -> &'static str {
        r#"
**Anti-pattern**

In this example, `int` and `unsigned` are in lower-case whereas `VARCHAR` is in upper-case.

```sql
CREATE TABLE t (
    a int unsigned,
    b VARCHAR(15)
);
```

**Best practice**

Ensure all datatypes are consistently upper or lower case

```sql
CREATE TABLE t (
    a INT UNSIGNED,
    b VARCHAR(15)
);
```
"#
    }

    fn groups(&self) -> &'static [RuleGroups] {
        &[
            RuleGroups::All,
            RuleGroups::Core,
            RuleGroups::Capitalisation,
        ]
    }

    fn eval(&self, context: RuleContext) -> Vec<LintResult> {
        let mut results = Vec::new();

        if self
            .base
            .ignore_words
            .contains(&context.segment.raw().to_lowercase())
        {
            return Vec::new();
        }

        if self
            .base
            .ignore_words_regex
            .iter()
            .any(|regex| regex.is_match(context.segment.raw().as_ref()))
        {
            return Vec::new();
        }

        if context.segment.is_type(SyntaxKind::PrimitiveType)
            || context.segment.is_type(SyntaxKind::DatetimeTypeIdentifier)
            || context.segment.is_type(SyntaxKind::DataType)
        {
            for seg in context.segment.segments() {
                if seg.is_type(SyntaxKind::Symbol)
                    || seg.is_type(SyntaxKind::Identifier)
                    || seg.is_type(SyntaxKind::QuotedLiteral)
                    || !seg.segments().is_empty()
                {
                    continue;
                }

                results.push(handle_segment(
                    "Datatypes",
                    &self.base.capitalisation_policy,
                    "extended_capitalisation_policy",
                    seg.clone(),
                    &context,
                ));
            }
        }

        results
    }

    fn is_fix_compatible(&self) -> bool {
        true
    }

    fn crawl_behaviour(&self) -> Crawler {
        SegmentSeekerCrawler::new(
            const {
                SyntaxSet::new(&[
                    SyntaxKind::DataTypeIdentifier,
                    SyntaxKind::PrimitiveType,
                    SyntaxKind::DatetimeTypeIdentifier,
                    SyntaxKind::DataType,
                ])
            },
        )
        .into()
    }
}
