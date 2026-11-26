use crate::error::Result;
use crate::models::{DashboardDatos, DashboardResumen, Grupo, PuntoDia};
use chrono::{Duration, Local, NaiveDate};
use rusqlite::params;
use rusqlite::Connection;
use std::collections::HashMap;

pub fn datos(conn: &Connection, periodo: &str) -> Result<DashboardDatos> {
    let hoy = Local::now().date_naive();
    let desde = match periodo {
        "HOY" => hoy,
        "ULTIMOS_7" => hoy - Duration::days(6),
        "ULTIMOS_30" => hoy - Duration::days(29),
        _ => NaiveDate::from_ymd_opt(1, 1, 1).unwrap_or(hoy),
    };
    let desde_texto = desde.to_string();
    let hasta_texto = hoy.to_string();

    let (total, cantidad_ventas, unidades): (f64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(v.total), 0), COUNT(*),
                COALESCE((SELECT SUM(vi.cantidad) FROM venta_items vi WHERE vi.id_venta = v.id_venta), 0)
         FROM ventas v
         WHERE v.estado = 'CONFIRMADA' AND substr(v.fecha_venta, 1, 10) BETWEEN ?1 AND ?2",
        params![desde_texto, hasta_texto],
        |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?)),
    )?;
    let (productos, stock_critico, valor_inventario): (i64, i64, f64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN stock <= 5 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(stock * precio_venta), 0) FROM productos",
        [],
        |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?)),
    )?;

    let resumen = DashboardResumen {
        total,
        cantidad_ventas,
        unidades,
        ticket_promedio: if cantidad_ventas > 0 {
            total / cantidad_ventas as f64
        } else {
            0.0
        },
        productos,
        stock_critico,
        valor_inventario,
    };

    let mut sentencia = conn.prepare(
        "SELECT substr(fecha_venta, 1, 10) AS dia, SUM(total) FROM ventas
         WHERE estado = 'CONFIRMADA' AND substr(fecha_venta, 1, 10) BETWEEN ?1 AND ?2
         GROUP BY dia ORDER BY dia",
    )?;
    let filas = sentencia.query_map(params![desde_texto, hasta_texto], |fila| {
        Ok(PuntoDia {
            fecha: fila.get(0)?,
            total: fila.get(1)?,
        })
    })?;
    let mut ventas_por_dia: Vec<PuntoDia> = Vec::new();
    for fila in filas {
        ventas_por_dia.push(fila?);
    }
    let dias: HashMap<String, f64> = ventas_por_dia
        .iter()
        .map(|punto| (punto.fecha.clone(), punto.total))
        .collect();
    let cantidad_dias = (hoy - desde).num_days() + 1;
    if (1..=31).contains(&cantidad_dias) {
        ventas_por_dia.clear();
        for indice in 0..cantidad_dias {
            let dia = (desde + Duration::days(indice)).to_string();
            ventas_por_dia.push(PuntoDia {
                fecha: dia.clone(),
                total: *dias.get(&dia).unwrap_or(&0.0),
            });
        }
    }

    let mut sentencia = conn.prepare(
        "SELECT COALESCE(NULLIF(p.categoria, ''), 'Sin categoría') AS categoria, SUM(vi.subtotal)
         FROM ventas v JOIN venta_items vi ON vi.id_venta = v.id_venta
         LEFT JOIN productos p ON lower(p.id_producto) = lower(vi.id_producto)
         WHERE v.estado = 'CONFIRMADA' AND substr(v.fecha_venta, 1, 10) BETWEEN ?1 AND ?2
         GROUP BY categoria ORDER BY SUM(vi.subtotal) DESC",
    )?;
    let filas = sentencia.query_map(params![desde_texto, hasta_texto], |fila| {
        Ok(Grupo {
            etiqueta: fila.get(0)?,
            valor: fila.get(1)?,
        })
    })?;
    let mut ventas_por_categoria = Vec::new();
    for fila in filas {
        ventas_por_categoria.push(fila?);
    }

    let mut sentencia = conn.prepare(
        "SELECT vi.nombre_producto, SUM(vi.cantidad) FROM ventas v
         JOIN venta_items vi ON vi.id_venta = v.id_venta
         WHERE v.estado = 'CONFIRMADA' AND substr(v.fecha_venta, 1, 10) BETWEEN ?1 AND ?2
         GROUP BY vi.id_producto ORDER BY SUM(vi.cantidad) DESC LIMIT 5",
    )?;
    let filas = sentencia.query_map(params![desde_texto, hasta_texto], |fila| {
        Ok(Grupo {
            etiqueta: fila.get(0)?,
            valor: fila.get(1)?,
        })
    })?;
    let mut top_productos = Vec::new();
    for fila in filas {
        top_productos.push(fila?);
    }

    let mut sentencia = conn.prepare(
        "SELECT COALESCE(NULLIF(categoria, ''), 'Sin categoría'), SUM(stock) FROM productos
         GROUP BY categoria ORDER BY SUM(stock) DESC",
    )?;
    let filas = sentencia.query_map([], |fila| {
        Ok(Grupo {
            etiqueta: fila.get(0)?,
            valor: fila.get(1)?,
        })
    })?;
    let mut stock_por_categoria = Vec::new();
    for fila in filas {
        stock_por_categoria.push(fila?);
    }

    Ok(DashboardDatos {
        desde: desde_texto,
        hasta: hasta_texto,
        resumen,
        ventas_por_dia,
        ventas_por_categoria,
        top_productos,
        stock_por_categoria,
    })
}
