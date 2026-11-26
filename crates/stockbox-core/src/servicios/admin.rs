use crate::csv_io;
use crate::error::{AppError, Result};
use crate::models::{Auditoria, Caja, Empleado, EmpleadoInput, Estadisticas, Pedido};
use crate::security::{generar_id, hash_password, hoy};
use crate::servicios::auditar;
use crate::validacion;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::Path;

pub fn estado_caja(conn: &Connection) -> Result<Caja> {
    let caja = conn
        .query_row(
            "SELECT estado, fecha_apertura, fecha_cierre, ci_empleado FROM caja WHERE id = 1",
            [],
            |fila| {
                Ok(Caja {
                    estado: fila.get(0)?,
                    fecha_apertura: fila.get(1)?,
                    fecha_cierre: fila.get(2)?,
                    ci_empleado: fila.get(3)?,
                })
            },
        )
        .unwrap_or(Caja {
            estado: "CERRADA".into(),
            fecha_apertura: String::new(),
            fecha_cierre: String::new(),
            ci_empleado: String::new(),
        });
    Ok(caja)
}

pub fn abrir_caja(conn: &mut Connection, ci: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO caja(id, estado, fecha_apertura, fecha_cierre, ci_empleado) VALUES(1, 'ABIERTA', ?1, '', ?2) \
         ON CONFLICT(id) DO UPDATE SET estado = 'ABIERTA', fecha_apertura = excluded.fecha_apertura, fecha_cierre = '', ci_empleado = excluded.ci_empleado",
        params![hoy(), ci],
    )?;
    auditar(&tx, ci, "ABRIR_CAJA", &hoy())?;
    tx.commit()?;
    Ok(())
}

pub fn cerrar_caja(conn: &mut Connection, ci: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE caja SET estado = 'CERRADA', fecha_cierre = ?1, ci_empleado = ?2 WHERE id = 1",
        params![hoy(), ci],
    )?;
    auditar(&tx, ci, "CERRAR_CAJA", &hoy())?;
    tx.commit()?;
    Ok(())
}

pub fn listar_empleados(conn: &Connection) -> Result<Vec<Empleado>> {
    let mut sentencia =
        conn.prepare("SELECT ci, nombre, rol FROM empleados ORDER BY rol, nombre")?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Empleado {
            ci: fila.get(0)?,
            nombre: fila.get(1)?,
            rol: fila.get(2)?,
        })
    })?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn crear_empleado(conn: &Connection, entrada: &EmpleadoInput, ci: &str) -> Result<()> {
    let ci_nuevo = entrada.ci.trim();
    let nombre = entrada.nombre.trim();
    let rol = normalizar_rol(&entrada.rol)?;
    if !validacion::identificador_valido(ci_nuevo) || !validacion::texto_valido(nombre) {
        return Err(AppError::Validation(
            "CI o nombre del empleado no válidos".into(),
        ));
    }
    if !validacion::contrasena_valida(&entrada.contrasena) {
        return Err(AppError::Validation(
            "La contraseña debe tener al menos 6 caracteres".into(),
        ));
    }
    let duplicado: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM empleados WHERE lower(ci) = lower(?1) OR lower(nombre) = lower(?2)",
            params![ci_nuevo, nombre],
            |fila| fila.get(0),
        )
        .optional()?;
    if duplicado.is_some() {
        return Err(AppError::Conflict(
            "Ya existe un empleado con ese CI o nombre".into(),
        ));
    }
    let hash = hash_password(&entrada.contrasena)?;
    conn.execute(
        "INSERT INTO empleados(ci, nombre, rol, contrasena_hash) VALUES(?1, ?2, ?3, ?4)",
        params![ci_nuevo, nombre, rol, hash],
    )?;
    auditar(conn, ci, "CREAR_EMPLEADO", &format!("{ci_nuevo} | {rol}"))?;
    Ok(())
}

pub fn eliminar_empleado(conn: &Connection, ci_objetivo: &str, ci: &str) -> Result<()> {
    let rol: Option<String> = conn
        .query_row(
            "SELECT rol FROM empleados WHERE lower(ci) = lower(?1)",
            params![ci_objetivo],
            |fila| fila.get(0),
        )
        .optional()?;
    match rol.as_deref() {
        None => return Err(AppError::NotFound(format!("Empleado {ci_objetivo}"))),
        Some("ADMINISTRADOR") => {
            let administradores: i64 = conn.query_row(
                "SELECT COUNT(*) FROM empleados WHERE rol = 'ADMINISTRADOR'",
                [],
                |fila| fila.get(0),
            )?;
            if administradores <= 1 {
                return Err(AppError::Conflict(
                    "No se puede eliminar el último administrador".into(),
                ));
            }
        }
        _ => {}
    }
    conn.execute(
        "DELETE FROM empleados WHERE lower(ci) = lower(?1)",
        params![ci_objetivo],
    )?;
    auditar(conn, ci, "ELIMINAR_EMPLEADO", ci_objetivo)?;
    Ok(())
}

fn normalizar_rol(rol: &str) -> Result<&'static str> {
    match rol.trim().to_ascii_uppercase().as_str() {
        "ADMINISTRADOR" | "ADMIN" => Ok("ADMINISTRADOR"),
        "CAJERO" | "VENDEDOR" => Ok("CAJERO"),
        "ALMACENERO" | "ALMACEN" => Ok("ALMACENERO"),
        _ => Err(AppError::Validation("Rol no reconocido".into())),
    }
}

pub fn crear_pedido(
    conn: &Connection,
    id_proveedor: &str,
    items: &[crate::models::ItemCarrito],
    ci: &str,
) -> Result<String> {
    let proveedor = id_proveedor.trim();
    let existe: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM proveedores WHERE lower(id_proveedor) = lower(?1)",
            params![proveedor],
            |fila| fila.get(0),
        )
        .optional()?;
    if existe.is_none() {
        return Err(AppError::NotFound(format!("Proveedor {proveedor}")));
    }
    let id_pedido = generar_id("PED");
    let mut lineas = 0;
    for item in items {
        if item.cantidad <= 0 || item.id_producto.trim().is_empty() {
            continue;
        }
        conn.execute(
            "INSERT INTO pedidos(id_pedido, id_proveedor, id_producto, cantidad, fecha_pedido, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(id_pedido, id_producto) DO UPDATE SET cantidad = excluded.cantidad",
            params![id_pedido, proveedor, item.id_producto.trim(), item.cantidad, hoy(), ci],
        )?;
        lineas += 1;
    }
    if lineas == 0 {
        return Err(AppError::Validation(
            "El pedido no contiene productos".into(),
        ));
    }
    auditar(
        conn,
        ci,
        "CREAR_PEDIDO",
        &format!("{id_pedido} | {proveedor} | {lineas} líneas"),
    )?;
    Ok(id_pedido)
}

pub fn listar_pedidos(conn: &Connection) -> Result<Vec<Pedido>> {
    let mut sentencia = conn.prepare(
        "SELECT id_pedido, id_proveedor, id_producto, cantidad, fecha_pedido, ci_empleado \
         FROM pedidos ORDER BY fecha_pedido DESC, id_pedido",
    )?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Pedido {
            id_pedido: fila.get(0)?,
            id_proveedor: fila.get(1)?,
            id_producto: fila.get(2)?,
            cantidad: fila.get(3)?,
            fecha_pedido: fila.get(4)?,
            ci_empleado: fila.get(5)?,
        })
    })?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn auditoria(conn: &Connection, limite: i64) -> Result<Vec<Auditoria>> {
    let limite = limite.clamp(1, 500);
    let mut sentencia = conn.prepare(
        "SELECT id, fecha, ci_empleado, accion, detalle FROM auditoria ORDER BY id DESC LIMIT ?1",
    )?;
    let filas = sentencia.query_map(params![limite], map_auditoria)?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

fn map_auditoria(fila: &Row<'_>) -> rusqlite::Result<Auditoria> {
    Ok(Auditoria {
        id: fila.get(0)?,
        fecha: fila.get(1)?,
        ci_empleado: fila.get(2)?,
        accion: fila.get(3)?,
        detalle: fila.get(4)?,
    })
}

pub fn estadisticas(conn: &Connection) -> Result<Estadisticas> {
    let hoy = hoy();
    let fecha = &hoy[..hoy.len().min(10)];
    Ok(Estadisticas {
        productos: conn.query_row("SELECT COUNT(*) FROM productos", [], |fila| fila.get(0))?,
        stock_critico: conn.query_row("SELECT COUNT(*) FROM productos WHERE stock <= 5", [], |fila| fila.get(0))?,
        ventas_hoy: conn.query_row(
            "SELECT COALESCE(SUM(total), 0) FROM ventas WHERE estado = 'CONFIRMADA' AND substr(fecha_venta, 1, 10) = ?1",
            params![fecha],
            |fila| fila.get(0),
        )?,
        entradas_hoy: conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM entradas WHERE fecha_entrada = ?1",
            params![fecha],
            |fila| fila.get(0),
        )?,
        garantias_vigentes: conn.query_row(
            "SELECT COUNT(*) FROM garantias WHERE estado = 'VIGENTE' AND fecha_fin >= ?1",
            params![fecha],
            |fila| fila.get(0),
        )?,
    })
}

pub fn exportar(conn: &Connection, tipo: &str, ruta: &Path) -> Result<usize> {
    match tipo {
        "inventario" => {
            let mut sentencia = conn.prepare(
                "SELECT id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock FROM productos ORDER BY nombre",
            )?;
            let filas = sentencia.query_map([], |fila| {
                Ok(vec![
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                    fila.get::<_, String>(3)?,
                    fila.get::<_, String>(4)?,
                    format!("{:.2}", fila.get::<_, f64>(5)?),
                    format!("{:.2}", fila.get::<_, f64>(6)?),
                    fila.get::<_, i64>(7)?.to_string(),
                ])
            })?;
            let mut datos = Vec::new();
            for fila in filas {
                datos.push(fila?);
            }
            csv_io::escribir(
                ruta,
                &[
                    "idProducto",
                    "nombre",
                    "marca",
                    "categoria",
                    "descripcion",
                    "precioEntrada",
                    "precioVenta",
                    "stock",
                ],
                &datos,
            )
        }
        "ventas" => {
            let mut sentencia = conn.prepare(
                "SELECT v.id_venta, v.id_orden, v.estado, v.nombre_cliente, vi.id_producto, vi.nombre_producto, vi.cantidad, vi.precio_unitario, vi.subtotal, v.fecha_venta \
                 FROM ventas v JOIN venta_items vi ON vi.id_venta = v.id_venta ORDER BY v.fecha_venta DESC",
            )?;
            let filas = sentencia.query_map([], |fila| {
                Ok(vec![
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                    fila.get::<_, String>(3)?,
                    fila.get::<_, String>(4)?,
                    fila.get::<_, String>(5)?,
                    fila.get::<_, i64>(6)?.to_string(),
                    format!("{:.4}", fila.get::<_, f64>(7)?),
                    format!("{:.4}", fila.get::<_, f64>(8)?),
                    fila.get::<_, String>(9)?,
                ])
            })?;
            let mut datos = Vec::new();
            for fila in filas {
                datos.push(fila?);
            }
            csv_io::escribir(
                ruta,
                &[
                    "idVenta",
                    "idOrden",
                    "estado",
                    "nombreCliente",
                    "idProducto",
                    "nombreProducto",
                    "cantidad",
                    "precioUnitario",
                    "total",
                    "fechaVenta",
                ],
                &datos,
            )
        }
        "entradas" => {
            let mut sentencia = conn.prepare(
                "SELECT id_entrada, id_proveedor, id_producto, nombre_producto, cantidad, precio_unitario, fecha_entrada, ci_empleado FROM entradas ORDER BY fecha_entrada DESC",
            )?;
            let filas = sentencia.query_map([], |fila| {
                Ok(vec![
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                    fila.get::<_, String>(3)?,
                    fila.get::<_, i64>(4)?.to_string(),
                    format!("{:.2}", fila.get::<_, f64>(5)?),
                    fila.get::<_, String>(6)?,
                    fila.get::<_, String>(7)?,
                ])
            })?;
            let mut datos = Vec::new();
            for fila in filas {
                datos.push(fila?);
            }
            csv_io::escribir(
                ruta,
                &[
                    "idEntrada",
                    "idProveedor",
                    "idProducto",
                    "nombreProducto",
                    "cantidad",
                    "precioUnitario",
                    "fechaEntrada",
                    "ciEmpleado",
                ],
                &datos,
            )
        }
        _ => Err(AppError::Validation("Tipo de reporte no reconocido".into())),
    }
}
