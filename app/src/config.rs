use sqlx::MySqlPool;
use crate::blockchain::fhenix::FhenixClient;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub fhenix: FhenixClient,
}