pub mod almacen;
pub mod admin;
pub mod catalogo;
pub mod ventas;

use crate::error::Result;
use crate::security::ahora;
use rusqlite::{params, Connection};

pub fn auditar(conn: &Connection, ci: &str, accion: &str, detalle: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO auditoria(fecha, ci_empleado, accion, detalle) VALUES(?1, ?2, ?3, ?4)",
        params![ahora(), ci, accion, detalle],
    )?;
    Ok(())
}
