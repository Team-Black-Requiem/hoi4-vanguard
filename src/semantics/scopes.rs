use std::collections::HashMap;

use crate::utility::util::{
    PrefixOptimisedStringSet,
    StringResourceManager,
    StringToken,
    StringTokens
};
use super::{
    constants::{
        Effect, ReferenceHint, ScopedEffect
    },
    scope_manager::{Scope, ScopeManager}
};

pub trait IntoEffectKey {
    fn into_token(self, manager: &StringResourceManager) -> StringTokens;
}

impl<'a> IntoEffectKey for &'a str {
    fn into_token(self, manager: &StringResourceManager) -> StringTokens {
        manager.intern_identifier_token(self)
    }
}

impl<'a> IntoEffectKey for &'a StringTokens {
    fn into_token(self, _manager: &StringResourceManager) -> StringTokens {
        self.clone()
    }
}

pub struct EffectDictionary {
    map: HashMap<StringToken, Effect>,
}

impl EffectDictionary {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn from_list(effects: impl IntoIterator<Item = Effect>) -> Self {
        let mut map = HashMap::new();
        for effect in effects {
            map.insert(effect.name.lower, effect);
        }
        Self { map }
    }

    pub fn try_find<K: IntoEffectKey>(
        &self,
        key: K,
        manager: &StringResourceManager,
    ) -> Option<&Effect> {
        let token = key.into_token(manager);
        self.map.get(&token.lower)
    }

    pub fn values(&self) -> impl Iterator<Item = &Effect> {
        self.map.values()
    }
}

pub type EffectMap = EffectDictionary;

pub type UsageScopeContext = Vec<Scope>;

#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub root: Scope,
    pub from: Vec<Scope>,
    pub scopes: Vec<Scope>,
}

impl ScopeContext {
    pub fn current_scope(&self, manager: &ScopeManager) -> Scope {
        self.scopes.first().cloned().unwrap_or(manager.any_scope)
    }

    pub fn pop_scope(&self) -> Vec<Scope> {
        self.scopes.iter().skip(1).cloned().collect()
    }

    pub fn get_from(&self, i: usize, manager: &ScopeManager) -> Scope {
        if i > 0 && i <= self.from.len() {
            self.from[i - 1]
        } else {
            manager.any_scope
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScopeResult {
    NewScope {
        new_scope: ScopeContext,
        ignore_keys: Vec<String>,
        ref_hint: Option<ReferenceHint>,
    },
    WrongScope {
        command: String,
        scope: Scope,
        expected: Vec<Scope>,
        ref_hint: Option<ReferenceHint>,
    },
    NotFound,
    VarFound,
    VarNotFound { var: String },
    ValueFound { ref_hint: Option<ReferenceHint> },
}

pub fn new_scope(
    new_scope: ScopeContext,
    ignore_keys: Vec<String>,
    ref_hint: Option<ReferenceHint>,
) -> ScopeResult {
    ScopeResult::NewScope {
        new_scope,
        ignore_keys,
        ref_hint,
    }
}

pub type ChangeScopeFn = dyn Fn(
    bool,                               // is_trigger
    bool,                               // is_from
    &EffectDictionary,                  // effect_map
    &EffectDictionary,                  // core_effect_map
    &[ScopedEffect],                    // scoped_effects
    &PrefixOptimisedStringSet,          // valid_keys
    &str,                               // command
    &ScopeContext,                      // scope_context
) -> ScopeResult;

pub type ChangeScope = Box<ChangeScopeFn>;

pub trait FnChangeScope: Fn(
    bool,
    bool,
    &EffectDictionary,
    &EffectDictionary,
    &[ScopedEffect],
    &PrefixOptimisedStringSet,
    &str,
    &ScopeContext
) -> ScopeResult {}

/// Returns the default `ScopeContext` with the root set to `any_scope`.
pub fn default_context(scope_manager: &ScopeManager) -> ScopeContext {
    ScopeContext {
        root: scope_manager.any_scope,
        from: vec![],
        scopes: vec![],
    }
}

/// Returns the `ScopeContext` representing an invalid state.
pub fn none_context(scope_manager: &ScopeManager) -> ScopeContext {
    ScopeContext {
        root: scope_manager.invalid_scope,
        from: vec![],
        scopes: vec![scope_manager.invalid_scope],
    }
}

/// Returns a closure that checks for and strips a simple variable prefix.
pub fn simple_var_prefix_fun<'a>(prefix: &'a str) -> impl Fn(&'a str) -> (String, bool) {
    let prefix_len = prefix.len();
    move |key: &str| {
        if key.to_lowercase().starts_with(&prefix.to_lowercase()) {
            (key[prefix_len..].to_string(), true)
        } else {
            (key.to_string(), false)
        }
    }
}

/// Returns a closure that checks for and strips one of two variable prefixes.
pub fn complex_var_prefix_fun<'a>(
    prefix1: &'a str,
    prefix2: &'a str,
) -> impl Fn(&'a str) -> (String, bool) {
    let p1_len = prefix1.len();
    let p2_len = prefix2.len();

    move |key: &str| {
        let key_lower = key.to_lowercase();
        if key_lower.starts_with(&prefix1.to_lowercase()) {
            (key[p1_len..].to_string(), true)
        } else if key_lower.starts_with(&prefix2.to_lowercase()) {
            (key[p2_len..].to_string(), true)
        } else {
            (key.to_string(), false)
        }
    }
}

/// Applies a target scope (if any) to the current scope stack.
pub fn apply_target_scope(scope: Option<Scope>, context: Vec<Scope>) -> Vec<Scope> {
    match scope {
        Some(s) => {
            let mut new_context = context;
            new_context.insert(0, s); // Prepend
            new_context
        }
        None => context,
    }
}

/// Creates a closure that handles Jomini change scope logic.
// POV: Are you sure whatever you're doing is worth it?
// this is so ass
// I am sorry
pub fn create_jomini_change_scope<'a>(
    one_to_one_scopes: Vec<(String, Box<dyn Fn(&(ScopeContext, bool), &str, bool) -> ((ScopeContext, bool), ScopeResult) + 'a>)>,
    var_prefix_fun: Box<dyn Fn(&str) -> (String, bool) + 'a>,
    scope_manager: &'a ScopeManager,
    string_manager: &'a StringResourceManager,
) -> impl Fn(
        bool, // var_lhs
        bool, // skip_effect
        &EffectMap,
        &EffectMap,
        &[ScopedEffect],
        &HashMap<String, ()>, // PrefixOptimisedStringSet
        &str,
        &ScopeContext,
    ) -> ScopeResult + 'a
{
    move |var_lhs, skip_effect, event_target_links, value_triggers, wildcard_links, vars, key, source| {
        let mut key = if key.to_ascii_lowercase().starts_with("hidden:") {
            &key[7..]
        } else {
            key
        };

        if key.to_ascii_lowercase().starts_with("event_target:")
            || key.to_ascii_lowercase().starts_with("parameter:")
            || key.starts_with('@')
        {
            return ScopeResult::NewScope {
                new_scope: ScopeContext {
                    root: source.root,
                    from: source.from.clone(),
                    scopes: {
                        let mut s = source.scopes.clone();
                        s.insert(0, scope_manager.any_scope);
                        s
                    },
                },
                ignore_keys: vec![],
                ref_hint: None,
            };
        }

        let (key, var_only) = var_prefix_fun(key);
        let (before_amp, after_amp, has_amp) = if let Some(idx) = key.find('@') {
            (&key[..idx], &key[idx + 1..], true)
        } else {
            (key.as_str(), "", false)
        };

        let keys: Vec<_> = before_amp.split('.').collect();
        let keylength = keys.len().saturating_sub(1);

        // Helper for scope application
        fn apply_target_scope(scope: Option<Scope>, context: &[Scope]) -> Vec<Scope> {
            let mut v = context.to_vec();
            if let Some(ps) = scope {
                v.insert(0, ps);
            }
            v
        }

        // Inner function
        let inner = |(context, changed): (ScopeContext, bool), next_key: &str, last: bool| -> ((ScopeContext, bool), ScopeResult) {
            // Find one-to-one scope mapping
            if let Some((_, f)) = one_to_one_scopes.iter().find(|(k, _)| k == next_key) {
                let (ctx, _) = f(&(context.clone(), false), next_key, last);
                return ((ctx.0.clone(), false), ScopeResult::NewScope {
                    new_scope: ctx.0,
                    ignore_keys: vec![],
                    ref_hint: None,
                });
            }
        
            // Try event target link
            let event_target_link_match = event_target_links.try_find(next_key, string_manager)
                .and_then(|e| e.as_any().downcast_ref::<ScopedEffect>());
            let value_scope_match = value_triggers.try_find(next_key, string_manager)
                .and_then(|e| e.as_any().downcast_ref::<ScopedEffect>());
            let wildcard_scope_match = wildcard_links.iter().find(|l| {
                next_key.to_ascii_lowercase().starts_with(
                    &string_manager
                        .get_string_for_id(l.doc_effect.effect.name.normal)
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                )
            });
        
            // F# Option.orElse: event_target_link_match.or(wildcard_scope_match)
            let event_or_wildcard = event_target_link_match.or(wildcard_scope_match);
        
            match (event_or_wildcard, value_scope_match) {
                // Value scope match
                (_, Some(e)) => {
                    if last {
                        let possible_scopes = &e.doc_effect.effect.scopes;
                        let current_scope = context.current_scope(scope_manager);
                        let exact = possible_scopes.iter().any(|x| current_scope.is_of_scope(*x, scope_manager));
                        let ref_hint = e.doc_effect.effect.ref_hint.clone();
                    
                        match (current_scope, possible_scopes.is_empty(), exact) {
                            (x, _, _) if x == scope_manager.any_scope => ((context, false), ScopeResult::ValueFound { ref_hint }),
                            (_, true, _) => ((context, false), ScopeResult::NotFound),
                            (_, _, true) => ((context, false), ScopeResult::ValueFound { ref_hint }),
                            (current, _, false) => ((context, false), ScopeResult::WrongScope {
                                command: next_key.to_string(),
                                scope: current,
                                expected: possible_scopes.clone(),
                                ref_hint,
                            }),
                        }
                    } else {
                        ((context, false), ScopeResult::NotFound)
                    }
                }
                // No event or wildcard, no value scope
                (None, _) => {
                    if last && vars.contains_key(next_key) {
                        ((context, false), ScopeResult::VarFound)
                    } else if var_only {
                        ((context, false), ScopeResult::VarNotFound { var: next_key.to_string() })
                    } else {
                        ((context, false), ScopeResult::NotFound)
                    }
                }
                // Event or wildcard match
                (Some(e), _) => {
                    let possible_scopes = &e.doc_effect.effect.scopes;
                    let current_scope = context.current_scope(scope_manager);
                    let exact = possible_scopes.iter().any(|x| current_scope.is_of_scope(*x, scope_manager));
                    let ref_hint = e.doc_effect.effect.ref_hint.clone();
                    let is_scope_change = e.is_scope_change;
                
                    match (current_scope, possible_scopes.is_empty(), exact, is_scope_change) {
                        (x, _, _, true) if x == scope_manager.any_scope => {
                            let new_scopes = apply_target_scope(e.doc_effect.target, &context.scopes.clone());
                            let new_ctx = ScopeContext { scopes: new_scopes.clone(), ..context.clone() };
                            (
                                (new_ctx.clone(), true),
                                ScopeResult::NewScope {
                                    new_scope: ScopeContext { scopes: new_scopes, ..source.clone() },
                                    ignore_keys: e.ignore_children.clone(),
                                    ref_hint,
                                }
                            )
                        }
                        (x, _, _, false) if x == scope_manager.any_scope => {
                            ((context.clone(), false), ScopeResult::NewScope {
                                new_scope: context,
                                ignore_keys: e.ignore_children.clone(),
                                ref_hint,
                            })
                        }
                        (_, true, _, _) => ((context, false), ScopeResult::NotFound),
                        (_, _, true, true) => {
                            let new_scopes = apply_target_scope(e.doc_effect.target, &context.scopes.clone());
                            let new_ctx = ScopeContext { scopes: new_scopes.clone(), ..context.clone() };
                            (
                                (new_ctx.clone(), true),
                                ScopeResult::NewScope {
                                    new_scope: ScopeContext { scopes: new_scopes, ..source.clone() },
                                    ignore_keys: e.ignore_children.clone(),
                                    ref_hint,
                                }
                            )
                        }
                        (_, _, true, false) => ((context.clone(), false), ScopeResult::NewScope {
                            new_scope: context,
                            ignore_keys: e.ignore_children.clone(),
                            ref_hint,
                        }),
                        (current, _, false, _) => ((context, false), ScopeResult::WrongScope {
                            command: next_key.to_string(),
                            scope: current,
                            expected: possible_scopes.clone(),
                            ref_hint,
                        }),
                    }
                }
            }
        };

        // Fold over keys
        let mut acc = ((source.clone(), false), None);
        for (i, k) in keys.iter().enumerate() {
            let last = i == keylength;
            let (ctx, res) = match &acc.1 {
                None => inner(acc.0.clone(), k, last),
                Some(ScopeResult::NewScope { new_scope: x, .. }) => inner((x.clone(), acc.0.1), k, last),
                Some(_) => (acc.0.clone(), acc.1.clone().unwrap()),
            };
            acc = (ctx, Some(res));
        }
        // Extract result
        let res2 = match acc {
            ((_, _), None) => ScopeResult::NotFound,
            ((_, true), Some(r)) => match r {
                ScopeResult::NewScope { new_scope: x, ignore_keys: i, ref_hint: rh } => ScopeResult::NewScope {
                    new_scope: ScopeContext {
                        scopes: {
                            let mut s = source.scopes.clone();
                            s.insert(0, x.current_scope(scope_manager));
                            s
                        },
                        ..source.clone()
                    },
                    ignore_keys: i,
                    ref_hint: rh,
                },
                x => x,
            },
            ((_, false), Some(r)) => r,
        };
        // If '@' present, fold over after '@' as well
        if has_amp {
            let keys: Vec<_> = after_amp.split('.').collect();
            let keylength = keys.len().saturating_sub(1);
            let mut acc = ((source.clone(), false), None);
            for (i, k) in keys.iter().enumerate() {
                let last = i == keylength;
                let (ctx, res) = match &acc.1 {
                    None => inner(acc.0.clone(), k, last),
                    Some(ScopeResult::NewScope { new_scope: x, .. }) => inner((x.clone(), acc.0.1), k, last),
                    Some(_) => (acc.0.clone(), acc.1.clone().unwrap()),
                };
                acc = (ctx, Some(res));
            }
            let tres2 = match acc {
                ((_, _), None) => ScopeResult::NotFound,
                ((_, true), Some(r)) => match r {
                    ScopeResult::NewScope { new_scope: x, ignore_keys: i, ref_hint: rh } => ScopeResult::NewScope {
                        new_scope: ScopeContext {
                            scopes: {
                                let mut s = source.scopes.clone();
                                s.insert(0, x.current_scope(scope_manager));
                                s
                            },
                            ..source.clone()
                        },
                        ignore_keys: i,
                        ref_hint: rh,
                    },
                    x => x,
                },
                ((_, false), Some(r)) => r,
            };

            match (res2, tres2) {
                (_, ScopeResult::NotFound) => ScopeResult::NotFound,
                (_, ScopeResult::VarNotFound { var: s }) => ScopeResult::VarNotFound { var: s },
                (ScopeResult::VarFound, _) => ScopeResult::VarFound,
                (_, _) => ScopeResult::NotFound,
            }
        } else {
            res2
        }
    }
}