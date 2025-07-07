use dashmap::DashMap;
use pyo3::pyclass;
use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Deserialize, Serialize};
use radix_trie::Trie;

// Additional constants
pub(crate) const MAGIC_CHAR: char = '\u{1E00}'; // Unicode for 'Ḁ' (Latin Capital Letter A with Ring Below)
pub(crate) const MAGIC_CHAR_STRING: &str = "\u{1E00}"; // String representation of 'Ḁ'
pub(crate) const QUOTE_CHAR_ARRAY: &[char] = &['"'];

pub type StringToken = u32;
pub type StringLowerToken = u32;

/// A token pair: one token for the normalized (lower-case, unquoted) string,
/// and one token for the original string. The `quoted` flag indicates whether
/// the original string was wrapped in quotes.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct StringTokens {
    pub lower: StringLowerToken,
    pub normal: StringToken,
    pub quoted: bool,
}

impl StringTokens {
    pub fn new(lower: StringLowerToken, normal: StringToken, quoted: bool) -> Self {
        Self { lower, normal, quoted }
    }
}

/// Holds metadata computed from a string’s lower-case version.
#[derive(Debug, Clone)]
pub struct StringMetadata {
    pub starts_with_amp: bool,
    pub contains_double_dollar: bool,
    pub contains_question_mark: bool,
    pub contains_hat: bool,
    pub starts_with_square_bracket: bool,
    pub contains_pipe: bool,
}

impl StringMetadata {
    pub fn from_string(s: &str) -> Self {
        let starts_with_amp = s.starts_with('@');
        let contains_double_dollar = s.matches('$').count() > 1;
        let contains_question_mark = s.contains('?');
        let contains_hat = s.contains('^');
        let starts_with_square_bracket = s.starts_with('[');
        let contains_pipe = s.contains('|');

        Self {
            starts_with_amp,
            contains_double_dollar,
            contains_question_mark,
            contains_hat,
            starts_with_square_bracket,
            contains_pipe,
        }
    }
}


/// A manager for interning strings along with metadata.
#[pyclass]
#[derive(Debug)]
pub struct StringResourceManager {
    strings: DashMap<String, StringTokens>,
    ints: DashMap<StringToken, String>,
    metadata: DashMap<StringToken, StringMetadata>,
    counter: AtomicU32,
}

impl Default for StringResourceManager {
    fn default() -> Self {
        Self {
            strings: DashMap::new(),
            ints: DashMap::new(),
            metadata: DashMap::new(),
            counter: AtomicU32::new(0),
        }
    }
}

impl StringResourceManager {
    /// Creates a new `StringResourceManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns the given string and returns its tokens.
    pub fn intern_identifier_token(&self, s: &str) -> StringTokens {
        // Check if the string is already interned.
        if let Some(tokens) = self.strings.get(s) {
            return tokens.clone();
        }

        // Normalize the string.
        let (ls_trimmed, quoted) = Self::normalize_string(s);

        // Check if the normalized version is already interned.
        if let Some(lower_token) = self.strings.get(&ls_trimmed).map(|t| t.lower) {
            let string_id = self.counter.fetch_add(1, Ordering::SeqCst);
            let new_tokens = StringTokens::new(lower_token, string_id, quoted);

            self.ints.insert(string_id, s.to_string());
            if let Some(meta) = self.metadata.get(&lower_token).map(|ref_meta| ref_meta.clone()) {
                self.metadata.insert(string_id, meta);
            }
            self.strings.insert(s.to_string(), new_tokens.clone());
            return new_tokens;
        }

        // Neither the string nor its normalized version is interned.
        let string_id = self.counter.fetch_add(1, Ordering::SeqCst);
        let low_id = self.counter.fetch_add(1, Ordering::SeqCst);

        let tokens = StringTokens::new(low_id, string_id, quoted);
        let tokens_lower = StringTokens::new(low_id, low_id, false);

        // Compute metadata from the normalized string.
        let meta = StringMetadata::from_string(&ls_trimmed);

        // Store metadata and strings.
        self.metadata.insert(low_id, meta.clone());
        self.metadata.insert(string_id, meta);
        self.ints.insert(low_id, ls_trimmed.clone());
        self.ints.insert(string_id, s.to_string());
        self.strings.insert(ls_trimmed, tokens_lower);
        self.strings.insert(s.to_string(), tokens.clone());

        tokens
    }

    /// Returns the original string for the token stored in the `normal` field.
    pub fn get_string_for_ids(&self, tokens: &StringTokens) -> Option<String> {
        self.ints.get(&tokens.normal).map(|v| v.clone())
    }

    /// Returns the lower-case string for the token stored in the `lower` field.
    pub fn get_lower_string_for_ids(&self, tokens: &StringTokens) -> Option<String> {
        self.ints.get(&tokens.lower).map(|v| v.clone())
    }

    /// Returns the string for a specific token.
    pub fn get_string_for_id(&self, id: StringToken) -> Option<String> {
        self.ints.get(&id).map(|v| v.clone())
    }

    /// Returns the metadata associated with a specific token.
    pub fn get_metadata_for_id(&self, id: StringToken) -> Option<StringMetadata> {
        self.metadata.get(&id).map(|v| v.clone())
    }

    /// Normalizes a string by converting it to lowercase and trimming quotes.
    fn normalize_string(s: &str) -> (String, bool) {
        let ls_trimmed = s.to_lowercase().trim_matches('"').to_string();
        let quoted = s.starts_with('"') && s.ends_with('"');
        (ls_trimmed, quoted)
    }
}



#[derive(Debug, Default)]
pub struct PrefixOptimisedStringSet {
    trie: Trie<String, String>,
    id_values: Vec<StringTokens>,
}

impl PrefixOptimisedStringSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_with_ids(&mut self, key: &str, string_manager: &StringResourceManager) {
        let lower_key = key.to_lowercase();
        self.trie.insert(lower_key.clone(), key.to_string());
        let token = string_manager.intern_identifier_token(key);
        self.id_values.push(token);
    }

    pub fn count(&self) -> usize {
        self.id_values.len()
    }

    pub fn id_values(&self) -> &[StringTokens] {
        &self.id_values
    }

    pub fn string_values(&self, string_manager: &StringResourceManager) -> Vec<Option<String>> {
        self.id_values
            .iter()
            .map(|token| string_manager.get_string_for_ids(token))
            .collect()
    }

    pub fn contains_prefix(&self, prefix: &str) -> bool {
        let prefix = prefix.to_lowercase();
        self.trie.subtrie(&prefix).is_some()
    }
}
