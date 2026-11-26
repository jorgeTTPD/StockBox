use crate::csv_io;
use crate::error::{AppError, Result};
use crate::models::{
    ClienteInput, Devolucion, ItemCarrito, ItemVenta, LineaVenta, OrdenCreada, OrdenInput,
    ResultadoVenta, Venta,
};
use crate::security::generar_id;
use crate::servicios::{auditar, catalogo};
use crate::validacion;
use chrono::{Local, Months, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::HashMap;
use std::path::Path;

const COLUMNAS_VENTA: &str =
    "id_venta, id_orden, estado, nombre_cliente, telefono_cliente, id_garantia, total, fecha_venta, ci_empleado";

#[derive(Clone)]
struct LineaOrden {
    id_producto: String,
    cantidad: i64,
}

fn map_venta(fila: &Row<'_>) -> rusqlite::Result<Venta> {
    Ok(Venta {
        id_venta: fila.get(0)?,
        id_orden: fila.get(1)?,
        estado: fila.get(2)?,
        nombre_cliente: fila.get(3)?,
        telefono_cliente: fila.get(4)?,
        id_garantia: fila.get(5)?,
        total: fila.get(6)?,
        fecha_venta: fila.get(7)?,
        ci_empleado: fila.get(8)?,
        items: Vec::new(),
    })
}

pub fn caja_abierta(conn: &Connection) -> Result<bool> {
    let estado: Option<String> = conn
        .query_row("SELECT estado FROM caja WHERE id = 1", [], |fila| {
            fila.get(0)
        })
        .optional()?;
    Ok(estado.as_deref() == Some("ABIERTA"))
}

pub fn crear_orden(conn: &mut Connection, entrada: &OrdenInput, ci: &str) -> Result<OrdenCreada> {
    let tx = conn.transaction()?;
    let creada = crear_orden_en(&tx, entrada, ci)?;
    tx.commit()?;
    Ok(creada)
}

fn crear_orden_en(conn: &Connection, entrada: &OrdenInput, ci: &str) -> Result<OrdenCreada> {
    let nombre = entrada.nombre_cliente.trim();
    let ci_cliente = entrada.ci_cliente.trim();
    let telefono = entrada.telefono_cliente.trim();
    if !validacion::texto_valido(nombre) {
        return Err(AppError::Validation(
            "El nombre del cliente no es válido".into(),
        ));
    }
    if !ci_cliente.is_empty() && !validacion::identificador_valido(&ci_cliente) {
        return Err(AppError::Validation(
            "La cédula del cliente no es válida".into(),
        ));
    }
    if !telefono.is_empty() && !validacion::telefono_valido(&telefono) {
        return Err(AppError::Validation(
            "El teléfono del cliente no es válido".into(),
        ));
    }
    if entrada.items.is_empty() {
        return Err(AppError::Validation("El carrito está vacío".into()));
    }

    let mut orden_lineas: Vec<LineaOrden> = Vec::new();
    let mut cantidades: HashMap<String, i64> = HashMap::new();
    for item in &entrada.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validation(
                "La cantidad debe ser mayor que cero".into(),
            ));
        }
        let id = item.id_producto.trim();
        if catalogo::buscar_producto(conn, id)?.is_none() {
            return Err(AppError::NotFound(format!("Producto {id}")));
        }
        let acumulado = cantidades.entry(id.to_string()).or_insert(0);
        *acumulado += item.cantidad;
        if !orden_lineas.iter().any(|linea| linea.id_producto == id) {
            orden_lineas.push(LineaOrden {
                id_producto: id.to_string(),
                cantidad: item.cantidad,
            });
        }
    }
    for linea in &mut orden_lineas {
        linea.cantidad = cantidades[&linea.id_producto];
        let producto = catalogo::buscar_producto(conn, &linea.id_producto)?.unwrap();
        if producto.stock < linea.cantidad {
            return Err(AppError::Conflict(format!(
                "Stock insuficiente de {} (disponible {})",
                producto.nombre, producto.stock
            )));
        }
    }

    let id_orden = generar_id("ORD");
    conn.execute(
        "INSERT INTO ordenes(id_orden, estado, nombre_cliente, ci_cliente, telefono_cliente, fecha, ci_empleado) VALUES(?1, 'PROFORMA', ?2, ?3, ?4, ?5, ?6)",
        params![id_orden, nombre, ci_cliente, telefono, crate::security::ahora(), ci],
    )?;

    let mut total = 0.0;
    for linea in &orden_lineas {
        let producto = catalogo::buscar_producto(conn, &linea.id_producto)?.unwrap();
        let subtotal = producto.precio_venta * linea.cantidad as f64;
        total += subtotal;
        let id_venta = generar_id("VEN");
        conn.execute(
            "INSERT INTO ventas(id_venta, id_orden, estado, nombre_cliente, telefono_cliente, id_garantia, total, fecha_venta, ci_empleado) VALUES(?1, ?2, 'PROFORMA', ?3, ?4, '', ?5, '', ?6)",
            params![id_venta, id_orden, nombre, telefono, subtotal, ci],
        )?;
        conn.execute(
            "INSERT INTO venta_items(id_venta, id_producto, nombre_producto, cantidad, precio_unitario, subtotal) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![id_venta, producto.id_producto, producto.nombre, linea.cantidad, producto.precio_venta, subtotal],
        )?;
    }
    auditar(
        conn,
        ci,
        "CREAR_ORDEN",
        &format!("{id_orden} | {nombre} | {total:.2}"),
    )?;
    Ok(OrdenCreada {
        id_orden,
        total,
        lineas: orden_lineas.len(),
    })
}

pub fn confirmar_orden(conn: &mut Connection, id_orden: &str, ci: &str) -> Result<ResultadoVenta> {
    let tx = conn.transaction()?;
    if !caja_abierta(&tx)? {
        return Err(AppError::Validation(
            "La caja debe estar abierta para confirmar".into(),
        ));
    }
    let estado: Option<String> = tx
        .query_row(
            "SELECT estado FROM ordenes WHERE id_orden = ?1",
            params![id_orden],
            |fila| fila.get(0),
        )
        .optional()?;
    match estado.as_deref() {
        Some("PROFORMA") => {}
        Some("CONFIRMADA") => return Err(AppError::Conflict("La orden ya está confirmada".into())),
        Some("ANULADA") => return Err(AppError::Conflict("La orden está anulada".into())),
        Some(_) => return Err(AppError::Validation("Estado de orden desconocido".into())),
        None => return Err(AppError::NotFound(format!("Orden {id_orden}"))),
    }

    let datos: Vec<(String, String, i64)> = {
        let mut sentencia = tx.prepare(
            "SELECT v.id_venta, vi.id_producto, vi.cantidad \
             FROM ventas v JOIN venta_items vi ON vi.id_venta = v.id_venta \
             WHERE v.id_orden = ?1 ORDER BY v.id_venta, vi.id",
        )?;
        let filas = sentencia.query_map(params![id_orden], |fila| {
            Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?))
        })?;
        let mut resultado = Vec::new();
        for fila in filas {
            resultado.push(fila?);
        }
        resultado
    };
    if datos.is_empty() {
        return Err(AppError::Validation("La orden no tiene productos".into()));
    }

    for (_, id_producto, cantidad) in &datos {
        let stock: i64 = tx.query_row(
            "SELECT stock FROM productos WHERE lower(id_producto) = lower(?1)",
            params![id_producto],
            |fila| fila.get(0),
        )?;
        if stock < *cantidad {
            return Err(AppError::Conflict(format!(
                "Stock insuficiente de {id_producto} (disponible {stock})"
            )));
        }
    }
    for (_, id_producto, cantidad) in &datos {
        tx.execute(
            "UPDATE productos SET stock = stock - ?1 WHERE lower(id_producto) = lower(?2)",
            params![cantidad, id_producto],
        )?;
    }

    let (nombre, ci_cliente, telefono): (String, String, String) = tx.query_row(
        "SELECT nombre_cliente, ci_cliente, telefono_cliente FROM ordenes WHERE id_orden = ?1",
        params![id_orden],
        |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?)),
    )?;
    let id_cliente = catalogo::guardar_cliente(
        &tx,
        &ClienteInput {
            nombre_cliente: nombre.clone(),
            ci: ci_cliente,
            telefono: telefono.clone(),
        },
    )?;

    let hoy = Local::now().date_naive();
    let fin = hoy
        .checked_add_months(Months::new(12))
        .unwrap_or(hoy)
        .to_string();
    let fecha = crate::security::ahora();
    let mut id_ventas = Vec::new();
    let mut id_garantias = Vec::new();

    for (id_venta, id_producto, _) in &datos {
        let id_garantia = generar_id("GAR");
        let (nombre_producto, ci_empleado): (String, String) = tx.query_row(
            "SELECT vi.nombre_producto, v.ci_empleado FROM venta_items vi \
             JOIN ventas v ON v.id_venta = vi.id_venta WHERE vi.id_venta = ?1",
            params![id_venta],
            |fila| Ok((fila.get(0)?, fila.get(1)?)),
        )?;
        tx.execute(
            "INSERT INTO garantias(id_garantia, id_venta, id_orden, id_cliente, nombre_cliente, telefono_cliente, id_producto, nombre_producto, fecha_inicio, fecha_fin, estado, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'VIGENTE', ?11)",
            params![id_garantia, id_venta, id_orden, id_cliente, nombre, telefono, id_producto, nombre_producto, hoy.to_string(), fin, ci_empleado],
        )?;
        tx.execute(
            "UPDATE ventas SET estado = 'CONFIRMADA', fecha_venta = ?2, id_cliente = ?3, telefono_cliente = ?4, id_garantia = ?5 WHERE id_venta = ?1",
            params![id_venta, fecha, id_cliente, telefono, id_garantia],
        )?;
        id_ventas.push(id_venta.clone());
        id_garantias.push(id_garantia);
    }
    tx.execute(
        "UPDATE ordenes SET estado = 'CONFIRMADA' WHERE id_orden = ?1",
        params![id_orden],
    )?;
    let factura = factura_de(&tx, id_orden)?;
    let total: f64 = tx.query_row(
        "SELECT COALESCE(SUM(total), 0) FROM ventas WHERE id_orden = ?1 AND estado = 'CONFIRMADA'",
        params![id_orden],
        |fila| fila.get(0),
    )?;
    auditar(
        &tx,
        ci,
        "CONFIRMAR_VENTA",
        &format!("{id_orden} | {total:.2}"),
    )?;
    tx.commit()?;
    Ok(ResultadoVenta {
        id_orden: id_orden.to_string(),
        id_ventas,
        id_garantias,
        total,
        factura,
    })
}

pub fn anular(conn: &mut Connection, referencia: &str, ci: &str) -> Result<()> {
    let referencia = referencia.trim();
    if referencia.is_empty() {
        return Err(AppError::Validation(
            "Debe indicar una venta u orden".into(),
        ));
    }
    let tx = conn.transaction()?;
    let registros: Vec<(String, String, String)> = {
        let mut sentencia = tx.prepare(
            "SELECT id_venta, id_orden, estado FROM ventas \
             WHERE lower(id_venta) = lower(?1) OR lower(id_orden) = lower(?1)",
        )?;
        let filas = sentencia.query_map(params![referencia], |fila| {
            Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?))
        })?;
        let mut resultado = Vec::new();
        for fila in filas {
            resultado.push(fila?);
        }
        resultado
    };
    if registros.is_empty() {
        return Err(AppError::NotFound(format!("Venta u orden {referencia}")));
    }
    if registros.iter().any(|(_, _, estado)| estado == "ANULADA") {
        return Err(AppError::Conflict("La venta ya está anulada".into()));
    }

    for (id_venta, _, estado) in &registros {
        if estado == "CONFIRMADA" {
            let lineas: Vec<(String, i64)> = {
                let mut sentencia = tx
                    .prepare("SELECT id_producto, cantidad FROM venta_items WHERE id_venta = ?1")?;
                let filas = sentencia
                    .query_map(params![id_venta], |fila| Ok((fila.get(0)?, fila.get(1)?)))?;
                let mut resultado = Vec::new();
                for fila in filas {
                    resultado.push(fila?);
                }
                resultado
            };
            for (id_producto, cantidad) in lineas {
                tx.execute(
                    "UPDATE productos SET stock = stock + ?1 WHERE lower(id_producto) = lower(?2)",
                    params![cantidad, id_producto],
                )?;
            }
        }
        tx.execute(
            "UPDATE ventas SET estado = 'ANULADA' WHERE id_venta = ?1",
            params![id_venta],
        )?;
        tx.execute(
            "UPDATE garantias SET estado = 'ANULADA' WHERE id_venta = ?1",
            params![id_venta],
        )?;
    }
    for id_orden in {
        let mut ordenes = Vec::new();
        for (_, id_orden, _) in &registros {
            if !ordenes.contains(id_orden) {
                ordenes.push(id_orden.clone());
            }
        }
        ordenes
    } {
        let pendientes: i64 = tx.query_row(
            "SELECT COUNT(*) FROM ventas WHERE id_orden = ?1 AND estado <> 'ANULADA'",
            params![id_orden],
            |fila| fila.get(0),
        )?;
        if pendientes == 0 {
            tx.execute(
                "UPDATE ordenes SET estado = 'ANULADA' WHERE id_orden = ?1",
                params![id_orden],
            )?;
        }
    }
    auditar(&tx, ci, "ANULAR_VENTA", referencia)?;
    tx.commit()?;
    Ok(())
}

pub fn listar(conn: &Connection, estado: Option<&str>, fecha: Option<&str>) -> Result<Vec<Venta>> {
    let estado = estado.unwrap_or("");
    let fecha = fecha.unwrap_or("");
    let mut sentencia = conn.prepare(&format!(
        "SELECT {COLUMNAS_VENTA} FROM ventas \
         WHERE (?1 = '' OR estado = ?1) AND (?2 = '' OR substr(fecha_venta, 1, 10) = ?2) \
         ORDER BY fecha_venta DESC, id_venta DESC"
    ))?;
    let filas = sentencia.query_map(params![estado, fecha], map_venta)?;
    let mut ventas = Vec::new();
    for fila in filas {
        let mut venta = fila?;
        let mut items = conn.prepare(
            "SELECT id_producto, nombre_producto, cantidad, precio_unitario, subtotal \
             FROM venta_items WHERE id_venta = ?1 ORDER BY id",
        )?;
        let filas_items = items.query_map(params![venta.id_venta], |fila| {
            Ok(ItemVenta {
                id_producto: fila.get(0)?,
                nombre_producto: fila.get(1)?,
                cantidad: fila.get(2)?,
                precio_unitario: fila.get(3)?,
                subtotal: fila.get(4)?,
            })
        })?;
        for item in filas_items {
            venta.items.push(item?);
        }
        ventas.push(venta);
    }
    Ok(ventas)
}

pub fn consultar_garantia(conn: &Connection, referencia: &str) -> Result<Vec<String>> {
    let referencia = referencia.trim();
    if referencia.is_empty() {
        return Ok(vec![
            "Debe indicar un código de garantía, venta u orden.".into()
        ]);
    }
    let mut sentencia = conn.prepare(
        "SELECT id_garantia, id_venta, nombre_cliente, nombre_producto, fecha_inicio, fecha_fin, estado \
         FROM garantias WHERE lower(id_garantia) = lower(?1) \
         OR lower(id_venta) = lower(?1) OR lower(id_orden) = lower(?1) ORDER BY id_garantia",
    )?;
    let filas = sentencia.query_map(params![referencia], |fila| {
        Ok((
            fila.get::<_, String>(0)?,
            fila.get::<_, String>(1)?,
            fila.get::<_, String>(2)?,
            fila.get::<_, String>(3)?,
            fila.get::<_, String>(4)?,
            fila.get::<_, String>(5)?,
            fila.get::<_, String>(6)?,
        ))
    })?;
    let hoy = Local::now().date_naive();
    let mut resultado = Vec::new();
    for fila in filas {
        let (id, id_venta, cliente, producto, inicio, fin, estado) = fila?;
        let confirmada: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM ventas WHERE id_venta = ?1 AND estado = 'CONFIRMADA'",
                params![id_venta],
                |fila| fila.get(0),
            )
            .optional()?;
        let vigente = confirmada.is_some()
            && estado.eq_ignore_ascii_case("VIGENTE")
            && fecha_valida(&inicio, &fin, hoy);
        resultado.push(format!(
            "{id} | {producto} | Cliente: {cliente} | Inicio: {inicio} | Vence: {fin} | Estado: {}",
            if vigente { "VIGENTE" } else { "NO VIGENTE" }
        ));
    }
    if resultado.is_empty() {
        resultado.push(format!("No se encontraron garantías para: {referencia}"));
    }
    Ok(resultado)
}

fn fecha_valida(inicio: &str, fin: &str, hoy: NaiveDate) -> bool {
    let inicio = NaiveDate::parse_from_str(&inicio[..inicio.len().min(10)], "%Y-%m-%d");
    let fin = NaiveDate::parse_from_str(&fin[..fin.len().min(10)], "%Y-%m-%d");
    match (inicio, fin) {
        (Ok(inicio), Ok(fin)) => hoy >= inicio && hoy <= fin,
        _ => false,
    }
}

pub fn factura(conn: &Connection, id_orden: &str) -> Result<String> {
    factura_de(conn, id_orden)
}

fn factura_de(conn: &Connection, id_orden: &str) -> Result<String> {
    let (nombre, estado): (String, String) = conn.query_row(
        "SELECT nombre_cliente, estado FROM ordenes WHERE id_orden = ?1",
        params![id_orden],
        |fila| Ok((fila.get(0)?, fila.get(1)?)),
    )?;
    let mut lineas = vec![
        "=== FACTURA ===".to_string(),
        format!("Orden: {id_orden}"),
        format!("Cliente: {nombre}"),
        format!("Estado: {estado}"),
    ];
    let mut total = 0.0;
    let mut sentencia = conn.prepare(
        "SELECT vi.nombre_producto, vi.cantidad, vi.precio_unitario, vi.subtotal, v.id_garantia \
         FROM ventas v JOIN venta_items vi ON vi.id_venta = v.id_venta \
         WHERE v.id_orden = ?1 ORDER BY v.id_venta, vi.id",
    )?;
    let filas = sentencia.query_map(params![id_orden], |fila| {
        Ok((
            fila.get::<_, String>(0)?,
            fila.get::<_, i64>(1)?,
            fila.get::<_, f64>(2)?,
            fila.get::<_, f64>(3)?,
            fila.get::<_, String>(4)?,
        ))
    })?;
    for fila in filas {
        let (nombre_producto, cantidad, precio, subtotal, garantia) = fila?;
        total += subtotal;
        lineas.push(format!(
            "{nombre_producto} x{cantidad} | {precio:.2} = {subtotal:.2}"
        ));
        if !garantia.is_empty() {
            if let Some(fin) = conn
                .query_row(
                    "SELECT fecha_fin FROM garantias WHERE id_garantia = ?1",
                    params![garantia],
                    |fila| fila.get::<_, String>(0),
                )
                .optional()?
            {
                lineas.push(format!("Garantía {garantia} válida hasta {fin}"));
            }
        }
    }
    lineas.push(format!("TOTAL: ${total:.2}"));
    lineas.push("================".to_string());
    Ok(lineas.join("\n"))
}

pub fn buscar_linea(conn: &Connection, id_venta: &str) -> Result<Option<LineaVenta>> {
    conn.query_row(
        "SELECT v.id_venta, v.id_orden, v.estado, vi.id_producto, vi.nombre_producto, vi.cantidad, vi.precio_unitario, vi.subtotal \
         FROM ventas v JOIN venta_items vi ON vi.id_venta = v.id_venta \
         WHERE lower(v.id_venta) = lower(?1) LIMIT 1",
        params![id_venta],
        |fila| {
            Ok(LineaVenta {
                id_venta: fila.get(0)?,
                id_orden: fila.get(1)?,
                estado: fila.get(2)?,
                id_producto: fila.get(3)?,
                nombre_producto: fila.get(4)?,
                cantidad: fila.get(5)?,
                precio_unitario: fila.get(6)?,
                total: fila.get(7)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub fn devolver_venta(
    conn: &mut Connection,
    id_venta: &str,
    cantidad: i64,
    ci: &str,
) -> Result<String> {
    if id_venta.trim().is_empty() || cantidad <= 0 {
        return Err(AppError::Validation(
            "Indica la venta y una cantidad mayor que cero".into(),
        ));
    }
    let tx = conn.transaction()?;
    let linea = buscar_linea(&tx, id_venta)?
        .ok_or_else(|| AppError::NotFound(format!("Venta {id_venta}")))?;
    if linea.estado != "CONFIRMADA" {
        return Err(AppError::Conflict(
            "Solo se pueden devolver ventas confirmadas".into(),
        ));
    }
    if cantidad > linea.cantidad {
        return Err(AppError::Validation(format!(
            "No se pueden devolver más de {} unidades",
            linea.cantidad
        )));
    }
    if !validacion::fluido(linea.precio_unitario) {
        return Err(AppError::Validation("Precio unitario inválido".into()));
    }

    let restante = linea.cantidad - cantidad;
    let total_devuelto = cantidad as f64 * linea.precio_unitario;
    let id_devolucion = generar_id("DEV");
    tx.execute(
        "INSERT INTO devoluciones(id_devolucion, id_venta, id_orden, id_producto, nombre_producto, cantidad, precio_unitario, total, fecha, ci_empleado) \
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id_devolucion, linea.id_venta, linea.id_orden, linea.id_producto, linea.nombre_producto, cantidad, linea.precio_unitario, total_devuelto, crate::security::ahora(), ci],
    )?;
    tx.execute(
        "UPDATE productos SET stock = stock + ?1 WHERE lower(id_producto) = lower(?2)",
        params![cantidad, linea.id_producto],
    )?;

    if restante > 0 {
        tx.execute(
            "UPDATE venta_items SET cantidad = ?2, subtotal = ?3 WHERE id_venta = ?1",
            params![
                linea.id_venta,
                restante,
                restante as f64 * linea.precio_unitario
            ],
        )?;
        let total: f64 = tx.query_row(
            "SELECT COALESCE(SUM(subtotal), 0) FROM venta_items WHERE id_venta = ?1",
            params![linea.id_venta],
            |fila| fila.get(0),
        )?;
        tx.execute(
            "UPDATE ventas SET total = ?2 WHERE id_venta = ?1",
            params![linea.id_venta, total],
        )?;
    } else {
        tx.execute(
            "UPDATE ventas SET estado = 'ANULADA' WHERE id_venta = ?1",
            params![linea.id_venta],
        )?;
        tx.execute(
            "UPDATE garantias SET estado = 'ANULADA' WHERE id_venta = ?1",
            params![linea.id_venta],
        )?;
    }
    auditar(
        &tx,
        ci,
        "DEVOLUCION",
        &format!("{id_devolucion} | {} x{cantidad}", linea.id_producto),
    )?;
    tx.commit()?;
    Ok(id_devolucion)
}

pub fn listar_devoluciones(conn: &Connection) -> Result<Vec<Devolucion>> {
    let mut sentencia = conn.prepare(
        "SELECT id_devolucion, id_venta, id_orden, id_producto, nombre_producto, cantidad, precio_unitario, total, fecha, ci_empleado \
         FROM devoluciones ORDER BY fecha DESC, id_devolucion DESC",
    )?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Devolucion {
            id_devolucion: fila.get(0)?,
            id_venta: fila.get(1)?,
            id_orden: fila.get(2)?,
            id_producto: fila.get(3)?,
            nombre_producto: fila.get(4)?,
            cantidad: fila.get(5)?,
            precio_unitario: fila.get(6)?,
            total: fila.get(7)?,
            fecha: fila.get(8)?,
            ci_empleado: fila.get(9)?,
        })
    })?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn importar_ordenes_csv(conn: &mut Connection, ruta: &Path, ci: &str) -> Result<usize> {
    let (cabeceras, filas) = csv_io::leer(ruta)?;
    let requeridas = [
        "nombreCliente",
        "idProducto",
        "cantidad",
        "ciCliente",
        "telefonoCliente",
    ];
    for nombre in requeridas {
        if !cabeceras
            .iter()
            .any(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
        {
            return Err(AppError::Validation(format!("Falta la columna {nombre}")));
        }
    }
    let mut ordenes: Vec<(String, OrdenInput)> = Vec::new();
    let mut indice: HashMap<String, usize> = HashMap::new();
    for (numero, fila) in filas.iter().enumerate() {
        let valor = |nombre: &str| -> &str {
            cabeceras
                .iter()
                .position(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
                .and_then(|indice| fila.get(indice))
                .map(String::as_str)
                .unwrap_or("")
        };
        let id = {
            let candidato = valor("idOrden").trim();
            if candidato.is_empty() {
                format!("IMPORT-{numero}")
            } else {
                candidato.to_string()
            }
        };
        let cantidad = valor("cantidad").parse::<i64>().map_err(|_| {
            AppError::Validation(format!("Cantidad inválida en la fila {}", numero + 2))
        })?;
        if cantidad <= 0 {
            return Err(AppError::Validation(format!(
                "Cantidad inválida en la fila {}",
                numero + 2
            )));
        }
        let mut cliente = OrdenInput {
            nombre_cliente: valor("nombreCliente").to_string(),
            ci_cliente: valor("ciCliente").to_string(),
            telefono_cliente: valor("telefonoCliente").to_string(),
            items: vec![ItemCarrito {
                id_producto: valor("idProducto").to_string(),
                ..Default::default()
            }],
        };
        cliente.items[0].cantidad = cantidad;
        if let Some(&posicion) = indice.get(&id) {
            ordenes[posicion].1.items.push(cliente.items[0].clone());
        } else {
            indice.insert(id.clone(), ordenes.len());
            ordenes.push((id, cliente));
        }
    }
    if ordenes.is_empty() {
        return Err(AppError::Validation(
            "El archivo no contiene órdenes".into(),
        ));
    }
    let tx = conn.transaction()?;
    for (_, orden) in &ordenes {
        crear_orden_en(&tx, orden, ci)?;
    }
    auditar(
        &tx,
        ci,
        "IMPORTAR_ORDENES",
        &format!("{} órdenes", ordenes.len()),
    )?;
    tx.commit()?;
    Ok(ordenes.len())
}
