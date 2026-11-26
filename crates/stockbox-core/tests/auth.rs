use stockbox_core::db::Db;
use stockbox_core::servicios::auth;

#[test]
fn verifica_credenciales_de_prueba() {
    let mut db = Db::memory().unwrap();
    db.seed_if_empty().unwrap();
    let sesion = auth::verificar(&db.conn, "ADM-001", "user", "ADMINISTRADOR").unwrap();
    assert_eq!(sesion.ci, "ADM-001");
    assert_eq!(sesion.rol, "ADMINISTRADOR");
}
