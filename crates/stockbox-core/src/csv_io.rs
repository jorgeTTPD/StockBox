use crate::error::Result;
use csv::{ReaderBuilder, StringRecord, Trim, WriterBuilder};
use std::io::Cursor;
use std::path::Path;

pub fn leer(ruta: &Path) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let contenido = std::fs::read_to_string(ruta)?;
    leer_contenido(&contenido)
}

pub fn leer_contenido(contenido: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut lector = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(Cursor::new(contenido));
    let cabeceras = lector.headers()?.iter().map(str::to_string).collect();
    let mut filas = Vec::new();
    for registro in lector.records() {
        filas.push(registro?.iter().map(str::to_string).collect());
    }
    Ok((cabeceras, filas))
}

pub fn campo<'a>(cabeceras: &[String], fila: &'a StringRecord, nombre: &str) -> &'a str {
    cabeceras
        .iter()
        .position(|cabecera| cabecera.eq_ignore_ascii_case(nombre))
        .and_then(|indice| fila.get(indice))
        .unwrap_or("")
}

pub fn escribir(ruta: &Path, cabeceras: &[&str], filas: &[Vec<String>]) -> Result<usize> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre)?;
    }
    let mut escritor = WriterBuilder::new().from_path(ruta)?;
    escritor.write_record(cabeceras)?;
    for fila in filas {
        escritor.write_record(fila)?;
    }
    escritor.flush()?;
    Ok(filas.len())
}
