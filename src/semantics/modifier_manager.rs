use std::{collections::{HashMap, HashSet}, fmt};

use super::scope_manager::{ModifierCategoryInput, Scope};


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModifierCategory {
    tag: u8,
}

impl ModifierCategory {
    pub fn new(tag: u8) -> Self {
        Self { tag }
    }

    pub fn supports_scope(&self, scope: Scope, manager: &ModifierCategoryManager) -> bool {
        manager.supports_scope(*self, scope)
    }
}

impl fmt::Display for ModifierCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

pub struct ModifierCategoryManager {
    initialized: bool,
    dict: HashMap<String, ModifierCategory>,
    reverse_dict: HashMap<ModifierCategory, ModifierCategoryInput>,
    matches_set: HashSet<(ModifierCategory, Scope)>,
    id_map: HashMap<i32, ModifierCategory>,
    pub any_modifier: ModifierCategory,
    pub invalid_modifier: ModifierCategory,
}

impl ModifierCategoryManager {
    pub fn new(any_scope: Scope) -> Self {
        let any_modifier = ModifierCategory::new(0);
        let invalid_modifier = ModifierCategory::new(1);

        let mut dict = HashMap::new();
        dict.insert("any".to_string(), any_modifier);
        dict.insert("invalid_modifier".to_string(), invalid_modifier);

        let mut reverse_dict = HashMap::new();
        reverse_dict.insert(
            any_modifier,
            ModifierCategoryInput {
                name: "Any".to_string(),
                internal_id: None,
                scopes: vec![any_scope],
            },
        );
        reverse_dict.insert(
            invalid_modifier,
            ModifierCategoryInput {
                name: "Invalid".to_string(),
                internal_id: None,
                scopes: vec![],
            },
        );

        Self {
            initialized: false,
            dict,
            reverse_dict,
            matches_set: HashSet::new(),
            id_map: HashMap::new(),
            any_modifier,
            invalid_modifier,
        }
    }

    pub fn init(&mut self, modifiers: Vec<ModifierCategoryInput>) {
        self.initialized = true;
        self.dict.clear();
        self.reverse_dict.clear();
        self.dict.insert("any".to_string(), self.any_modifier);
        self.dict.insert("invalid_modifier".to_string(), self.invalid_modifier);

        self.reverse_dict.insert(self.any_modifier, self.reverse_dict[&self.any_modifier].clone());
        self.reverse_dict.insert(self.invalid_modifier, self.reverse_dict[&self.invalid_modifier].clone());

        let mut next_byte = 2u8;

        for input in modifiers {
            let modifier = ModifierCategory::new(next_byte);
            next_byte += 1;
            self.dict.insert(input.name.to_lowercase(), modifier);
            input.scopes.iter().for_each(|s| {
                self.matches_set.insert((modifier, *s));
            });
            if let Some(id) = input.internal_id {
                self.id_map.insert(id, modifier);
            }
            self.reverse_dict.insert(modifier, input);
        }
    }

    pub fn parse_modifier(&self, name: &str) -> ModifierCategory {
        self.dict
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or(self.any_modifier)
    }

    pub fn get_name(&self, modifier: ModifierCategory) -> String {
        self.reverse_dict
            .get(&modifier)
            .map(|mi| mi.name.clone())
            .unwrap_or_default()
    }

    pub fn all_modifiers(&self) -> Vec<ModifierCategory> {
        self.reverse_dict.keys().cloned().collect()
    }

    pub fn supports_scope(&self, modifier: ModifierCategory, scope: Scope) -> bool {
        self.matches_set.contains(&(modifier, scope))
            || modifier == self.any_modifier
            || scope.tag() == 0
    }

    pub fn supported_scopes(&self, modifier: ModifierCategory) -> Vec<Scope> {
        self.reverse_dict
            .get(&modifier)
            .map(|mi| mi.scopes.clone())
            .unwrap_or_default()
    }

    pub fn get_category_from_id(&self, id: i32) -> ModifierCategory {
        *self.id_map.get(&id).unwrap_or(&self.any_modifier)
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
