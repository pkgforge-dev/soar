use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum FilterCondition {
    Eq(String),
    Ne(String),
    Gt(String),
    Gte(String),
    Lt(String),
    Lte(String),
    Like(String),
    ILike(String),
    In(Vec<String>),
    NotIn(Vec<String>),
    Between(String, String),
    IsNull,
    IsNotNull,
    None,
}

#[derive(Debug, Default, Clone)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Clone, Debug)]
pub struct QueryFilter {
    pub field: String,
    pub condition: FilterCondition,
    pub logical_op: Option<LogicalOp>,
}

pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub limit: Option<u32>,
    pub total: u64,
    pub has_next: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub enum ProvideStrategy {
    KeepTargetOnly,
    KeepBoth,
    Alias,
    #[default]
    None,
}

impl Display for ProvideStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ProvideStrategy::KeepTargetOnly => "=>",
            ProvideStrategy::KeepBoth => "==",
            ProvideStrategy::Alias => ":",
            _ => "",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct PackageProvide {
    pub name: String,
    pub target_name: Option<String>,
    pub strategy: ProvideStrategy,
}

impl PackageProvide {
    pub fn from_string(provide: &str) -> Self {
        if let Some((name, target_name)) = provide.split_once("==") {
            Self {
                name: name.to_string(),
                target_name: Some(target_name.to_string()),
                strategy: ProvideStrategy::KeepBoth,
            }
        } else if let Some((name, target_name)) = provide.split_once("=>") {
            Self {
                name: name.to_string(),
                target_name: Some(target_name.to_string()),
                strategy: ProvideStrategy::KeepTargetOnly,
            }
        } else if let Some((name, target_name)) = provide.split_once(":") {
            Self {
                name: name.to_string(),
                target_name: Some(target_name.to_string()),
                strategy: ProvideStrategy::Alias,
            }
        } else {
            Self {
                name: provide.to_string(),
                target_name: None,
                strategy: ProvideStrategy::None,
            }
        }
    }
}
