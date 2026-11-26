use rusqlite::params;
use stockbox_lib::db::Db;
use stockbox_lib::security;

#[test]
fn la_siembra_importa_los_nueve_csv() {
    let mut db = Db::memory().unwrap();
    let informe = db.seed_if_empty().unwrap();

    assert_eq!(informe.productos, 164);
    assert_eq!(informe.entradas, 455);
    assert_eq!(informe.empleados, 3);
    assert_eq!(informe.proveedores, 2);
    assert_eq!(informe.clientes, 8);
    assert_eq!(informe.ventas, 27);
    assert_eq!(informe.garantias, 15);
    assert_eq!(informe.caja, 1);
    assert_eq!(informe.filas_omitidas, 0);

    let total_ventas: f64 = db
        .conn
        .query_row("SELECT COALESCE(SUM(total), 0) FROM ventas", [], |fila| {
            fila.get(0)
        })
        .unwrap();
    assert!(total_ventas > 0.0);

    let items: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM venta_items", [], |fila| fila.get(0))
        .unwrap();
    assert_eq!(items, 27);
}

#[test]
fn las_contrasenas_se_guardan_con_hash() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();

    let hash: String = db
        .conn
        .query_row(
            "SELECT contrasena_hash FROM empleados WHERE ci = ?1",
            params!["ADM-001"],
            |fila| fila.get(0),
        )
        .unwrap();

    assert!(hash.starts_with("$argon2id$"));
    assert!(!hash.contains("user"));
    assert!(security::verify_password("user", &hash));
    assert!(!security::verify_password("incorrecta", &hash));
}

#[test]
fn reimportar_no_duplica_registros() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    let segundo = db.seed_if_empty().unwrap();

    assert_eq!(segundo.productos, 0);
    assert_eq!(segundo.ventas, 0);

    let productos: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM productos", [], |fila| fila.get(0))
        .unwrap();
    assert_eq!(productos, 164);
}

#[test]
fn el_esquema_exige_precios_y_stock_validos() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();

    let resultado = db.conn.execute(
        "INSERT INTO productos(id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock) VALUES('MAL', 'x', 'x', 'x', 'x', -1, 2, 3)",
        [],
    );
    assert!(resultado.is_err());
}
