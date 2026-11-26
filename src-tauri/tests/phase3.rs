use rusqlite::params;
use std::fs;
use stockbox_lib::db::Db;
use stockbox_lib::models::ItemCarrito;
use stockbox_lib::servicios::{admin, dashboard, imagenes, ventas};

fn db_con_venta(cantidad: i64) -> Db {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    db.conn
        .execute(
            "UPDATE productos SET stock = 100 WHERE lower(id_producto) = 'prod-01'",
            [],
        )
        .unwrap();
    admin::abrir_caja(&mut db.conn, "ADM-001").unwrap();
    let orden = ventas::crear_orden(
        &mut db.conn,
        &stockbox_lib::models::OrdenInput {
            nombre_cliente: "Cliente Devolucion".into(),
            ci_cliente: "1234567".into(),
            telefono_cliente: "76543210".into(),
            items: vec![ItemCarrito {
                id_producto: "Prod-01".into(),
                cantidad,
                ..Default::default()
            }],
        },
        "VEN-1",
    )
    .unwrap();
    ventas::confirmar_orden(&mut db.conn, &orden.id_orden, "VEN-1").unwrap();
    db
}

fn stock_de(db: &Db) -> i64 {
    db.conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap()
}

#[test]
fn devolucion_parcial_ajusta_linea_y_mantiene_garantia() {
    let mut db = db_con_venta(5);
    let antes = stock_de(&db);
    let venta: String = db
        .conn
        .query_row(
            "SELECT id_venta FROM ventas WHERE estado = 'CONFIRMADA' AND id_orden IN \
             (SELECT id_orden FROM ordenes WHERE nombre_cliente = 'Cliente Devolucion') LIMIT 1",
            [],
            |fila| fila.get(0),
        )
        .unwrap();

    let id_devolucion = ventas::devolver_venta(&mut db.conn, &venta, 2, "VEN-1").unwrap();
    assert!(id_devolucion.starts_with("DEV-"));
    assert_eq!(stock_de(&db), antes + 2);

    let (cantidad, estado): (i64, String) = db
        .conn
        .query_row(
            "SELECT vi.cantidad, v.estado FROM venta_items vi \
             JOIN ventas v ON v.id_venta = vi.id_venta WHERE vi.id_venta = ?1",
            params![venta],
            |fila| Ok((fila.get(0)?, fila.get(1)?)),
        )
        .unwrap();
    assert_eq!(cantidad, 3);
    assert_eq!(estado, "CONFIRMADA");

    let garantia: String = db
        .conn
        .query_row(
            "SELECT estado FROM garantias WHERE id_venta = ?1",
            params![venta],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(garantia, "VIGENTE");
    assert_eq!(ventas::listar_devoluciones(&db.conn).unwrap().len(), 1);
}

#[test]
fn devolucion_total_anula_venta_y_garantia() {
    let mut db = db_con_venta(2);
    let venta: String = db
        .conn
        .query_row(
            "SELECT id_venta FROM ventas WHERE estado = 'CONFIRMADA' AND id_orden IN \
             (SELECT id_orden FROM ordenes WHERE nombre_cliente = 'Cliente Devolucion') LIMIT 1",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    ventas::devolver_venta(&mut db.conn, &venta, 2, "VEN-1").unwrap();
    let estado: String = db
        .conn
        .query_row(
            "SELECT estado FROM ventas WHERE id_venta = ?1",
            params![venta],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(estado, "ANULADA");
    let garantia: String = db
        .conn
        .query_row(
            "SELECT estado FROM garantias WHERE id_venta = ?1",
            params![venta],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(garantia, "ANULADA");
}

#[test]
fn dashboard_calcula_totales_y_agrupaciones() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();

    let datos = dashboard::datos(&db.conn, "TODO").unwrap();
    assert!(datos.resumen.total > 0.0);
    assert!(datos.resumen.cantidad_ventas > 0);
    assert!(datos.resumen.unidades > 0);
    assert!(datos.resumen.ticket_promedio > 0.0);
    assert_eq!(datos.resumen.productos, 164);
    assert!(datos.resumen.valor_inventario > 0.0);
    assert!(!datos.ventas_por_categoria.is_empty());
    assert!(!datos.top_productos.is_empty());
    assert!(!datos.stock_por_categoria.is_empty());

    let hoy = dashboard::datos(&db.conn, "HOY").unwrap();
    assert_eq!(hoy.resumen.cantidad_ventas, 0);
}

#[test]
fn imagenes_producto_usa_convencion_de_archivos() {
    let dir = std::path::Path::new("/tmp/opencode");
    let origen = dir.join("stockbox_test_imagen.png");
    fs::write(&origen, "contenido de prueba").unwrap();

    let ruta = imagenes::guardar(dir, "Prod-01", &origen).unwrap();
    assert!(ruta.ends_with("imagenes/Prod-01.png"));
    assert!(imagenes::buscar(dir, "Prod-01").is_some());
    assert!(imagenes::ruta_para_ui(dir, "Prod-01").is_some());
    assert!(imagenes::guardar(dir, "Prod-01", &dir.join("mal.gif")).is_err());

    fs::remove_file(dir.join("imagenes/Prod-01.png")).unwrap();
    fs::remove_file(origen).unwrap();
}
