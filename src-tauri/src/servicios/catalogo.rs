use crate::error::{AppError, Result};
use crate::models::{Cliente, ClienteInput, Producto, ProductoInput, Proveedor, ProveedorInput};
use crate::security::generar_id;
use crate::validacion;
use rusqlite::{params, Connection, OptionalExtension, Row};

pub fn map_producto(fila: &Row<'_>) -> rusqlite::Result<Producto> {
    Ok(Producto {
        id_producto: fila.get(0)?,
        nombre: fila.get(1)?,
        marca: fila.get(2)?,
        categoria: fila.get(3)?,
        descripcion: fila.get(4)?,
        precio_entrada: fila.get(5)?,
        precio_venta: fila.get(6)?,
        stock: fila.get(7)?,
    })
}

const COLUMNAS_PRODUCTO: &str =
    "id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock";

pub fn listar_productos(conn: &Connection, termino: Option<&str>) -> Result<Vec<Producto>> {
    let termino = termino.unwrap_or("").trim();
    let patron = format!("%{}%", termino.to_lowercase());
    let mut sentencia = conn.prepare(&format!(
        "SELECT {COLUMNAS_PRODUCTO} FROM productos \
         WHERE ?1 = '' OR lower(id_producto) LIKE ?2 OR lower(nombre) LIKE ?2 \
         OR lower(marca) LIKE ?2 OR lower(categoria) LIKE ?2 ORDER BY nombre"
    ))?;
    let filas = sentencia.query_map(params![termino, patron], map_producto)?;
    let mut productos = Vec::new();
    for fila in filas {
        productos.push(fila?);
    }
    Ok(productos)
}

pub fn buscar_producto(conn: &Connection, id: &str) -> Result<Option<Producto>> {
    let mut sentencia = conn.prepare(&format!(
        "SELECT {COLUMNAS_PRODUCTO} FROM productos WHERE lower(id_producto) = lower(?1)"
    ))?;
    sentencia
        .query_row(params![id], map_producto)
        .optional()
        .map_err(Into::into)
}

pub fn categorias(conn: &Connection) -> Result<Vec<String>> {
    let mut sentencia =
        conn.prepare("SELECT DISTINCT categoria FROM productos ORDER BY categoria")?;
    let filas = sentencia.query_map([], |fila| fila.get::<_, String>(0))?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn criticos(conn: &Connection) -> Result<Vec<Producto>> {
    let mut sentencia = conn.prepare(&format!(
        "SELECT {COLUMNAS_PRODUCTO} FROM productos WHERE stock <= 5 ORDER BY stock, nombre"
    ))?;
    let filas = sentencia.query_map([], map_producto)?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn crear_producto(conn: &Connection, entrada: &ProductoInput) -> Result<String> {
    let nombre = entrada.nombre.trim();
    let marca = entrada.marca.trim();
    let categoria = entrada.categoria.trim();
    let descripcion = {
        let texto = entrada.descripcion.trim();
        if texto.is_empty() {
            "Sin descripcion"
        } else {
            texto
        }
    };
    if !validacion::texto_valido(nombre) {
        return Err(AppError::Validation(
            "El nombre del producto no es válido".into(),
        ));
    }
    if !validacion::texto_valido(marca) || !validacion::texto_valido(categoria) {
        return Err(AppError::Validation("Marca o categoría no válidas".into()));
    }
    if !validacion::texto_valido(descripcion) {
        return Err(AppError::Validation("La descripción no es válida".into()));
    }
    if !validacion::fluido(entrada.precio_entrada)
        || !validacion::fluido(entrada.precio_venta)
        || entrada.precio_entrada <= 0.0
        || entrada.precio_venta <= 0.0
        || entrada.precio_venta <= entrada.precio_entrada
    {
        return Err(AppError::Validation(
            "El precio de venta debe ser mayor que el precio de entrada".into(),
        ));
    }
    if entrada.stock < 0 {
        return Err(AppError::Validation(
            "El stock no puede ser negativo".into(),
        ));
    }

    let id = entrada
        .id_producto
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty())
        .unwrap_or("")
        .to_string();
    let id = if id.is_empty() {
        generar_id("PROD")
    } else {
        if !validacion::identificador_valido(&id) {
            return Err(AppError::Validation(
                "El identificador del producto no es válido".into(),
            ));
        }
        id
    };
    if buscar_producto(conn, &id)?.is_some() {
        return Err(AppError::Conflict(format!("Ya existe el producto {id}")));
    }
    conn.execute(
        "INSERT INTO productos(id_producto, nombre, marca, categoria, descripcion, precio_entrada, precio_venta, stock) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, nombre, marca, categoria, descripcion, entrada.precio_entrada, entrada.precio_venta, entrada.stock],
    )?;
    Ok(id)
}

pub fn eliminar_producto(conn: &Connection, id: &str) -> Result<()> {
    let cambiados = conn.execute(
        "DELETE FROM productos WHERE lower(id_producto) = lower(?1)",
        params![id],
    )?;
    if cambiados == 0 {
        return Err(AppError::NotFound(format!("Producto {id}")));
    }
    Ok(())
}

pub fn listar_proveedores(conn: &Connection) -> Result<Vec<Proveedor>> {
    let mut sentencia =
        conn.prepare("SELECT id_proveedor, nombre, contacto FROM proveedores ORDER BY nombre")?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Proveedor {
            id_proveedor: fila.get(0)?,
            nombre: fila.get(1)?,
            contacto: fila.get(2)?,
        })
    })?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn crear_proveedor(conn: &Connection, entrada: &ProveedorInput) -> Result<String> {
    let nombre = entrada.nombre.trim();
    let contacto = entrada.contacto.trim();
    if !validacion::texto_valido(nombre) {
        return Err(AppError::Validation(
            "El nombre del proveedor no es válido".into(),
        ));
    }
    if !contacto.is_empty() && !validacion::telefono_valido(contacto) {
        return Err(AppError::Validation(
            "El contacto del proveedor no es válido".into(),
        ));
    }
    let id = entrada
        .id_proveedor
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| generar_id("PROV"));
    if !validacion::identificador_valido(&id) {
        return Err(AppError::Validation(
            "El identificador del proveedor no es válido".into(),
        ));
    }
    let existe: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM proveedores WHERE lower(id_proveedor) = lower(?1)",
            params![id],
            |fila| fila.get(0),
        )
        .optional()?;
    if existe.is_some() {
        return Err(AppError::Conflict(format!("Ya existe el proveedor {id}")));
    }
    conn.execute(
        "INSERT INTO proveedores(id_proveedor, nombre, contacto) VALUES(?1, ?2, ?3)",
        params![id, nombre, contacto],
    )?;
    Ok(id)
}

pub fn eliminar_proveedor(conn: &Connection, id: &str) -> Result<()> {
    let cambiados = conn.execute(
        "DELETE FROM proveedores WHERE lower(id_proveedor) = lower(?1)",
        params![id],
    )?;
    if cambiados == 0 {
        return Err(AppError::NotFound(format!("Proveedor {id}")));
    }
    Ok(())
}

pub fn listar_clientes(conn: &Connection) -> Result<Vec<Cliente>> {
    let mut sentencia = conn.prepare(
        "SELECT id_cliente, nombre_cliente, ci, telefono FROM clientes ORDER BY nombre_cliente",
    )?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Cliente {
            id_cliente: fila.get(0)?,
            nombre_cliente: fila.get(1)?,
            ci: fila.get(2)?,
            telefono: fila.get(3)?,
        })
    })?;
    let mut resultado = Vec::new();
    for fila in filas {
        resultado.push(fila?);
    }
    Ok(resultado)
}

pub fn guardar_cliente(conn: &Connection, entrada: &ClienteInput) -> Result<String> {
    let nombre = entrada.nombre_cliente.trim();
    let ci = entrada.ci.trim();
    let telefono = entrada.telefono.trim();
    if !validacion::texto_valido(nombre) {
        return Err(AppError::Validation(
            "El nombre del cliente no es válido".into(),
        ));
    }
    if !ci.is_empty() && !validacion::identificador_valido(&ci) {
        return Err(AppError::Validation(
            "La cédula del cliente no es válida".into(),
        ));
    }
    if !telefono.is_empty() && !validacion::telefono_valido(&telefono) {
        return Err(AppError::Validation(
            "El teléfono del cliente no es válido".into(),
        ));
    }

    let encontrado: Option<String> = if !ci.is_empty() {
        conn.query_row(
            "SELECT id_cliente FROM clientes WHERE lower(ci) = lower(?1) LIMIT 1",
            params![ci],
            |fila| fila.get(0),
        )
        .optional()?
    } else {
        conn.query_row(
            "SELECT id_cliente FROM clientes WHERE lower(nombre_cliente) = lower(?1) LIMIT 1",
            params![nombre],
            |fila| fila.get(0),
        )
        .optional()?
    };

    if let Some(id) = encontrado {
        conn.execute(
            "UPDATE clientes SET nombre_cliente = ?2, ci = ?3, telefono = ?4 WHERE id_cliente = ?1",
            params![id, nombre, ci, telefono],
        )?;
        return Ok(id);
    }

    let id = generar_id("CLI");
    conn.execute(
        "INSERT INTO clientes(id_cliente, nombre_cliente, ci, telefono) VALUES(?1, ?2, ?3, ?4)",
        params![id, nombre, ci, telefono],
    )?;
    Ok(id)
}
