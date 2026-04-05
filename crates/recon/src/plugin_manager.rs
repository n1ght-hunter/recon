use std::{collections::HashMap, ops::DerefMut, path::Path};

use igloo::widgets::{Message, ToElement, WrapperRenderer, WrapperTheme};
use recon_bus::{Bus, host::EventBusView};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, HasSelf, Linker},
};
use wasmtime_wasi::{WasiCtxBuilder, p2::add_to_linker_sync};

use crate::bindings::{ReconApp, ReconState};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Wasm error: {0}")]
    WasmError(#[from] wasmtime::Error),
}

type Result<T> = std::result::Result<T, PluginError>;

pub struct ReconPluginManager {
    store: std::cell::RefCell<Store<ReconState>>,
    engine: Engine,
    linker: Linker<ReconState>,
    plugins: HashMap<String, ReconApp>,
}

impl ReconPluginManager {
    pub fn new(bus: Bus) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let mut linker = Linker::new(&engine);
        // Register iced widget host functions
        igloo::bindings::App::add_to_linker::<_, HasSelf<_>>(&mut linker, |s: &mut ReconState| s)?;
        // Register event bus host functions
        recon_bus::host::recon::event_bus::bus::add_to_linker::<_, recon_bus::host::EventBus>(
            &mut linker,
            |s| s.event_bus(),
        )?;
        // Register WASI functions
        add_to_linker_sync(&mut linker)?;

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stderr()
            .inherit_stdout()
            .build();

        let store = Store::new(&engine, ReconState::new(wasi_ctx, bus));

        Ok(Self {
            store: std::cell::RefCell::new(store),
            engine,
            linker,
            plugins: HashMap::new(),
        })
    }

    pub fn ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn add_plugin_from_file(
        &mut self,
        name: impl Into<String>,
        file: impl AsRef<Path>,
    ) -> Result<()> {
        let name = name.into();
        let component = Component::from_file(&self.engine, file)?;
        let app = ReconApp::instantiate(self.store.get_mut(), &component, &self.linker)?;
        if self.plugins.insert(name.clone(), app).is_some() {
            tracing::info!("Replaced existing plugin: {}", name);
        }
        Ok(())
    }

    pub fn plugin_update(&mut self, id: &str, msg: Message) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(id) {
            let mut store = self.store.borrow_mut();
            let Message { id, content } = msg;
            Ok(plugin.call_update(store.deref_mut(), id, &content)?)
        } else {
            Err(PluginError::NotFound(id.into()))
        }
    }

    pub fn plugin_view<'a, Theme, Renderer>(
        &self,
        id: &str,
    ) -> Option<iced::Element<'a, Message, Theme, Renderer>>
    where
        Theme: WrapperTheme + 'a,
        Renderer: WrapperRenderer + 'a,
    {
        let plugin = self.plugins.get(id)?;
        let mut store = self.store.borrow_mut();
        let result = plugin
            .call_view(store.deref_mut())
            .inspect_err(|e| {
                tracing::error!("Failed to call view for plugin {}: {}", id, e);
            })
            .ok()?;
        let element = store
            .data_mut()
            .table
            .delete(result)
            .inspect_err(|e| {
                tracing::error!("Failed to delete element for plugin {}: {}", id, e);
            })
            .ok()?;
        Some(element.to_element(&mut store.data_mut().table))
    }
}
