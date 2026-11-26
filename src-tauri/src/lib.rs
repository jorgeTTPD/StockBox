pub mod commands;
pub mod csv_io;
pub mod db;
pub mod error;
pub mod models;
pub mod security;
pub mod servicios;
pub mod validacion;

use commands::AppState;
use db::Db;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let ruta = app.path().app_data_dir()?.join("stockbox.sqlite");
            let mut db = Db::open(&ruta)?;
            db.seed_if_empty()?;
            app.manage(AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::iniciar_sesion,
            commands::cerrar_sesion,
            commands::sesion_actual,
            commands::listar_productos,
            commands::listar_categorias,
            commands::productos_criticos,
            commands::crear_producto,
            commands::eliminar_producto,
            commands::listar_proveedores,
            commands::crear_proveedor,
            commands::eliminar_proveedor,
            commands::listar_clientes,
            commands::crear_cliente,
            commands::crear_orden,
            commands::confirmar_orden,
            commands::anular_venta,
            commands::listar_ventas,
            commands::consultar_garantia,
            commands::factura_orden,
            commands::importar_ordenes,
            commands::listar_entradas,
            commands::registrar_entrada,
            commands::importar_entradas,
            commands::estado_caja,
            commands::abrir_caja,
            commands::cerrar_caja,
            commands::listar_empleados,
            commands::crear_empleado,
            commands::eliminar_empleado,
            commands::crear_pedido,
            commands::listar_pedidos,
            commands::auditoria,
            commands::estadisticas,
            commands::exportar_reporte,
            commands::importar_datos,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar StockBox");
}
