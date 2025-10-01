pub mod models;
pub mod schema;

use std::env;

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;

use crate::pkgs::package::PackageDefinition;

use self::models::{NewPackage, Package};
use self::schema::packages::dsl as packages_dsl;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type Connection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

const DEFAULT_DB_URL: &str = "lpkg.db";

/// Resolve the database URL from `LPKG_DATABASE_URL` or fall back to `lpkg.db` in the CWD.
pub fn database_url() -> String {
    env::var("LPKG_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DB_URL.to_string())
}

/// Build an r2d2 connection pool and ensure the schema exists.
pub fn establish_pool() -> Result<Pool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url());
    let pool = Pool::builder()
        .build(manager)
        .context("creating Diesel connection pool")?;

    {
        let mut conn = pool
            .get()
            .context("establishing initial database connection")?;
        initialize(&mut conn)?;
    }

    Ok(pool)
}

fn initialize(conn: &mut SqliteConnection) -> Result<()> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS packages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            source TEXT,
            md5 TEXT,
            configure_args TEXT,
            build_commands TEXT,
            install_commands TEXT,
            dependencies TEXT,
            enable_lto BOOLEAN NOT NULL DEFAULT 1,
            enable_pgo BOOLEAN NOT NULL DEFAULT 1,
            cflags TEXT,
            ldflags TEXT,
            profdata TEXT
        )",
    )
    .execute(conn)
    .context("creating packages table")?;

    diesel::sql_query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_packages_name_version ON packages(name, version)",
    )
    .execute(conn)
    .context("creating packages unique index")?;

    Ok(())
}

/// Insert or update a package definition in the database.
pub fn upsert_package(conn: &mut SqliteConnection, definition: &PackageDefinition) -> Result<()> {
    let record = NewPackage::try_from(definition)?;

    diesel::insert_into(packages_dsl::packages)
        .values(&record)
        .on_conflict((packages_dsl::name, packages_dsl::version))
        .do_update()
        .set(&record)
        .execute(conn)
        .context("upserting package record")?;

    Ok(())
}

/// Convenience helper to upsert via pool and return the persisted definition.
pub fn upsert_package_via_pool(pool: &Pool, definition: &PackageDefinition) -> Result<()> {
    let mut conn = pool.get().context("acquiring database connection")?;
    upsert_package(&mut conn, definition)
}

/// Load all packages from the database.
pub fn load_packages(conn: &mut SqliteConnection) -> Result<Vec<Package>> {
    packages_dsl::packages
        .order((packages_dsl::name, packages_dsl::version))
        .load::<Package>(conn)
        .context("loading packages from database")
}

/// Load packages using the shared pool.
pub fn load_packages_via_pool(pool: &Pool) -> Result<Vec<Package>> {
    let mut conn = pool.get().context("acquiring database connection")?;
    load_packages(&mut conn)
}
