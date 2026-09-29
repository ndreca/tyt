// Internal API

mod add_command_to_crate;
mod crate_naming;
mod kebab_to_pascal_case;
mod kebab_to_snake_case;
mod projects_dir;
mod separate_members;
mod templates;

pub(crate) use add_command_to_crate::*;
pub(crate) use crate_naming::*;
pub(crate) use kebab_to_pascal_case::*;
pub(crate) use kebab_to_snake_case::*;
pub(crate) use projects_dir::*;
pub(crate) use separate_members::*;
pub(crate) use templates::*;

// Test support

#[cfg(test)]
mod nested_command_tests;
