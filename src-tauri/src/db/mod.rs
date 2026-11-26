pub mod migrate;

pub use migrate::{CsvBundle, MigrationReport};

use crate::error::Result;
use crate::security;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

const SCHEMA_VERSION: &str = "1";

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    pub fn memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let mut db = Self { conn };
        db.inicializar()?;
        Ok(db)
    }

    fn inicializar(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(include_str!("../schema.sql"))?;
        tx.execute(
            "INSERT OR IGNORE INTO meta(key, value) VALUES('schema_version', ?1)",
            params![SCHEMA_VERSION],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn meta(&self, clave: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut filas = stmt.query_map(params![clave], |fila| fila.get::<_, String>(0))?;
        match filas.next() {
            Some(valor) => Ok(Some(valor?)),
            None => Ok(None),
        }
    }

    pub fn set_meta(&self, clave: &str, valor: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![clave, valor],
        )?;
        Ok(())
    }

    pub fn seed_if_empty(&mut self) -> Result<MigrationReport> {
        if self.meta("sembrado")?.as_deref() == Some("true") {
            return Ok(MigrationReport::default());
        }
        let report = migrate::importar(&mut self.conn, &CsvBundle::incrustado())?;
        self.set_meta("sembrado", "true")?;
        Ok(report)
    }

    pub fn importar_csv(&mut self, directorio: &Path) -> Result<MigrationReport> {
        migrate::importar(&mut self.conn, &CsvBundle::desde_directorio(directorio))
    }

    pub fn auditar(&self, ci: &str, accion: &str, detalle: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO auditoria(fecha, ci_empleado, accion, detalle) VALUES(?1, ?2, ?3, ?4)",
            params![security::ahora(), ci, accion, detalle],
        )?;
        Ok(())
    }
}
