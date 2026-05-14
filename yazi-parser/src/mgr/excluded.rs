use std::str::FromStr;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExcludedForm {
	pub state: ExcludedFormState,
}

impl TryFrom<ActionCow> for ExcludedForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { state: a.str(0).parse().unwrap_or_default() })
	}
}

impl FromLua for ExcludedForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for ExcludedForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}

// --- State
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExcludedFormState {
	#[default]
	None,
	Show,
	Hide,
	Toggle,
}

impl FromStr for ExcludedFormState {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl ExcludedFormState {
	pub fn bool(self, old: bool) -> bool {
		match self {
			Self::None => old,
			Self::Show => true,
			Self::Hide => false,
			Self::Toggle => !old,
		}
	}
}
