//! Cat Key core: key names, the config model, the remapping engine and the
//! PowerToys importer. No OS calls live here, so all of it is unit-tested.

pub mod appscan;
pub mod coachtext;
pub mod dock;
pub mod config;
pub mod engine;
pub mod keys;
pub mod powertoys;
pub mod shellcmd;
pub mod switcher;

pub use config::{builtin_templates, compile, Action, Compiled, Config, Rule, Scope, ScopeMode, Settings, Template};
pub use engine::{Effect, Engine, Env, KeyEvent, Out};
pub use keys::{ModSpec, Shortcut};
