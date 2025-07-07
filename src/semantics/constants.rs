use std::cmp::Ordering;

use crate::{parser::sharedparsers::Span, utility::util::StringTokens};
use super::{modifier_manager::ModifierCategory, scope_manager::Scope};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Game {
    CK2 = 0,
    HOI4 = 1,
    EU4 = 2,
    Stl = 3,
    VIC2 = 4,
    IR = 5,
    CK3 = 6,
    Custom = 99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CK2Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Russian = 4,
    Default = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum STLLang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Russian = 4,
    Polish = 5,
    BrazPor = 6,
    Default = 7,
    Chinese = 8,
    Japanese = 9,
    Korean = 10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HOI4Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Russian = 4,
    Polish = 5,
    BrazPor = 6,
    Default = 7, // Doesn't exist, but kept for compatibility
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EU4Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Default = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRLang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Chinese = 4,
    Russian = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VIC2Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CK3Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Chinese = 4,
    Russian = 5,
    Korean = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VIC3Lang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Chinese = 4,
    Russian = 5,
    Korean = 6,
    BrazPor = 7,
    Japanese = 8,
    Polish = 9,
    Turkish = 10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomLang {
    English = 0,
    French = 1,
    German = 2,
    Spanish = 3,
    Russian = 4,
    Polish = 5,
    BrazPor = 6,
    Chinese = 7,
    Default = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    CK2(CK2Lang),
    Stl(STLLang),
    HOI4(HOI4Lang),
    EU4(EU4Lang),
    IR(IRLang),
    VIC2(VIC2Lang),
    CK3(CK3Lang),
    VIC3(VIC3Lang),
    Custom(CustomLang),
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Lang::*;
        match self {
            CK2(l) => write!(f, "{:?}", l),
            Stl(l) => write!(f, "{:?}", l),
            HOI4(l) => write!(f, "{:?}", l),
            EU4(l) => write!(f, "{:?}", l),
            IR(l) => write!(f, "{:?}", l),
            VIC2(l) => write!(f, "{:?}", l),
            CK3(l) => write!(f, "{:?}", l),
            VIC3(l) => write!(f, "{:?}", l),
            Custom(l) => write!(f, "{:?}", l),
        }
    }
}

pub mod lang_helpers {
    use super::*;

    pub fn all_ck2_langs() -> Vec<Lang> {
        vec![
            Lang::CK2(CK2Lang::English),
            Lang::CK2(CK2Lang::French),
            Lang::CK2(CK2Lang::German),
            Lang::CK2(CK2Lang::Spanish),
            Lang::CK2(CK2Lang::Russian),
        ]
    }

    pub fn all_stl_langs() -> Vec<Lang> {
        vec![
            Lang::Stl(STLLang::English),
            Lang::Stl(STLLang::French),
            Lang::Stl(STLLang::German),
            Lang::Stl(STLLang::Spanish),
            Lang::Stl(STLLang::Russian),
            Lang::Stl(STLLang::Polish),
            Lang::Stl(STLLang::BrazPor),
            Lang::Stl(STLLang::Chinese),
            Lang::Stl(STLLang::Japanese),
            Lang::Stl(STLLang::Korean),
        ]
    }

    pub fn all_hoi4_langs() -> Vec<Lang> {
        vec![
            Lang::HOI4(HOI4Lang::English),
            Lang::HOI4(HOI4Lang::French),
            Lang::HOI4(HOI4Lang::German),
            Lang::HOI4(HOI4Lang::Spanish),
            Lang::HOI4(HOI4Lang::Russian),
            Lang::HOI4(HOI4Lang::Polish),
            Lang::HOI4(HOI4Lang::BrazPor),
        ]
    }

    pub fn all_eu4_langs() -> Vec<Lang> {
        vec![
            Lang::EU4(EU4Lang::English),
            Lang::EU4(EU4Lang::French),
            Lang::EU4(EU4Lang::German),
            Lang::EU4(EU4Lang::Spanish),
        ]
    }

    pub fn all_ir_langs() -> Vec<Lang> {
        vec![
            Lang::IR(IRLang::English),
            Lang::IR(IRLang::French),
            Lang::IR(IRLang::German),
            Lang::IR(IRLang::Spanish),
            Lang::IR(IRLang::Russian),
            Lang::IR(IRLang::Chinese),
        ]
    }

    pub fn all_vic2_langs() -> Vec<Lang> {
        vec![
            Lang::VIC2(VIC2Lang::English),
            Lang::VIC2(VIC2Lang::French),
            Lang::VIC2(VIC2Lang::German),
            Lang::VIC2(VIC2Lang::Spanish),
        ]
    }

    pub fn all_ck3_langs() -> Vec<Lang> {
        vec![
            Lang::CK3(CK3Lang::English),
            Lang::CK3(CK3Lang::French),
            Lang::CK3(CK3Lang::German),
            Lang::CK3(CK3Lang::Spanish),
            Lang::CK3(CK3Lang::Chinese),
            Lang::CK3(CK3Lang::Russian),
            Lang::CK3(CK3Lang::Korean),
        ]
    }

    pub fn all_vic3_langs() -> Vec<Lang> {
        vec![
            Lang::VIC3(VIC3Lang::English),
            Lang::VIC3(VIC3Lang::Chinese),
            Lang::VIC3(VIC3Lang::French),
            Lang::VIC3(VIC3Lang::German),
            Lang::VIC3(VIC3Lang::Japanese),
            Lang::VIC3(VIC3Lang::Korean),
            Lang::VIC3(VIC3Lang::Polish),
            Lang::VIC3(VIC3Lang::Russian),
            Lang::VIC3(VIC3Lang::Spanish),
            Lang::VIC3(VIC3Lang::Turkish),
            Lang::VIC3(VIC3Lang::BrazPor),
        ]
    }

    pub fn all_custom_langs() -> Vec<Lang> {
        vec![
            Lang::Custom(CustomLang::English),
            Lang::Custom(CustomLang::French),
            Lang::Custom(CustomLang::German),
            Lang::Custom(CustomLang::Spanish),
            Lang::Custom(CustomLang::Russian),
            Lang::Custom(CustomLang::Polish),
            Lang::Custom(CustomLang::BrazPor),
            Lang::Custom(CustomLang::Chinese),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct RawEffect {
    pub name: String,
    pub desc: String,
    pub usage: String,
    pub scopes: Vec<String>,
    pub targets: Vec<String>,
    pub traits: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, Clone)]
pub struct TypeDefInfo<'a> {
    pub id: String,
    pub validate: bool,
    pub range: Span<'a>,
    pub explicit_localisation: Vec<(String, String, bool)>,
    pub subtypes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ModifierSource {
    Rules,
    CodeGen,
    TypeDef { name: String, type_def: String },
}

#[derive(Debug, Clone)]
pub struct ActualModifier {
    pub tag: String,
    pub category: ModifierCategory,
}

#[derive(Debug, Clone)]
pub struct StaticModifier {
    pub tag: String,
    pub categories: Vec<ModifierCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    Effect,
    Trigger,
    Link,
    ValueTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceHint {
    Type { type_name: String, type_value: String },
    Loc { loc_key: String },
    Enum { enum_name: String, enum_value: String },
    File { filename: String },
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub name: StringTokens,
    pub scopes: Vec<Scope>,
    pub effect_type: EffectType,
    pub ref_hint: Option<ReferenceHint>,
}

impl Effect {
    pub fn new(name: StringTokens, scopes: Vec<Scope>, effect_type: EffectType) -> Self {
        Self {
            name,
            scopes,
            effect_type,
            ref_hint: None,
        }
    }

    pub fn new_with_hint(
        name: StringTokens,
        scopes: Vec<Scope>,
        effect_type: EffectType,
        ref_hint: Option<ReferenceHint>,
    ) -> Self {
        Self {
            name,
            scopes,
            effect_type,
            ref_hint,
        }
    }

    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PartialEq for Effect {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.scopes == other.scopes && self.effect_type == other.effect_type
    }
}

impl Eq for Effect {}

impl PartialOrd for Effect {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.normal.cmp(&other.name.normal))
    }
}

impl Ord for Effect {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.normal.cmp(&other.name.normal)
    }
}

#[derive(Debug, Clone)]
pub struct ScriptedEffect {
    pub effect: Effect,
    pub comments: String,
    pub global_event_targets: Vec<String>,
    pub saved_event_targets: Vec<String>,
    pub used_event_targets: Vec<String>,
}

impl PartialEq for ScriptedEffect {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect
    }
}

impl Eq for ScriptedEffect {}

#[derive(Debug, Clone)]
pub struct DocEffect {
    pub effect: Effect,
    pub desc: String,
    pub usage: String,
    pub target: Option<Scope>,
}

impl PartialEq for DocEffect {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect
            && self.desc == other.desc
            && self.usage == other.usage
    }
}

impl Eq for DocEffect {}

#[derive(Debug, Clone)]
pub struct ScopedEffect {
    pub doc_effect: DocEffect,
    pub is_scope_change: bool,
    pub ignore_children: Vec<String>,
    pub scope_only_not_effect: bool,
    pub is_value_scope: bool,
    pub is_wildcard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitleType {
    Empire,
    Kingdom,
    DuchyHired,
    DuchyNormal,
    County,
    Barony,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLinkType {
    Scope,
    Value,
    Both,
}

#[derive(Debug, Clone)]
pub struct EventTargetDataLink {
    pub name: String,
    pub input_scopes: Vec<Scope>,
    pub output_scope: Scope,
    pub description: String,
    pub data_prefix: Option<String>,
    pub source_rule_type: String,
    pub data_link_type: DataLinkType,
}

#[derive(Debug, Clone)]
pub enum EventTargetLink {
    SimpleLink(ScopedEffect),
    DataLink(EventTargetDataLink),
}