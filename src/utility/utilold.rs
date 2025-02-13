use std::{collections::HashMap, sync::Arc};

// Additional constants
pub(crate) const MAGIC_CHAR: char = '\u{1E00}'; // Unicode for 'Ḁ' (Latin Capital Letter A with Ring Below)
pub(crate) const MAGIC_CHAR_STRING: &str = "\u{1E00}"; // String representation of 'Ḁ'
pub(crate) const QUOTE_CHAR_ARRAY: &[char] = &['"'];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringTokens {
    pub lower: usize,
    pub normal: usize,
    pub quoted: bool,
}

impl StringTokens {
    pub fn new(lower: usize, normal: usize, quoted: bool) -> Self {
        StringTokens { lower, normal, quoted }
    }
}

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
        StringMetadata {
            starts_with_amp,
            contains_double_dollar,
            contains_question_mark,
            contains_hat,
            starts_with_square_bracket,
            contains_pipe,
        }
    }
}

//Unique string resource manager
//Somewhere off in the distance, you can hear Bob screaming into the void

//I'm gonna be so mad if this needs to be on a global scope

//I'm mad.

pub struct StringResourceManager {
    strings: HashMap<String, StringTokens>,
    ids: HashMap<usize, String>,
    metadata: HashMap<usize, StringMetadata>,
    next_id: usize,
}

impl StringResourceManager {
    pub fn new() -> Self {
        StringResourceManager {
            strings: HashMap::new(),
            ids: HashMap::new(),
            metadata: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn intern_identifier_token(&mut self, s: &str) -> StringTokens {
        if let Some(token) = self.strings.get(s) {
            return token.clone();
        }

        let ls = s.to_lowercase();
        let quoted = s.starts_with('"') && s.ends_with('"');
        
        let string_id = self.next_id;
        self.next_id += 1;
        let lower_id = self.next_id;
        self.next_id += 1;

        let token = StringTokens::new(lower_id, string_id, quoted);
        self.strings.insert(s.to_string(), token.clone());
        self.ids.insert(string_id, s.to_string());
        self.ids.insert(lower_id, ls.clone());

        let starts_with_amp = ls.starts_with('@');
        let contains_double_dollar = ls.matches('$').count() > 1;
        let contains_question_mark = ls.contains('?');
        let contains_hat = ls.contains('^');
        let starts_with_square_bracket = ls.starts_with('[') || ls.starts_with(']');
        let contains_pipe = ls.contains('|');

        let meta = StringMetadata::new(
            starts_with_amp,
            contains_double_dollar,
            contains_question_mark,
            contains_hat,
            starts_with_square_bracket,
            contains_pipe,
        );

        self.metadata.insert(string_id, meta.clone());
        self.metadata.insert(lower_id, meta);

        token
    }

    pub fn get_string_for_ids(&self, id: &StringTokens) -> Option<String> {
        self.ids.get(&id.normal).cloned()
    }

    pub fn get_lower_string_for_ids(&self, id: &StringTokens) -> Option<String> {
        self.ids.get(&id.lower).cloned()
    }

    pub fn get_string_for_id(&self, id: usize) -> Option<String> {
        self.ids.get(&id).cloned()
    }

    pub fn get_metadata_for_id(&self, id: usize) -> Option<&StringMetadata> {
        self.metadata.get(&id)
    }
}

use once_cell::sync::Lazy;
pub static STRING_RESOURCE_MANAGER: Lazy<Arc<StringResourceManager>> =
    Lazy::new(|| Arc::new(StringResourceManager::new()));