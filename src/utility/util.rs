use std::{collections::HashMap, fmt, path::Display, sync::{Arc, Mutex}};
use once_cell::sync::Lazy;

// Additional constants
pub(crate) const MAGIC_CHAR: char = '\u{1E00}'; // Unicode for 'Ḁ' (Latin Capital Letter A with Ring Below)
pub(crate) const MAGIC_CHAR_STRING: &str = "\u{1E00}"; // String representation of 'Ḁ'
pub(crate) const QUOTE_CHAR_ARRAY: &[char] = &['"'];

// Type aliases matching F# definitions.
pub type StringToken = u32;
pub type StringLowerToken = u32;

/// A token pair: one token for the normalized (lower-case, unquoted) string,
/// and one token for the original string. The `quoted` flag indicates whether
/// the original string was wrapped in quotes.
#[derive(Clone, PartialEq, Eq, Hash)]
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

impl std::fmt::Debug for StringTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringTokens(\nlower: {},\nnormal: {},\nquoted: {},\nThis StringToken refers to string: {}\n)", self.lower, self.normal, self.quoted, self.get_string().unwrap_or("".to_string()))
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
    pub fn new(
        starts_with_amp: bool,
        contains_double_dollar: bool,
        contains_question_mark: bool,
        contains_hat: bool,
        starts_with_square_bracket: bool,
        contains_pipe: bool,
    ) -> Self {
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

/// Internal state for the resource manager. In F#, these were separate
/// ConcurrentDictionaries. Here we bundle them together and protect
/// the entire state with a Mutex.
struct Inner {
    /// Maps an original string (or normalized string) to its interned tokens.
    strings: HashMap<String, StringTokens>,
    /// Maps a token to the actual string.
    ints: HashMap<StringToken, String>,
    /// Maps a token to its computed metadata.
    metadata: HashMap<StringToken, StringMetadata>,
    /// Counter used to generate unique token values.
    counter: StringToken,
}

/// A manager for interning strings along with metadata. Its behavior tries to
/// match the F# implementation.
pub struct StringResourceManager {
    inner: Mutex<Inner>,
}

impl StringResourceManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                strings: HashMap::new(),
                ints: HashMap::new(),
                metadata: HashMap::new(),
                counter: 0,
            }),
        }
    }

    /// Interns the given string and returns its tokens. The logic is as follows:
    ///
    /// 1. If `s` is already interned, return its tokens.
    /// 2. Otherwise, create a lower-case, trimmed version (`ls_trimmed`) of the
    ///    string. Also determine whether the string is quoted.
    /// 3. If the lower-case version is already interned, create a new token that
    ///    reuses the lower-case token.
    /// 4. Otherwise, create new tokens for both the lower-case version and the
    ///    original string. Compute and store metadata based on the lower-case string.
    pub fn intern_identifier_token(&self, s: &str) -> StringTokens {
        // First check without locking (or with minimal lock) if we already have s.
        {
            let inner = self.inner.lock().unwrap();
            if let Some(tokens) = inner.strings.get(s) {
                return tokens.clone();
            }
        }

        // Now grab the lock for the remainder of the operation.
        let mut inner = self.inner.lock().unwrap();

        // Double-check in case another thread interened the same string.
        if let Some(tokens) = inner.strings.get(s) {
            return tokens.clone();
        }

        // Compute normalized version.
        let ls = s.to_lowercase();
        let ls_trimmed = ls.trim_matches('"').to_string();
        let quoted = s.starts_with('"') && s.ends_with('"');

        let existing_lower_token = inner.strings.get(&ls_trimmed).map(|t| t.lower);

        if let Some(lower_token) = existing_lower_token {
            let string_id = inner.counter;
            inner.counter += 1;
        
            let new_tokens = StringTokens::new(lower_token, string_id, quoted);
        
            inner.ints.insert(string_id, s.to_string());
        
            if let Some(meta) = inner.metadata.get(&lower_token).cloned() {
                inner.metadata.insert(string_id, meta);
            }
        
            inner.strings.insert(s.to_string(), new_tokens.clone());
            return new_tokens;
        } else {
            // Neither s nor its normalized version have been interned.
            // Reserve two tokens.
            let string_id = inner.counter;
            let low_id = inner.counter + 1;
            inner.counter += 2;
            let tokens = StringTokens::new(low_id, string_id, quoted);
            let tokens_lower = StringTokens::new(low_id, low_id, false);

            // Compute metadata from the normalized string.
            let (starts_with_amp, contains_question_mark, contains_hat, contains_double_dollar, starts_with_square_bracket, contains_pipe) =
                if !ls_trimmed.is_empty() {
                    let starts_with_amp = ls_trimmed.chars().next() == Some('@');
                    let contains_question_mark = ls_trimmed.contains('?');
                    let contains_hat = ls_trimmed.contains('^');
                    let first = ls_trimmed.find('$');
                    let last = ls_trimmed.rfind('$');
                    let contains_double_dollar = match (first, last) {
                        (Some(f), Some(l)) if f != l => true,
                        _ => false,
                    };
                    let starts_with_square_bracket = ls_trimmed
                        .chars()
                        .next()
                        .map(|c| c == '[' || c == ']')
                        .unwrap_or(false);
                    let contains_pipe = ls_trimmed.contains('|');
                    (
                        starts_with_amp,
                        contains_question_mark,
                        contains_hat,
                        contains_double_dollar,
                        starts_with_square_bracket,
                        contains_pipe,
                    )
                } else {
                    (false, false, false, false, false, false)
                };

            let meta = StringMetadata::new(
                starts_with_amp,
                contains_double_dollar,
                contains_question_mark,
                contains_hat,
                starts_with_square_bracket,
                contains_pipe,
            );

            // Store metadata for both tokens.
            inner.metadata.insert(low_id, meta.clone());
            inner.metadata.insert(string_id, meta);
            // Store the actual strings.
            inner.ints.insert(low_id, ls_trimmed.clone());
            inner.ints.insert(string_id, s.to_string());
            // Insert tokens into the dictionary.
            inner.strings.insert(ls_trimmed, tokens_lower);
            inner.strings.insert(s.to_string(), tokens.clone());
            tokens
        }
    }

    /// Returns the original string for the token stored in the `normal` field.
    pub fn get_string_for_ids(&self, tokens: &StringTokens) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner.ints.get(&tokens.normal).cloned()
    }

    /// Returns the lower-case string for the token stored in the `lower` field.
    pub fn get_lower_string_for_ids(&self, tokens: &StringTokens) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner.ints.get(&tokens.lower).cloned()
    }

    /// Returns the string for a specific token.
    pub fn get_string_for_id(&self, id: StringToken) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner.ints.get(&id).cloned()
    }

    /// Returns the metadata associated with a specific token.
    pub fn get_metadata_for_id(&self, id: StringToken) -> Option<StringMetadata> {
        let inner = self.inner.lock().unwrap();
        inner.metadata.get(&id).cloned()
    }
}

// A global instance of the string manager, similar to F#’s mutable global.
// (You could also inject this dependency where needed.)

pub static STRING_RESOURCE_MANAGER: Lazy<Arc<StringResourceManager>> =
    Lazy::new(|| Arc::new(StringResourceManager::new()));

/// Methods on `StringTokens` similar to the F# extension members.
impl StringTokens {
    /// Gets the original string (from the `normal` token) via the global manager.
    pub fn get_string(&self) -> Option<String> {
        STRING_RESOURCE_MANAGER.get_string_for_ids(self)
    }

    /// Gets the metadata (using the `normal` token) via the global manager.
    pub fn get_metadata(&self) -> Option<StringMetadata> {
        STRING_RESOURCE_MANAGER.get_metadata_for_id(self.normal)
    }
}