//! Application state

use std::sync::{Arc, Mutex};

use derive_builder::Builder;
use rand::SeedableRng;
use rand_chacha::{ChaCha12Rng, ChaCha20Rng};

/// App state managed by the router
#[derive(Debug, Builder)]
#[builder(
    name = "AppStateBuilder",
    pattern = "owned",
    build_fn(name = "build_inner", private),
    public
)]
struct AppStateInner {
    db: sqlx::Pool<sqlx::Sqlite>,
    #[builder(setter(custom))]
    rng: Mutex<ChaCha20Rng>,
}

impl AppStateBuilder {
    pub(crate) fn build(self) -> Result<AppState, AppStateBuilderError> {
        Ok(AppState {
            inner: Arc::new(self.build_inner()?),
        })
    }
    pub(crate) fn rng(mut self, rng: ChaCha20Rng) -> Self {
        self.rng = Some(Mutex::new(rng));
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub(crate) fn pool(&self) -> sqlx::Pool<sqlx::Sqlite> {
        self.inner.db.clone()
    }

    pub(crate) fn rng(&self) -> rand_chacha::ChaCha12Rng {
        let mut rng = self.inner.rng.lock().unwrap();
        ChaCha12Rng::from_rng(&mut *rng).expect("seeding rng")
    }
}
