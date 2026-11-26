use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::{
    Auditoria, Caja, Cliente, ClienteInput, Empleado, EmpleadoInput, Entrada, Estadisticas,
    ItemCarrito, OrdenCreada, OrdenInput, Pedido, Producto, ProductoInput, Proveedor,
    ProveedorInput, ResultadoVenta, Sesion, Venta,
};
use crate::security;
use crate::servicios::{admin, almacen, catalogo, ventas};
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub struct AppState {
    pub db: Mutex<Db>,
    pub sesion: Mutex<Option<Sesion>>,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self {
            db: Mutex::new(db),
            sesion: Mutex::new(None),
        }
    }
}

fn bloqueo(state: &AppState) -> Result<MutexGuard<'_, Db>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::Validation("Base de datos ocupada".into()))
}

fn sesion(state: &AppState, roles: &[&str]) -> Result<Sesion> {
    let sesion = state
        .sesion
        .lock()
        .map_err(|_| AppError::Validation("Sesión bloqueada".into()))?
        .clone()
        .ok_or_else(|| AppError::Validation("Debe iniciar sesión".into()))?;
    if !roles.is_empty() && !roles.contains(&sesion.rol.as_str()) {
        return Err(AppError::Validation(
            "Su rol no tiene permiso para esta operación".into(),
        ));
    }
    Ok(sesion)
}

const ADMIN: &[&str] = &["ADMINISTRADOR"];
const ADMIN_ALMACEN: &[&str] = &["ADMINISTRADOR", "ALMACENERO"];
const ADMIN_CAJERO: &[&str] = &["ADMINISTRADOR", "CAJERO"];
const TODOS: &[&str] = &[];

#[tauri::command]
pub fn iniciar_sesion(
    ci: String,
    contrasena: String,
    rol: String,
    state: State<'_, AppState>,
) -> std::result::Result<Sesion, String> {
    let db = bloqueo(&state)?;
    let registro: Option<(String, String, String, String)> = db
        .conn
        .query_row(
            "SELECT ci, nombre, rol, contrasena_hash FROM empleados WHERE lower(ci) = lower(?1)",
            params![ci.trim()],
            |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?, fila.get(3)?)),
        )
        .optional()
        .map_err(AppError::from)?;
    let Some((ci_real, nombre, rol_real, hash)) = registro else {
        return Err("Credenciales incorrectas".into());
    };
    if !security::verify_password(&contrasena, &hash) {
        return Err("Credenciales incorrectas".into());
    }
    let rol_solicitado = match rol.trim().to_ascii_uppercase().as_str() {
        "ADMIN" | "ADMINISTRADOR" => "ADMINISTRADOR",
        "CAJERO" | "VENDEDOR" => "CAJERO",
        "ALMACEN" | "ALMACENERO" => "ALMACENERO",
        _ => "",
    };
    if rol_solicitado != rol_real {
        return Err("El rol seleccionado no corresponde a este usuario".into());
    }
    let sesion = Sesion {
        ci: ci_real.clone(),
        nombre,
        rol: rol_real,
    };
    *state
        .sesion
        .lock()
        .map_err(|_| AppError::Validation("Sesión bloqueada".into()))? = Some(sesion.clone());
    db.auditar(&ci_real, "INICIAR_SESION", &sesion.rol)?;
    Ok(sesion)
}

#[tauri::command]
pub fn cerrar_sesion(state: State<'_, AppState>) -> std::result::Result<(), String> {
    *state
        .sesion
        .lock()
        .map_err(|_| AppError::Validation("Sesión bloqueada".into()))? = None;
    Ok(())
}

#[tauri::command]
pub fn sesion_actual(state: State<'_, AppState>) -> std::result::Result<Option<Sesion>, String> {
    let sesion = state
        .sesion
        .lock()
        .map_err(|_| AppError::Validation("Sesión bloqueada".into()))?
        .clone();
    Ok(sesion)
}

#[tauri::command]
pub fn listar_productos(
    termino: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Producto>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    catalogo::listar_productos(&db.conn, termino.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn listar_categorias(state: State<'_, AppState>) -> std::result::Result<Vec<String>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    catalogo::categorias(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn productos_criticos(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Producto>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    catalogo::criticos(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn crear_producto(
    entrada: ProductoInput,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, ADMIN_ALMACEN)?;
    let mut db = bloqueo(&state)?;
    almacen::crear_producto_con_entrada(&mut db.conn, &entrada, None, &sesion.ci)
        .map_err(Into::into)
}

#[tauri::command]
pub fn eliminar_producto(
    id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    sesion(&state, ADMIN_ALMACEN)?;
    let db = bloqueo(&state)?;
    catalogo::eliminar_producto(&db.conn, &id).map_err(Into::into)
}

#[tauri::command]
pub fn listar_proveedores(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Proveedor>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    catalogo::listar_proveedores(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn crear_proveedor(
    entrada: ProveedorInput,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    let id = catalogo::crear_proveedor(&db.conn, &entrada)?;
    db.auditar(&sesion.ci, "CREAR_PROVEEDOR", &id)?;
    Ok(id)
}

#[tauri::command]
pub fn eliminar_proveedor(
    id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    catalogo::eliminar_proveedor(&db.conn, &id)?;
    db.auditar(&sesion.ci, "ELIMINAR_PROVEEDOR", &id)?;
    Ok(())
}

#[tauri::command]
pub fn listar_clientes(state: State<'_, AppState>) -> std::result::Result<Vec<Cliente>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    catalogo::listar_clientes(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn crear_cliente(
    entrada: ClienteInput,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    let id = catalogo::guardar_cliente(&db.conn, &entrada)?;
    db.auditar(&sesion.ci, "CREAR_CLIENTE", &id)?;
    Ok(id)
}

#[tauri::command]
pub fn crear_orden(
    entrada: OrdenInput,
    state: State<'_, AppState>,
) -> std::result::Result<OrdenCreada, String> {
    let sesion = sesion(&state, ADMIN_CAJERO)?;
    let mut db = bloqueo(&state)?;
    ventas::crear_orden(&mut db.conn, &entrada, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn confirmar_orden(
    id_orden: String,
    state: State<'_, AppState>,
) -> std::result::Result<ResultadoVenta, String> {
    let sesion = sesion(&state, ADMIN_CAJERO)?;
    let mut db = bloqueo(&state)?;
    ventas::confirmar_orden(&mut db.conn, &id_orden, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn anular_venta(
    referencia: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN_CAJERO)?;
    let mut db = bloqueo(&state)?;
    ventas::anular(&mut db.conn, &referencia, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn listar_ventas(
    estado: Option<String>,
    fecha: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Venta>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    ventas::listar(&db.conn, estado.as_deref(), fecha.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn consultar_garantia(
    referencia: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<String>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    ventas::consultar_garantia(&db.conn, &referencia).map_err(Into::into)
}

#[tauri::command]
pub fn factura_orden(
    id_orden: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    ventas::factura(&db.conn, &id_orden).map_err(Into::into)
}

#[tauri::command]
pub fn importar_ordenes(
    ruta: String,
    state: State<'_, AppState>,
) -> std::result::Result<usize, String> {
    let sesion = sesion(&state, ADMIN_CAJERO)?;
    let mut db = bloqueo(&state)?;
    ventas::importar_ordenes_csv(&mut db.conn, &PathBuf::from(ruta), &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn listar_entradas(
    fecha: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Entrada>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    almacen::listar_entradas(&db.conn, fecha.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn registrar_entrada(
    id_producto: String,
    cantidad: i64,
    precio: f64,
    proveedor: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, ADMIN_ALMACEN)?;
    let mut db = bloqueo(&state)?;
    almacen::registrar_entrada(
        &mut db.conn,
        &id_producto,
        cantidad,
        precio,
        proveedor.as_deref(),
        &sesion.ci,
    )
    .map_err(Into::into)
}

#[tauri::command]
pub fn importar_entradas(
    ruta: String,
    state: State<'_, AppState>,
) -> std::result::Result<usize, String> {
    let sesion = sesion(&state, ADMIN_ALMACEN)?;
    let mut db = bloqueo(&state)?;
    almacen::importar_entradas_csv(&mut db.conn, &PathBuf::from(ruta), &sesion.ci)
        .map_err(Into::into)
}

#[tauri::command]
pub fn estado_caja(state: State<'_, AppState>) -> std::result::Result<Caja, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    admin::estado_caja(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn abrir_caja(state: State<'_, AppState>) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    admin::abrir_caja(&mut db.conn, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn cerrar_caja(state: State<'_, AppState>) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    admin::cerrar_caja(&mut db.conn, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn listar_empleados(state: State<'_, AppState>) -> std::result::Result<Vec<Empleado>, String> {
    sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    admin::listar_empleados(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn crear_empleado(
    entrada: EmpleadoInput,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    let tx = db.conn.transaction().map_err(AppError::from)?;
    admin::crear_empleado(&tx, &entrada, &sesion.ci)?;
    tx.commit().map_err(AppError::from)?;
    Ok(())
}

#[tauri::command]
pub fn eliminar_empleado(
    ci: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let sesion = sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    let tx = db.conn.transaction().map_err(AppError::from)?;
    admin::eliminar_empleado(&tx, &ci, &sesion.ci)?;
    tx.commit().map_err(AppError::from)?;
    Ok(())
}

#[tauri::command]
pub fn crear_pedido(
    id_proveedor: String,
    items: Vec<ItemCarrito>,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    let tx = db.conn.transaction().map_err(AppError::from)?;
    let id = admin::crear_pedido(&tx, &id_proveedor, &items, &sesion.ci)?;
    tx.commit().map_err(AppError::from)?;
    Ok(id)
}

#[tauri::command]
pub fn listar_pedidos(state: State<'_, AppState>) -> std::result::Result<Vec<Pedido>, String> {
    sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    admin::listar_pedidos(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn auditoria(
    limite: Option<i64>,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Auditoria>, String> {
    sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    admin::auditoria(&db.conn, limite.unwrap_or(100)).map_err(Into::into)
}

#[tauri::command]
pub fn estadisticas(state: State<'_, AppState>) -> std::result::Result<Estadisticas, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    admin::estadisticas(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn exportar_reporte(
    tipo: String,
    ruta: String,
    state: State<'_, AppState>,
) -> std::result::Result<usize, String> {
    sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    admin::exportar(&db.conn, &tipo, &PathBuf::from(ruta)).map_err(Into::into)
}

#[tauri::command]
pub fn importar_datos(
    ruta: String,
    state: State<'_, AppState>,
) -> std::result::Result<crate::db::MigrationReport, String> {
    sesion(&state, ADMIN)?;
    let mut db = bloqueo(&state)?;
    db.importar_csv(&PathBuf::from(ruta)).map_err(Into::into)
}
