use std::path::PathBuf;
use std::sync::Arc;

use crate::app::error::{Result, RivuletError};
use crate::fetcher::http_fetcher::HttpFetcher;
use crate::fetcher::parallel::{ParallelFetcher, DEFAULT_WORKERS};
use crate::fetcher::Fetcher;
use crate::normalizer::Normalizer;
use crate::store::sqlite::SqliteStore;

pub struct AppContext {
    pub store: Arc<SqliteStore>,
    pub fetcher: Arc<dyn Fetcher + Send + Sync>,
    pub parallel_fetcher: ParallelFetcher,
    pub normalizer: Normalizer,
}

impl AppContext {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        Self::with_workers(db_path, DEFAULT_WORKERS)
    }

    pub fn with_workers(db_path: Option<PathBuf>, workers: usize) -> Result<Self> {
        let db_path = match db_path {
            Some(p) => p,
            None => Self::default_db_path()?,
        };

        let store = Arc::new(SqliteStore::new(&db_path)?);
        let fetcher: Arc<dyn Fetcher + Send + Sync> = Arc::new(HttpFetcher::new());
        let parallel_fetcher = ParallelFetcher::with_workers(fetcher.clone(), workers);
        let normalizer = Normalizer::new();

        Ok(Self {
            store,
            fetcher,
            parallel_fetcher,
            normalizer,
        })
    }

    pub fn in_memory() -> Result<Self> {
        Self::in_memory_with_workers(DEFAULT_WORKERS)
    }

    pub fn in_memory_with_workers(workers: usize) -> Result<Self> {
        let store = Arc::new(SqliteStore::in_memory()?);
        let fetcher: Arc<dyn Fetcher + Send + Sync> = Arc::new(HttpFetcher::new());
        let parallel_fetcher = ParallelFetcher::with_workers(fetcher.clone(), workers);
        let normalizer = Normalizer::new();

        Ok(Self {
            store,
            fetcher,
            parallel_fetcher,
            normalizer,
        })
    }

    fn default_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| RivuletError::Config("Could not find data directory".into()))?;
        let rivulet_dir = data_dir.join("rivulet");
        std::fs::create_dir_all(&rivulet_dir)?;
        Ok(rivulet_dir.join("rivulet.db"))
    }
}
