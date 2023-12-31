//! Database-related functions
use crate::config::{Config, CONFIG};
use actix_web::web::{self, Data};
use diesel::{
    mysql::MysqlConnection,
    pg::PgConnection,
    r2d2::{ConnectionManager, PoolError},
    sqlite::SqliteConnection,
    Connection,
};
use log::info;

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum DatabaseConnection {
    Cockroach,
    Mysql,
    Postgres,
    Sqlite,
}

pub type Pool<T> = r2d2::Pool<ConnectionManager<T>>;
pub type CockroachPool = Pool<PgConnection>;
pub type MysqlPool = Pool<MysqlConnection>;
pub type PostgresPool = Pool<PgConnection>;
pub type SqlitePool = Pool<SqliteConnection>;

#[cfg(feature = "cockraoch")]
pub type PoolType = CockroachPool;
#[cfg(feature = "cockraoch")]
pub type ConnectionType = PgConnection;

#[cfg(feature = "mysql")]
pub type PoolType = MysqlPool;
#[cfg(feature = "mysql")]
pub type ConnectionType = MysqlConnection;

#[cfg(feature = "postgres")]
pub type PoolType = PostgresPool;
#[cfg(feature = "postgres")]
pub type ConnectionType = PgConnection;

#[cfg(feature = "sqlite")]
pub type PoolType = SqlitePool;
#[cfg(feature = "sqlite")]
pub type ConnectionType = SqliteConnection;

#[derive(Clone)]
pub enum InferPool {
    Cockroach(CockroachPool),
    Mysql(MysqlPool),
    Postgres(PostgresPool),
    Sqlite(SqlitePool),
}

impl InferPool {
    pub fn init_pool(config: Config) -> Result<Self, r2d2::Error> {
        match config.database {
            DatabaseConnection::Cockroach => {
                init_pool::<PgConnection>(config).map(InferPool::Cockroach)
            }
            DatabaseConnection::Mysql => init_pool::<MysqlConnection>(config).map(InferPool::Mysql),
            DatabaseConnection::Postgres => {
                init_pool::<PgConnection>(config).map(InferPool::Postgres)
            }
            DatabaseConnection::Sqlite => {
                init_pool::<SqliteConnection>(config).map(InferPool::Sqlite)
            }
        }
        .map_err(Into::into)
    }
}

pub fn init_pool<T>(config: Config) -> Result<Pool<T>, PoolError>
where
    T: Connection + 'static + diesel::r2d2::R2D2Connection,
{
    let manager = ConnectionManager::<T>::new(config.database_url);
    Pool::builder().build(manager)
}

pub fn add_pool(cfg: &mut web::ServiceConfig) {
    info!("config database pool");
    let pool = InferPool::init_pool(CONFIG.clone()).expect("Failed to create connection pool");
    match pool {
        InferPool::Cockroach(cockroach_pool) => cfg.app_data(Data::new(cockroach_pool)),
        InferPool::Mysql(mysql_pool) => cfg.app_data(Data::new(mysql_pool)),
        InferPool::Postgres(postgres_pool) => cfg.app_data(Data::new(postgres_pool)),
        InferPool::Sqlite(sqlite_pool) => cfg.app_data(Data::new(sqlite_pool)),
    };
}
