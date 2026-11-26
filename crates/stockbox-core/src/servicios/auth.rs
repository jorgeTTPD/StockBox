use crate::error::{AppError, Result};
use crate::models::Sesion;
use crate::security;
use rusqlite::{params, Connection, OptionalExtension};

pub fn verificar(conn: &Connection, ci: &str, contrasena: &str, rol: &str) -> Result<Sesion> {
    let registro: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT ci, nombre, rol, contrasena_hash FROM empleados WHERE lower(ci) = lower(?1)",
            params![ci.trim()],
            |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?, fila.get(3)?)),
        )
        .optional()?;
    let Some((ci_real, nombre, rol_real, hash)) = registro else {
        return Err(AppError::Validation("Credenciales incorrectas".into()));
    };
    if !security::verify_password(contrasena, &hash) {
        return Err(AppError::Validation("Credenciales incorrectas".into()));
    }
    let rol_solicitado = match rol.trim().to_ascii_uppercase().as_str() {
        "ADMIN" | "ADMINISTRADOR" => "ADMINISTRADOR",
        "CAJERO" | "VENDEDOR" => "CAJERO",
        "ALMACEN" | "ALMACENERO" => "ALMACENERO",
        _ => "",
    };
    if rol_solicitado != rol_real {
        return Err(AppError::Validation(
            "El rol seleccionado no corresponde a este usuario".into(),
        ));
    }
    Ok(Sesion {
        ci: ci_real,
        nombre,
        rol: rol_real.to_string(),
    })
}
