use crate::error::{AppError, Result};
use std::fs;
use std::path::{Path, PathBuf};

const EXTENSIONES: [&str; 3] = [".png", ".jpg", ".jpeg"];

pub fn carpeta(dir: &Path) -> PathBuf {
    dir.join("imagenes")
}

pub fn buscar(dir: &Path, id_producto: &str) -> Option<PathBuf> {
    if id_producto.trim().is_empty() {
        return None;
    }
    let base = carpeta(dir);
    EXTENSIONES
        .iter()
        .map(|extension| base.join(format!("{id_producto}{extension}")))
        .find(|candidato| candidato.is_file())
}

pub fn ruta_para_ui(dir: &Path, id_producto: &str) -> Option<String> {
    buscar(dir, id_producto).map(|ruta| ruta.to_string_lossy().into_owned())
}

pub fn guardar(dir: &Path, id_producto: &str, origen: &Path) -> Result<String> {
    let id = id_producto.trim();
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(AppError::Validation(
            "Identificador de producto inválido".into(),
        ));
    }
    let extension = origen
        .extension()
        .and_then(|valor| valor.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !EXTENSIONES
        .iter()
        .any(|valida| valida.trim_start_matches('.') == extension)
    {
        return Err(AppError::Validation(
            "La imagen debe ser PNG, JPG o JPEG".into(),
        ));
    }
    if !origen.is_file() {
        return Err(AppError::NotFound(format!("Archivo {}", origen.display())));
    }
    let destino = carpeta(dir).join(format!("{id}.{extension}"));
    if let Some(padre) = destino.parent() {
        fs::create_dir_all(padre)?;
    }
    fs::copy(origen, &destino)?;
    Ok(destino.to_string_lossy().into_owned())
}
