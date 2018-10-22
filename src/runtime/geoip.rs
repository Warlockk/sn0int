use errors::*;

use engine::ctx::State;
use json::LuaJsonValue;
use hlua::{self, AnyLuaValue};
use serde_json;
use std::sync::Arc;
use std::net::IpAddr;


pub fn geoip_lookup(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("geoip_lookup", hlua::function1(move |ip: String| -> Result<AnyLuaValue> {
        let ip = ip.parse::<IpAddr>()
            .context("Failed to parse IP")
            .map_err(|err| state.set_error(err.into()))?;

        let lookup = state.geoip().lookup(ip)
            .map_err(|err| state.set_error(err))?;

        let lookup = serde_json::to_value(lookup)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(lookup).into())
    }))
}


#[cfg(test)]
mod tests {
    use engine::ctx::Script;

    #[test]
    fn verify_geoip() {
        let script = Script::load_unchecked(r#"
        function run()
            x = geoip_lookup('1.1.1.1')
            print(x)
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
