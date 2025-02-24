use mlua::prelude::*;

fn main() -> LuaResult<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    let require = lua.create_function(|lua, path: String| {
        if path.contains("recon-plugin") {
            println!("Loading recon-plugin");
            return recon_plugin(lua);
        }
        Err(LuaError::runtime("Module not found"))
    })?;
    lua.globals().set("require", require)?;

    // let sum = lau_sum(&lua)?;

    lua.load(std::path::Path::new("test.lua")).exec()?;

    Ok(())
}

fn recon_plugin(lua: &Lua) -> Result<LuaTable, LuaError> {
    let table = lua.create_table()?;
    table.set("sum", lau_sum(lua)?)?;
    

    Ok(table)
}

fn lau_sum(lua: &Lua) -> Result<LuaFunction, LuaError> {
    lua.create_function(|_, (a, b): (i32, i32)| Ok(a + b))
}
