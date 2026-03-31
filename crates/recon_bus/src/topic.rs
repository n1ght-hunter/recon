//! Topic parsing, validation, and wildcard matching.

use std::{fmt, sync::Arc};

/// A topic path with `/`-separated segments.
///
/// Supports wildcards for subscription filters:
/// - `*` matches exactly one segment
/// - `**` matches zero or more remaining segments (must be last)
///
/// Separator indices are cached at construction so segment access never allocates.
#[derive(Debug, Clone)]
pub struct Topic {
    raw: Arc<str>,
    separators: Arc<[usize]>,
}

impl Topic {
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn segment_count(&self) -> usize {
        self.separators.len() + 1
    }

    pub fn segment(&self, index: usize) -> &str {
        let start = if index == 0 {
            0
        } else {
            self.separators[index - 1] + 1
        };
        let end = self
            .separators
            .get(index)
            .copied()
            .unwrap_or(self.raw.len());
        &self.raw[start..end]
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        (0..self.segment_count()).map(|i| self.segment(i))
    }

    pub fn as_raw(&self) -> &Arc<str> {
        &self.raw
    }

    pub fn into_arc(self) -> Arc<str> {
        self.raw
    }

    pub fn has_wildcards(&self) -> bool {
        self.raw.contains('*')
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        *self.raw == *other.raw
    }
}

impl Eq for Topic {}

impl std::hash::Hash for Topic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopicError {
    Empty,
    EmptySegment,
    WildcardMixedWithText,
    MultiWildcardNotLast,
}

impl fmt::Display for TopicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "topic must not be empty"),
            Self::EmptySegment => write!(f, "topic must not contain empty segments"),
            Self::MultiWildcardNotLast => write!(f, "'**' must be the last segment"),
            Self::WildcardMixedWithText => {
                write!(f, "'*' and '**' must be the entire segment")
            }
        }
    }
}

impl std::error::Error for TopicError {}

impl TryFrom<&str> for Topic {
    type Error = TopicError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(TopicError::Empty);
        }
        let separators: Arc<[usize]> = s
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| (b == b'/').then_some(i))
            .collect();
        let count = separators.len() + 1;

        (0..count).try_for_each(|i| {
            let start = if i == 0 { 0 } else { separators[i - 1] + 1 };
            let end = separators.get(i).copied().unwrap_or(s.len());
            let seg = &s[start..end];

            if seg.is_empty() {
                return Err(TopicError::EmptySegment);
            }
            if seg == "*" || seg == "**" {
                if seg == "**" && i != count - 1 {
                    return Err(TopicError::MultiWildcardNotLast);
                }
                return Ok(());
            }
            if seg.contains('*') {
                return Err(TopicError::WildcardMixedWithText);
            }
            Ok(())
        })?;

        Ok(Self {
            raw: Arc::from(s),
            separators,
        })
    }
}

impl TryFrom<String> for Topic {
    type Error = TopicError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Check if a filter topic matches a concrete topic.
pub fn topic_matches(filter: &Topic, topic: &Topic) -> bool {
    let mut fi = 0;
    let mut ti = 0;
    let fc = filter.segment_count();
    let tc = topic.segment_count();

    while fi < fc {
        match filter.segment(fi) {
            "**" => return true,
            "*" => {
                if ti >= tc {
                    return false;
                }
                fi += 1;
                ti += 1;
            }
            literal => {
                if ti >= tc || topic.segment(ti) != literal {
                    return false;
                }
                fi += 1;
                ti += 1;
            }
        }
    }

    ti == tc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_topics() {
        assert!(Topic::try_from("game/valorant/status").is_ok());
        assert!(Topic::try_from("a").is_ok());
        assert!(Topic::try_from("a/b/c/d").is_ok());
        assert!(Topic::try_from("game/*/status").is_ok());
        assert!(Topic::try_from("game/**").is_ok());
        assert!(Topic::try_from("**").is_ok());
        assert!(Topic::try_from("*").is_ok());
        assert!(Topic::try_from("*/*/*").is_ok());
    }

    #[test]
    fn invalid_topics() {
        assert_eq!(Topic::try_from(""), Err(TopicError::Empty));
        assert_eq!(Topic::try_from("a//b"), Err(TopicError::EmptySegment));
        assert_eq!(
            Topic::try_from("a/**/b"),
            Err(TopicError::MultiWildcardNotLast)
        );
        assert_eq!(
            Topic::try_from("a/b*c"),
            Err(TopicError::WildcardMixedWithText)
        );
    }

    #[test]
    fn segment_indexing() {
        let t = Topic::try_from("game/valorant/status").unwrap();
        assert_eq!(t.segment_count(), 3);
        assert_eq!(t.segment(0), "game");
        assert_eq!(t.segment(1), "valorant");
        assert_eq!(t.segment(2), "status");
    }

    #[test]
    fn single_segment() {
        let t = Topic::try_from("hello").unwrap();
        assert_eq!(t.segment_count(), 1);
        assert_eq!(t.segment(0), "hello");
    }

    #[test]
    fn has_wildcards() {
        assert!(!Topic::try_from("game/valorant").unwrap().has_wildcards());
        assert!(Topic::try_from("game/*").unwrap().has_wildcards());
        assert!(Topic::try_from("game/**").unwrap().has_wildcards());
    }

    #[test]
    fn exact_match() {
        let f = Topic::try_from("game/valorant/status").unwrap();
        let t = Topic::try_from("game/valorant/status").unwrap();
        assert!(topic_matches(&f, &t));
    }

    #[test]
    fn exact_no_match() {
        let f = Topic::try_from("game/valorant/status").unwrap();
        let t = Topic::try_from("game/apex/status").unwrap();
        assert!(!topic_matches(&f, &t));
    }

    #[test]
    fn single_wildcard() {
        let f = Topic::try_from("game/*/status").unwrap();
        assert!(topic_matches(
            &f,
            &Topic::try_from("game/valorant/status").unwrap()
        ));
        assert!(topic_matches(
            &f,
            &Topic::try_from("game/apex/status").unwrap()
        ));
        assert!(!topic_matches(
            &f,
            &Topic::try_from("game/valorant/health").unwrap()
        ));
        assert!(!topic_matches(&f, &Topic::try_from("game/status").unwrap()));
    }

    #[test]
    fn multi_wildcard() {
        let f = Topic::try_from("game/**").unwrap();
        assert!(topic_matches(
            &f,
            &Topic::try_from("game/valorant").unwrap()
        ));
        assert!(topic_matches(
            &f,
            &Topic::try_from("game/valorant/status").unwrap()
        ));
        assert!(topic_matches(&f, &Topic::try_from("game/a/b/c/d").unwrap()));
        assert!(!topic_matches(&f, &Topic::try_from("other/thing").unwrap()));
    }

    #[test]
    fn double_star_matches_zero_segments() {
        let f = Topic::try_from("game/**").unwrap();
        assert!(topic_matches(&f, &Topic::try_from("game").unwrap()));
    }

    #[test]
    fn root_double_star_matches_everything() {
        let f = Topic::try_from("**").unwrap();
        assert!(topic_matches(&f, &Topic::try_from("anything").unwrap()));
        assert!(topic_matches(&f, &Topic::try_from("a/b/c").unwrap()));
    }

    #[test]
    fn length_mismatch() {
        let f = Topic::try_from("a/b").unwrap();
        assert!(!topic_matches(&f, &Topic::try_from("a/b/c").unwrap()));
        assert!(!topic_matches(&f, &Topic::try_from("a").unwrap()));
    }
}
