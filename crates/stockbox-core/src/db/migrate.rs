use crate::error::Result;
use crate::security::{es_hash, hash_password};
use csv::{ReaderBuilder, StringRecord, Trim};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub empleados: usize,
    pub proveedores: usize,
    pub clientes: usize,
    pub productos: usize,
    pub entradas: usize,
    pub caja: usize,
    pub ordenes: usize,
    pub ventas: usize,
    pub garantias: usize,
    pub pedidos: usize,
    pub filas_omitidas: usize,
}

impl Default for MigrationReport {
    fn default() -> Self {
        Self {
            empleados: 0,
            proveedores: 0,
            clientes: 0,
            productos: 0,
            entradas: 0,
            caja: 0,
            ordenes: 0,
            ventas: 0,
            garantias: 0,
            pedidos: 0,
            filas_omitidas: 0,
        }
    }
}

pub struct CsvBundle {
    pub empleados: String,
    pub proveedores: String,
    pub clientes: String,
    pub inventario: String,
    pub entradas: String,
    pub caja: String,
    pub ventas: String,
    pub garantias: String,
    pub pedidos: String,
}

impl CsvBundle {
    pub fn incrustado() -> Self {
        Self {
            empleados: include_str!("../../../../data/empleado.csv").to_string(),
            proveedores: include_str!("../../../../data/proveedores.csv").to_string(),
            clientes: include_str!("../../../../data/clientes.csv").to_string(),
            inventario: include_str!("../../../../data/inventario.csv").to_string(),
            entradas: include_str!("../../../../data/entradas.csv").to_string(),
            caja: include_str!("../../../../data/caja.csv").to_string(),
            ventas: include_str!("../../../../data/ventas.csv").to_string(),
            garantias: include_str!("../../../../data/garantias.csv").to_string(),
            pedidos: include_str!("../../../../data/pedidos.csv").to_string(),
        }
    }

    pub fn desde_directorio(directorio: &Path) -> Self {
        let leer =
            |nombre: &str| std::fs::read_to_string(directorio.join(nombre)).unwrap_or_default();
        Self {
            empleados: leer("empleado.csv"),
            proveedores: leer("proveedores.csv"),
            clientes: leer("clientes.csv"),
            inventario: leer("inventario.csv"),
            entradas: leer("entradas.csv"),
            caja: leer("caja.csv"),
            ventas: leer("ventas.csv"),
            garantias: leer("garantias.csv"),
            pedidos: leer("pedidos.csv"),
        }
    }
}

#[derive(Clone)]
struct FilaVenta {
    id_venta: String,
    id_orden: String,
    estado: String,
    id_cliente: String,
    nombre_cliente: String,
    ci_cliente: String,
    telefono_cliente: String,
    id_garantia: String,
    id_producto: String,
    nombre_producto: String,
    cantidad: i64,
    precio_unitario: f64,
    total: f64,
    fecha: String,
    ci_empleado: String,
}

pub fn importar(conn: &mut Connection, bundle: &CsvBundle) -> Result<MigrationReport> {
    let tx = conn.transaction()?;
    let mut informe = MigrationReport::default();

    let (cabeceras, filas) = leer_csv(&bundle.empleados)?;
    for fila in &filas {
        let ci = campo(&cabeceras, fila, "ci");
        if ci.is_empty() {
            informe.filas_omitidas += 1;
            continue;
        }
        let contrasena = campo(&cabeceras, fila, "contrasena");
        let hash = if es_hash(contrasena) {
            contrasena.to_string()
        } else {
            hash_password(contrasena)?
        };
        informe.empleados += tx.execute(
            "INSERT OR IGNORE INTO empleados(ci, nombre, rol, contrasena_hash) VALUES(?1, ?2, ?3, ?4)",
            params![ci, campo(&cabeceras, fila, "nombre"), rol(campo(&cabeceras, fila, "rol")), hash],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.proveedores)?;
    for fila in &filas {
        let id = campo(&cabeceras, fila, "idProveedor");
        if id.is_empty() {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.proveedores += tx.execute(
            "INSERT OR IGNORE INTO proveedores(id_proveedor, nombre, contacto) VALUES(?1, ?2, ?3)",
            params![
                id,
                campo(&cabeceras, fila, "nombreProveedor"),
                campo(&cabeceras, fila, "contactoProveedor")
            ],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.clientes)?;
    for fila in &filas {
        let id = campo(&cabeceras, fila, "idCliente");
        if id.is_empty() {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.clientes += tx.execute(
            "INSERT OR IGNORE INTO clientes(id_cliente, nombre_cliente, ci, telefono) VALUES(?1, ?2, ?3, ?4)",
            params![id, campo(&cabeceras, fila, "nombreCliente"), campo(&cabeceras, fila, "ci"), campo(&cabeceras, fila, "telefono")],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.inventario)?;
    for fila in &filas {
        let id = campo(&cabeceras, fila, "idProducto");
        let (Some(precio_entrada), Some(precio_venta), Some(stock)) = (
            numero(&cabeceras, fila, "precioEntrada"),
            numero(&cabeceras, fila, "precioVenta"),
            entero(&cabeceras, fila, "stock"),
        ) else {
            informe.filas_omitidas += 1;
            continue;
        };
        if id.is_empty() || precio_entrada <= 0.0 || precio_venta <= 0.0 || stock < 0 {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.productos += tx.execute(
            "INSERT OR IGNORE INTO productos(id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, campo(&cabeceras, fila, "nombre"), campo(&cabeceras, fila, "marca"), campo(&cabeceras, fila, "categoria"), campo(&cabeceras, fila, "descripcion"), precio_entrada, precio_venta, stock],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.entradas)?;
    for fila in &filas {
        let (Some(cantidad), Some(precio)) = (
            entero(&cabeceras, fila, "cantidad"),
            numero(&cabeceras, fila, "precioUnitario"),
        ) else {
            informe.filas_omitidas += 1;
            continue;
        };
        let id = campo(&cabeceras, fila, "idEntrada");
        if id.is_empty() || cantidad <= 0 || precio <= 0.0 {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.entradas += tx.execute(
            "INSERT OR IGNORE INTO entradas(id_entrada, id_proveedor, id_producto, nombre_producto, cantidad, precio_unitario, fecha_entrada, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, campo(&cabeceras, fila, "idProveedor"), campo(&cabeceras, fila, "idProducto"), campo(&cabeceras, fila, "nombreProducto"), cantidad, precio, campo(&cabeceras, fila, "fechaEntrada"), campo(&cabeceras, fila, "ciEmpleado")],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.caja)?;
    if let Some(fila) = filas.first() {
        informe.caja = tx.execute(
            "INSERT OR REPLACE INTO caja(id, estado, fecha_apertura, fecha_cierre, ci_empleado) VALUES(1, ?1, ?2, ?3, ?4)",
            params![campo(&cabeceras, fila, "estado"), campo(&cabeceras, fila, "fechaApertura"), campo(&cabeceras, fila, "fechaCierre"), campo(&cabeceras, fila, "ciEmpleado")],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.ventas)?;
    let mut orden_ventas: Vec<String> = Vec::new();
    let mut agrupadas: HashMap<String, (FilaVenta, Vec<FilaVenta>)> = HashMap::new();
    for fila in &filas {
        let Some(f) = fila_venta(&cabeceras, fila) else {
            informe.filas_omitidas += 1;
            continue;
        };
        if f.id_venta.is_empty() {
            informe.filas_omitidas += 1;
            continue;
        }
        if !agrupadas.contains_key(&f.id_venta) {
            orden_ventas.push(f.id_venta.clone());
            agrupadas.insert(f.id_venta.clone(), (f.clone(), Vec::new()));
        }
        if let Some((_, items)) = agrupadas.get_mut(&f.id_venta) {
            items.push(f);
        }
    }
    for id_venta in &orden_ventas {
        let Some((cabecera, items)) = agrupadas.get(id_venta) else {
            continue;
        };
        informe.ordenes += tx.execute(
            "INSERT OR IGNORE INTO ordenes(id_orden, estado, id_cliente, nombre_cliente, ci_cliente, telefono_cliente, fecha, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![cabecera.id_orden, cabecera.estado, cabecera.id_cliente, cabecera.nombre_cliente, cabecera.ci_cliente, cabecera.telefono_cliente, cabecera.fecha, cabecera.ci_empleado],
        )?;
        let total: f64 = items
            .iter()
            .map(|item| {
                if item.total > 0.0 {
                    item.total
                } else {
                    item.cantidad as f64 * item.precio_unitario
                }
            })
            .sum();
        informe.ventas += tx.execute(
            "INSERT OR IGNORE INTO ventas(id_venta, id_orden, estado, id_cliente, nombre_cliente, ci_cliente, telefono_cliente, id_garantia, total, fecha_venta, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![cabecera.id_venta, cabecera.id_orden, cabecera.estado, cabecera.id_cliente, cabecera.nombre_cliente, cabecera.ci_cliente, cabecera.telefono_cliente, cabecera.id_garantia, total, cabecera.fecha, cabecera.ci_empleado],
        )?;
        for item in items {
            tx.execute(
                "INSERT INTO venta_items(id_venta, id_producto, nombre_producto, cantidad, precio_unitario, subtotal) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                params![cabecera.id_venta, item.id_producto, item.nombre_producto, item.cantidad, item.precio_unitario, item.total],
            )?;
        }
    }

    let (cabeceras, filas) = leer_csv(&bundle.garantias)?;
    for fila in &filas {
        let id = campo(&cabeceras, fila, "idGarantia");
        if id.is_empty() {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.garantias += tx.execute(
            "INSERT OR IGNORE INTO garantias(id_garantia, id_venta, id_orden, id_cliente, ci_cliente, nombre_cliente, telefono_cliente, id_producto, nombre_producto, fecha_inicio, fecha_fin, estado, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![id, campo(&cabeceras, fila, "idVenta"), campo(&cabeceras, fila, "idOrden"), campo(&cabeceras, fila, "idCliente"), campo(&cabeceras, fila, "ciCliente"), campo(&cabeceras, fila, "nombreCliente"), campo(&cabeceras, fila, "telefonoCliente"), campo(&cabeceras, fila, "idProducto"), campo(&cabeceras, fila, "nombreProducto"), campo(&cabeceras, fila, "fechaInicio"), campo(&cabeceras, fila, "fechaFin"), campo(&cabeceras, fila, "estado"), campo(&cabeceras, fila, "ciEmpleado")],
        )?;
    }

    let (cabeceras, filas) = leer_csv(&bundle.pedidos)?;
    for fila in &filas {
        let Some(cantidad) = entero(&cabeceras, fila, "cantidad") else {
            informe.filas_omitidas += 1;
            continue;
        };
        let id_pedido = campo(&cabeceras, fila, "idPedido");
        let id_producto = campo(&cabeceras, fila, "idProducto");
        if id_pedido.is_empty() || id_producto.is_empty() || cantidad <= 0 {
            informe.filas_omitidas += 1;
            continue;
        }
        informe.pedidos += tx.execute(
            "INSERT OR IGNORE INTO pedidos(id_pedido, id_proveedor, id_producto, cantidad, fecha_pedido, ci_empleado) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![id_pedido, campo(&cabeceras, fila, "idProveedor"), id_producto, cantidad, campo(&cabeceras, fila, "fechaPedido"), campo(&cabeceras, fila, "ciEmpleado")],
        )?;
    }

    tx.commit()?;
    Ok(informe)
}

fn leer_csv(contenido: &str) -> Result<(Vec<String>, Vec<StringRecord>)> {
    if contenido.trim().is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut lector = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(Cursor::new(contenido));
    let cabeceras = lector.headers()?.iter().map(str::to_string).collect();
    let mut filas = Vec::new();
    for fila in lector.records() {
        filas.push(fila?);
    }
    Ok((cabeceras, filas))
}

fn campo<'a>(cabeceras: &[String], fila: &'a StringRecord, nombre: &str) -> &'a str {
    cabeceras
        .iter()
        .position(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
        .and_then(|indice| fila.get(indice))
        .unwrap_or("")
}

fn numero(cabeceras: &[String], fila: &StringRecord, nombre: &str) -> Option<f64> {
    campo(cabeceras, fila, nombre)
        .parse::<f64>()
        .ok()
        .filter(|valor| valor.is_finite())
}

fn entero(cabeceras: &[String], fila: &StringRecord, nombre: &str) -> Option<i64> {
    campo(cabeceras, fila, nombre).parse::<i64>().ok()
}

fn rol(valor: &str) -> &'static str {
    match valor.to_ascii_uppercase().as_str() {
        "ADMIN" | "ADMINISTRADOR" | "GESTOR" => "ADMINISTRADOR",
        "ALMACEN" | "ALMACENERO" => "ALMACENERO",
        _ => "CAJERO",
    }
}

fn fila_venta(cabeceras: &[String], fila: &StringRecord) -> Option<FilaVenta> {
    let cantidad = entero(cabeceras, fila, "cantidad")?;
    let precio_unitario = numero(cabeceras, fila, "precioUnitario")?;
    let total = numero(cabeceras, fila, "total").unwrap_or(0.0);
    let fecha = campo(cabeceras, fila, "fechaVenta");
    Some(FilaVenta {
        id_venta: campo(cabeceras, fila, "idVenta").to_string(),
        id_orden: campo(cabeceras, fila, "idOrden").to_string(),
        estado: campo(cabeceras, fila, "estado").to_string(),
        id_cliente: campo(cabeceras, fila, "idCliente").to_string(),
        nombre_cliente: campo(cabeceras, fila, "nombreCliente").to_string(),
        ci_cliente: campo(cabeceras, fila, "ciCliente").to_string(),
        telefono_cliente: campo(cabeceras, fila, "telefonoCliente").to_string(),
        id_garantia: campo(cabeceras, fila, "idGarantia").to_string(),
        id_producto: campo(cabeceras, fila, "idProducto").to_string(),
        nombre_producto: campo(cabeceras, fila, "nombreProducto").to_string(),
        cantidad,
        precio_unitario,
        total,
        fecha: fecha.to_string(),
        ci_empleado: campo(cabeceras, fila, "ciEmpleado").to_string(),
    })
}
