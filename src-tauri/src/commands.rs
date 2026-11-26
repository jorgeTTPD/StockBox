use crate::csv_io;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::{
    Auditoria, Caja, Cliente, ClienteInput, ConfigAsistente, DashboardDatos, Devolucion, Empleado,
    EmpleadoInput, Entrada, Estadisticas, ItemCarrito, LineaVenta, OrdenCreada, OrdenInput, Pedido,
    Preferencias, Producto, ProductoInput, Proveedor, ProveedorInput, ResultadoVenta, Sesion,
    Venta, VistaPrevia,
};
use crate::servicios::{admin, almacen, asistente, auth, catalogo, dashboard, imagenes, ventas};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub struct AppState {
    pub db: Mutex<Db>,
    pub sesion: Mutex<Option<Sesion>>,
    pub ruta_datos: PathBuf,
    pub asistente: asistente::Asistente,
}

impl AppState {
    pub fn new(db: Db, ruta_datos: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            sesion: Mutex::new(None),
            ruta_datos,
            asistente: asistente::Asistente::new(),
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
    let sesion = auth::verificar(&db.conn, &ci, &contrasena, &rol)?;
    db.auditar(&sesion.ci, "INICIAR_SESION", &sesion.rol)?;
    *state
        .sesion
        .lock()
        .map_err(|_| AppError::Validation("Sesión bloqueada".into()))? = Some(sesion.clone());
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

#[tauri::command]
pub fn buscar_venta(
    id_venta: String,
    state: State<'_, AppState>,
) -> std::result::Result<Option<LineaVenta>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    ventas::buscar_linea(&db.conn, &id_venta).map_err(Into::into)
}

#[tauri::command]
pub fn devolver_venta(
    id_venta: String,
    cantidad: i64,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let sesion = sesion(&state, ADMIN_CAJERO)?;
    let mut db = bloqueo(&state)?;
    ventas::devolver_venta(&mut db.conn, &id_venta, cantidad, &sesion.ci).map_err(Into::into)
}

#[tauri::command]
pub fn listar_devoluciones(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<Devolucion>, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    ventas::listar_devoluciones(&db.conn).map_err(Into::into)
}

#[tauri::command]
pub fn dashboard(
    periodo: String,
    state: State<'_, AppState>,
) -> std::result::Result<DashboardDatos, String> {
    sesion(&state, ADMIN)?;
    let db = bloqueo(&state)?;
    dashboard::datos(&db.conn, &periodo).map_err(Into::into)
}

#[tauri::command]
pub fn vista_previa_csv(
    ruta: String,
    tipo: String,
    state: State<'_, AppState>,
) -> std::result::Result<VistaPrevia, String> {
    sesion(&state, TODOS)?;
    let _ = tipo;
    let (columnas, filas) = csv_io::leer(&PathBuf::from(ruta))?;
    Ok(VistaPrevia {
        columnas,
        filas: filas.iter().take(8).cloned().collect(),
        total: filas.len(),
    })
}

#[tauri::command]
pub fn ruta_imagen_producto(
    id_producto: String,
    state: State<'_, AppState>,
) -> std::result::Result<Option<String>, String> {
    sesion(&state, TODOS)?;
    Ok(imagenes::ruta_para_ui(&state.ruta_datos, &id_producto))
}

#[tauri::command]
pub fn rutas_imagenes(
    ids: Vec<String>,
    state: State<'_, AppState>,
) -> std::result::Result<HashMap<String, String>, String> {
    sesion(&state, TODOS)?;
    Ok(ids
        .into_iter()
        .filter_map(|id| imagenes::ruta_para_ui(&state.ruta_datos, &id).map(|ruta| (id, ruta)))
        .collect())
}

#[tauri::command]
pub fn guardar_imagen_producto(
    id_producto: String,
    ruta: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    sesion(&state, ADMIN_ALMACEN)?;
    let db = bloqueo(&state)?;
    if catalogo::buscar_producto(&db.conn, &id_producto)?.is_none() {
        return Err(AppError::NotFound(format!("Producto {id_producto}")).into());
    }
    drop(db);
    imagenes::guardar(&state.ruta_datos, &id_producto, &PathBuf::from(ruta)).map_err(Into::into)
}

#[tauri::command]
pub fn configuracion_asistente(
    state: State<'_, AppState>,
) -> std::result::Result<ConfigAsistente, String> {
    sesion(&state, ADMIN_CAJERO)?;
    Ok(state.asistente.config_para_ui(&state.ruta_datos))
}

#[tauri::command]
pub fn preguntar_asistente(
    pregunta: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    sesion(&state, ADMIN_CAJERO)?;
    let snapshot = {
        let db = bloqueo(&state)?;
        asistente::snapshot_inventario(&db.conn)?
    };
    state
        .asistente
        .preguntar(&state.ruta_datos, &snapshot, &pregunta)
        .map_err(Into::into)
}

#[tauri::command]
pub fn limpiar_asistente(state: State<'_, AppState>) -> std::result::Result<(), String> {
    sesion(&state, ADMIN_CAJERO)?;
    state.asistente.limpiar().map_err(Into::into)
}

fn leer_preferencias(db: &Db) -> Result<Preferencias> {
    Ok(Preferencias {
        tema_oscuro: db.meta("tema_oscuro")?.as_deref() != Some("false"),
        periodo_dashboard: db
            .meta("periodo_dashboard")?
            .unwrap_or_else(|| "ULTIMOS_7".into()),
    })
}

#[tauri::command]
pub fn preferencias(state: State<'_, AppState>) -> std::result::Result<Preferencias, String> {
    sesion(&state, TODOS)?;
    let db = bloqueo(&state)?;
    leer_preferencias(&db).map_err(Into::into)
}

#[tauri::command]
pub fn guardar_preferencias(
    tema_oscuro: Option<bool>,
    periodo: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<Preferencias, String> {
    sesion(&state, TODOS)?;
    if let Some(periodo) = periodo.as_deref() {
        if !matches!(periodo, "HOY" | "ULTIMOS_7" | "ULTIMOS_30" | "TODO") {
            return Err("Período no reconocido".into());
        }
    }
    let db = bloqueo(&state)?;
    if let Some(tema) = tema_oscuro {
        db.set_meta("tema_oscuro", &tema.to_string())?;
    }
    if let Some(periodo) = periodo {
        db.set_meta("periodo_dashboard", &periodo)?;
    }
    leer_preferencias(&db).map_err(Into::into)
}
