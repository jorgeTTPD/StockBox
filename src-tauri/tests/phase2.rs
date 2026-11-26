use rusqlite::params;
use std::fs;
use stockbox_lib::db::Db;
use stockbox_lib::models::{ClienteInput, ItemCarrito, OrdenInput, ProductoInput};
use stockbox_lib::servicios::{admin, almacen, catalogo, ventas};

fn db_con_datos() -> Db {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    db.conn
        .execute(
            "UPDATE productos SET stock = 100 WHERE lower(id_producto) = 'prod-01'",
            [],
        )
        .unwrap();
    db
}

fn orden_de(id_producto: &str, cantidad: i64) -> OrdenInput {
    OrdenInput {
        nombre_cliente: "Cliente Prueba".into(),
        ci_cliente: "1234567".into(),
        telefono_cliente: "76543210".into(),
        items: vec![ItemCarrito {
            id_producto: id_producto.into(),
            cantidad,
            ..Default::default()
        }],
    }
}

#[test]
fn confirmar_orden_descuenta_stock_y_crea_garantia() {
    let mut db = db_con_datos();
    admin::abrir_caja(&mut db.conn, "ADM-001").unwrap();

    let antes: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    let creada = ventas::crear_orden(&mut db.conn, &orden_de("Prod-01", 3), "VEN-1").unwrap();
    let resultado = ventas::confirmar_orden(&mut db.conn, &creada.id_orden, "VEN-1").unwrap();

    let despues: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(antes - despues, 3);
    assert_eq!(resultado.id_ventas.len(), 1);
    assert_eq!(resultado.id_garantias.len(), 1);
    assert!(resultado.factura.contains("TOTAL: $"));

    let garantias: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM garantias WHERE id_venta = ?1",
            params![resultado.id_ventas[0]],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(garantias, 1);
    let clientes: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM clientes", [], |fila| fila.get(0))
        .unwrap();
    assert_eq!(clientes, 9);
}

#[test]
fn confirmar_sin_caja_abierta_falla() {
    let mut db = db_con_datos();
    admin::cerrar_caja(&mut db.conn, "ADM-001").unwrap();
    let creada = ventas::crear_orden(&mut db.conn, &orden_de("Prod-01", 1), "VEN-1").unwrap();
    let resultado = ventas::confirmar_orden(&mut db.conn, &creada.id_orden, "VEN-1");
    assert!(resultado.is_err());
}

#[test]
fn anular_venta_devuelve_el_stock() {
    let mut db = db_con_datos();
    admin::abrir_caja(&mut db.conn, "ADM-001").unwrap();
    let antes: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    let creada = ventas::crear_orden(&mut db.conn, &orden_de("Prod-01", 2), "VEN-1").unwrap();
    let confirmada = ventas::confirmar_orden(&mut db.conn, &creada.id_orden, "VEN-1").unwrap();
    ventas::anular(&mut db.conn, &confirmada.id_ventas[0], "ADM-001").unwrap();

    let despues: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(despues, antes);
    let garantia: String = db
        .conn
        .query_row(
            "SELECT estado FROM garantias WHERE id_garantia = ?1",
            params![confirmada.id_garantias[0]],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(garantia, "ANULADA");
}

#[test]
fn el_stock_insuficiente_bloquea_la_orden() {
    let mut db = db_con_datos();
    let resultado = ventas::crear_orden(&mut db.conn, &orden_de("Prod-01", 101), "VEN-1");
    assert!(resultado.is_err());
}

#[test]
fn registrar_entrada_aumenta_stock_y_deja_constancia() {
    let mut db = db_con_datos();
    let antes: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    let id =
        almacen::registrar_entrada(&mut db.conn, "Prod-01", 7, 100.0, Some("PROV-01"), "ALM-1")
            .unwrap();
    assert!(id.starts_with("ENT-"));
    let despues: i64 = db
        .conn
        .query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = 'prod-01'",
            [],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(despues - antes, 7);
    assert_eq!(almacen::listar_entradas(&db.conn, None).unwrap().len(), 456);
}

#[test]
fn crear_producto_valida_precios() {
    let db = Db::memory().unwrap();
    let entrada = ProductoInput {
        id_producto: Some("NUEVO-1".into()),
        nombre: "Producto Nuevo".into(),
        marca: "Marca".into(),
        categoria: "Categoria".into(),
        descripcion: "Descripcion".into(),
        precio_entrada: 10.0,
        precio_venta: 5.0,
        stock: 1,
    };
    assert!(catalogo::crear_producto(&db.conn, &entrada).is_err());
}

#[test]
fn no_se_puede_eliminar_el_ultimo_administrador() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    assert!(admin::eliminar_empleado(&db.conn, "ADM-001", "ADM-001").is_err());
}

#[test]
fn guardar_cliente_reutiliza_identidad() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    let cliente = ClienteInput {
        nombre_cliente: "Nuevo Cliente".into(),
        ci: "9999999".into(),
        telefono: "71234567".into(),
    };
    let primero = catalogo::guardar_cliente(&db.conn, &cliente).unwrap();
    let segundo = catalogo::guardar_cliente(&db.conn, &cliente).unwrap();
    assert_eq!(primero, segundo);
}

#[test]
fn exportar_inventario_escribe_csv() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    let ruta = "/tmp/opencode/stockbox_test_export.csv";
    let filas = admin::exportar(&db.conn, "inventario", std::path::Path::new(ruta)).unwrap();
    assert_eq!(filas, 164);
    let contenido = fs::read_to_string(ruta).unwrap();
    assert!(contenido.starts_with("idProducto,nombre,marca"));
    fs::remove_file(ruta).unwrap();
}
