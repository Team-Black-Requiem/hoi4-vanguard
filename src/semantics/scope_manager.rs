use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scope {
    tag: u8,
}

impl Scope {
    pub fn new(tag: u8) -> Self {
        Self { tag }
    }

    pub fn tag(&self) -> u8 {
        self.tag
    }

    pub fn is_of_scope(&self, target: Scope, manager: &ScopeManager) -> bool {
        self.tag == manager.any_scope.tag
            || target.tag == manager.any_scope.tag
            || manager.matches_scope(*self, target)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.tag.cmp(&other.tag))
    }
}

impl Ord for Scope {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tag.cmp(&other.tag)
    }
}

#[derive(Clone, Debug)]
pub struct ScopeInput {
    pub name: String,
    pub aliases: Vec<String>,
    pub is_subscope_of: Vec<String>,
    pub data_type_name: Option<String>,
}

pub struct ScopeManager {
    initialized: bool,
    dict: HashMap<String, Scope>,
    reverse_dict: HashMap<Scope, ScopeInput>,
    group_dict: HashMap<String, Vec<Scope>>,
    complex_equality: bool,
    matches_set: HashSet<(Scope, Scope)>,
    data_type_map: HashMap<Scope, String>,
    pub any_scope: Scope,
    pub invalid_scope: Scope,
}

impl ScopeManager {
    pub fn new() -> Self {
        let any_scope = Scope::new(0);
        let invalid_scope = Scope::new(1);

        let mut dict = HashMap::new();
        dict.insert("any".to_string(), any_scope);
        dict.insert("all".to_string(), any_scope);
        dict.insert("no_scope".to_string(), any_scope);
        dict.insert("none".to_string(), any_scope);
        dict.insert("invalid_scope".to_string(), invalid_scope);

        let mut reverse_dict = HashMap::new();
        reverse_dict.insert(
            any_scope,
            ScopeInput {
                name: "Any".to_string(),
                aliases: vec!["any".to_string(), "all".to_string(), "no_scope".to_string(), "none".to_string()],
                is_subscope_of: vec![],
                data_type_name: None,
            },
        );
        reverse_dict.insert(
            invalid_scope,
            ScopeInput {
                name: "Invalid".to_string(),
                aliases: vec!["invalid_scope".to_string()],
                is_subscope_of: vec![],
                data_type_name: None,
            },
        );

        Self {
            initialized: false,
            dict,
            reverse_dict,
            group_dict: HashMap::new(),
            complex_equality: false,
            matches_set: HashSet::new(),
            data_type_map: HashMap::new(),
            any_scope,
            invalid_scope,
        }
    }

    pub fn init(&mut self, scopes: Vec<ScopeInput>, scope_groups: Vec<(String, Vec<String>)>) {
        self.initialized = true;
        self.dict.clear();
        self.reverse_dict.clear();
        self.dict.insert("any".to_string(), self.any_scope);
        self.dict.insert("all".to_string(), self.any_scope);
        self.dict.insert("no_scope".to_string(), self.any_scope);
        self.dict.insert("none".to_string(), self.any_scope);
        self.dict.insert("invalid_scope".to_string(), self.invalid_scope);

        self.reverse_dict.insert(self.any_scope, self.reverse_dict[&self.any_scope].clone());
        self.reverse_dict.insert(self.invalid_scope, self.reverse_dict[&self.invalid_scope].clone());

        let mut next_byte = 2u8;

        for input in &scopes {
            let scope = Scope::new(next_byte);
            next_byte += 1;
            for alias in &input.aliases {
                self.dict.insert(alias.to_lowercase(), scope);
            }
            self.reverse_dict.insert(scope, input.clone());
            if let Some(ref dtype) = input.data_type_name {
                self.data_type_map.insert(scope, dtype.clone());
            }
        }

        for input in &scopes {
            if let Some(source_alias) = input.aliases.first() {
                let source = self.parse_scope(source_alias);
                for subscope in &input.is_subscope_of {
                    let target = self.parse_scope(subscope);
                    self.matches_set.insert((source, target));
                }
            }
        }

        self.complex_equality = !self.matches_set.is_empty();

        for (name, group_scopes) in scope_groups {
            let parsed = group_scopes.into_iter().map(|s| self.parse_scope(&s)).collect();
            self.group_dict.insert(name, parsed);
        }
    }

    pub fn parse_scope(&self, name: &str) -> Scope {
        self.dict
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or(self.any_scope)
    }

    pub fn get_name(&self, scope: Scope) -> String {
        self.reverse_dict
            .get(&scope)
            .map(|si| si.name.clone())
            .unwrap_or_default()
    }

    pub fn all_scopes(&self) -> Vec<Scope> {
        self.reverse_dict.keys().cloned().collect()
    }

    pub fn scope_groups(&self) -> &HashMap<String, Vec<Scope>> {
        &self.group_dict
    }

    pub fn matches_scope(&self, source: Scope, target: Scope) -> bool {
        if !self.complex_equality {
            source == target || source == self.any_scope || target == self.any_scope
        } else {
            self.matches_set.contains(&(source, target)) || source == self.any_scope || target == self.any_scope
        }
    }

    pub fn data_type_for_scope(&self, scope: Scope) -> String {
        self.data_type_map
            .get(&scope)
            .cloned()
            .unwrap_or_else(|| scope.to_string())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
} 
