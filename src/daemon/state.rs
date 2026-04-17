use crate::error::Result;
use crate::registry::FunctionMetadata;
use crate::runtime::WorkerdPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DaemonState {
    pub functions: Arc<RwLock<HashMap<String, FunctionMetadata>>>,
    pub registry: Arc<crate::registry::FunctionRegistry>,
    pub workerd_pool: Arc<RwLock<WorkerdPool>>,
}

impl DaemonState {
    pub fn new(registry: crate::registry::FunctionRegistry) -> Self {
        let workerd_pool = WorkerdPool::new(crate::config::workerd_dir());

        Self {
            functions: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(registry),
            workerd_pool: Arc::new(RwLock::new(workerd_pool)),
        }
    }

    pub async fn load_functions(&self) -> Result<()> {
        let functions_list = self.registry.list_functions()?;
        let mut functions = self.functions.write().await;

        for func in functions_list {
            functions.insert(func.name.clone(), func);
        }

        Ok(())
    }
}
