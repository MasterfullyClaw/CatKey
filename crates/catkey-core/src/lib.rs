//! Cat Key core: key names, the config model, the remapping engine and the
//! PowerToys importer. No OS calls live here, so all of it is unit-tested.

pub mod config;
pub mod engine;
pub mod keys;
pub mod powertoys;

pub use config::{compile, Action, Compiled, Config, Rule, Scope, ScopeMode, Settings};
pub use engine::{Effect, Engine, Env, KeyEvent, Out};
pub use keys::{ModSpec, Shortcut};
