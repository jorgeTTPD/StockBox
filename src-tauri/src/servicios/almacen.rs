use crate::csv_io;
use crate::error::{AppError, Result};
use crate::models::{Entrada, ProductoInput};
use crate::security::generar_id;
use crate::servicios::{auditar, catalogo};
use crate::validacion;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::Path;

fn campo_csv<'a>(cabeceras: &[String], fila: &'a [String], nombre: &str) -> &'a str {
    cabeceras
        .iter()
        .position(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
        .and_then(|indice| fila.get(indice))
        .map(String::as_str)
        .unwrap_or("")
}

pub fn listar_entradas(conn: &Connection, fecha: Option<&str>) -> Result<Vec<Entrada>> {
    let fecha = fecha.unwrap_or("");
    let mut sentencia = conn.prepare(
        "SELECT id_entrada, id_proveedor, id_producto, nombre_producto, cantidad, precio_unitario, fecha_entrada, ci_empleado \
         FROM entradas WHERE ?1 = '' OR fecha_entrada = ?1 ORDER BY fecha_entrada DESC, id_entrada DESC",
    )?;
    let filas = sentencia.query_map(params![fecha], map_entrada)?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

fn map_entrada(fila: &Row<'_>) -> rusqlite::Result<Entrada> {
    Ok(Entrada {
        id_entrada: fila.get(0)?,
        id_proveedor: fila.get(1)?,
        id_producto: fila.get(2)?,
        nombre_producto: fila.get(3)?,
        cantidad: fila.get(4)?,
        precio_unitario: fila.get(5)?,
        fecha_entrada: fila.get(6)?,
        ci_empleado: fila.get(7)?,
    })
}

fn proveedor_por_defecto(conn: &Connection) -> String {
    conn.query_row(
        "SELECT id_proveedor FROM proveedores ORDER BY id_proveedor LIMIT 1",
        [],
        |fila| fila.get(0),
    )
    .unwrap_or_default()
}

fn registrar_entrada_en(
    conn: &Connection,
    id_producto: &str,
    cantidad: i64,
    precio: f64,
    proveedor: Option<&str>,
    ci: &str,
) -> Result<String> {
    let producto = catalogo::buscar_producto(conn, id_producto)?
        .ok_or_else(|| AppError::NotFound(format!("Producto {id_producto}")))?;
    if cantidad <= 0 {
        return Err(AppError::Validation(
            "La cantidad debe ser mayor que cero".into(),
        ));
    }
    if !validacion::fluido(precio) || precio <= 0.0 {
        return Err(AppError::Validation(
            "El precio unitario no es válido".into(),
        ));
    }
    let id_proveedor = proveedor
        .map(str::trim)
        .filter(|valor| !valor.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| proveedor_por_defecto(conn));
    let existe_proveedor: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM proveedores WHERE lower(id_proveedor) = lower(?1)",
            params![id_proveedor],
            |fila| fila.get(0),
        )
        .optional()?;
    if existe_proveedor.is_none() {
        return Err(AppError::NotFound(format!("Proveedor {id_proveedor}")));
    }

    conn.execute(
        "UPDATE productos SET stock = stock + ?1 WHERE lower(id_producto) = lower(?2)",
        params![cantidad, id_producto],
    )?;
    let id_entrada = generar_id("ENT");
    conn.execute(
        "INSERT INTO entradas(id_entrada, id_proveedor, id_producto, nombre_producto, cantidad, precio_unitario, fecha_entrada, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id_entrada, id_proveedor, producto.id_producto, producto.nombre, cantidad, precio, crate::security::hoy(), ci],
    )?;
    Ok(id_entrada)
}

pub fn registrar_entrada(
    conn: &mut Connection,
    id_producto: &str,
    cantidad: i64,
    precio: f64,
    proveedor: Option<&str>,
    ci: &str,
) -> Result<String> {
    let tx = conn.transaction()?;
    let id = registrar_entrada_en(&tx, id_producto, cantidad, precio, proveedor, ci)?;
    auditar(
        &tx,
        ci,
        "REGISTRAR_ENTRADA",
        &format!("{id} | {id_producto} | {cantidad}"),
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn crear_producto_con_entrada(
    conn: &mut Connection,
    entrada: &ProductoInput,
    proveedor: Option<&str>,
    ci: &str,
) -> Result<String> {
    let tx = conn.transaction()?;
    let id = catalogo::crear_producto(&tx, entrada)?;
    if entrada.stock > 0 {
        registrar_entrada_en(
            &tx,
            &id,
            entrada.stock,
            entrada.precio_entrada,
            proveedor,
            ci,
        )?;
    }
    auditar(
        &tx,
        ci,
        "CREAR_PRODUCTO",
        &format!("{id} | {}", entrada.nombre),
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn importar_entradas_csv(conn: &mut Connection, ruta: &Path, ci: &str) -> Result<usize> {
    let (cabeceras, filas) = csv_io::leer(ruta)?;
    for nombre in [
        "idProveedor",
        "idProducto",
        "nombreProducto",
        "marca",
        "categoria",
        "precio",
        "cantidad",
    ] {
        if !cabeceras
            .iter()
            .any(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
        {
            return Err(AppError::Validation(format!("Falta la columna {nombre}")));
        }
    }
    let tx = conn.transaction()?;
    let mut importadas = 0usize;
    for (numero, fila) in filas.iter().enumerate() {
        let id_producto = campo_csv(&cabeceras, fila, "idProducto").trim();
        let cantidad = campo_csv(&cabeceras, fila, "cantidad")
            .parse::<i64>()
            .map_err(|_| {
                AppError::Validation(format!("Cantidad inválida en la fila {}", numero + 2))
            })?;
        let precio = campo_csv(&cabeceras, fila, "precio")
            .parse::<f64>()
            .map_err(|_| {
                AppError::Validation(format!("Precio inválido en la fila {}", numero + 2))
            })?;
        let proveedor = campo_csv(&cabeceras, fila, "idProveedor").trim();
        if id_producto.is_empty() || cantidad <= 0 || precio <= 0.0 || proveedor.is_empty() {
            return Err(AppError::Validation(format!(
                "Fila {} incompleta",
                numero + 2
            )));
        }
        let existe_proveedor: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM proveedores WHERE lower(id_proveedor) = lower(?1)",
                params![proveedor],
                |fila| fila.get(0),
            )
            .optional()?;
        if existe_proveedor.is_none() {
            return Err(AppError::NotFound(format!(
                "Proveedor {proveedor} en la fila {}",
                numero + 2
            )));
        }

        if catalogo::buscar_producto(&tx, id_producto)?.is_none() {
            let nombre = campo_csv(&cabeceras, fila, "nombreProducto");
            let marca = campo_csv(&cabeceras, fila, "marca");
            let categoria = campo_csv(&cabeceras, fila, "categoria");
            if !validacion::texto_valido(nombre)
                || !validacion::texto_valido(marca)
                || !validacion::texto_valido(categoria)
            {
                return Err(AppError::Validation(format!(
                    "Datos de producto inválidos en la fila {}",
                    numero + 2
                )));
            }
            tx.execute(
                "INSERT INTO productos(id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock) VALUES(?1, ?2, ?3, ?4, 'Sin descripcion', ?5, ?6, 0)",
                params![id_producto, nombre, marca, categoria, precio, precio * 1.3],
            )?;
        } else {
            tx.execute(
                "UPDATE productos SET precio_entrada = ?2, precio_venta = ?3 WHERE lower(id_producto) = lower(?1)",
                params![id_producto, precio, precio * 1.3],
            )?;
        }
        registrar_entrada_en(&tx, id_producto, cantidad, precio, Some(proveedor), ci)?;
        importadas += 1;
    }
    if importadas == 0 {
        return Err(AppError::Validation(
            "El archivo no contiene entradas".into(),
        ));
    }
    auditar(
        &tx,
        ci,
        "IMPORTAR_ENTRADAS",
        &format!("{importadas} entradas"),
    )?;
    tx.commit()?;
    Ok(importadas)
}
