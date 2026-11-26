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
            let directorio = app.path().app_data_dir()?;
            let mut db = Db::open(&directorio.join("stockbox.sqlite"))?;
            db.seed_if_empty()?;
            let imagenes = directorio.join("imagenes");
            if !imagenes.exists() {
                let origen = app.path().resource_dir()?.join("imagenes");
                if origen.is_dir() {
                    std::fs::create_dir_all(&imagenes)?;
                    for entrada in std::fs::read_dir(origen)? {
                        let entrada = entrada?;
                        if entrada.path().is_file() {
                            std::fs::copy(entrada.path(), imagenes.join(entrada.file_name()))?;
                        }
                    }
                }
            }
            app.manage(AppState::new(db, directorio));
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
            commands::buscar_venta,
            commands::devolver_venta,
            commands::listar_devoluciones,
            commands::dashboard,
            commands::vista_previa_csv,
            commands::ruta_imagen_producto,
            commands::rutas_imagenes,
            commands::guardar_imagen_producto,
            commands::configuracion_asistente,
            commands::preguntar_asistente,
            commands::limpiar_asistente,
            commands::preferencias,
            commands::guardar_preferencias,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar StockBox");
}
