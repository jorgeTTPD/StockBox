use crate::error::{AppError, Result};
use crate::models::ConfigAsistente;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

const URL_OPENROUTER: &str = "https://openrouter.ai/api/v1/chat/completions";
const URL_NVIDIA: &str = "https://integrate.api.nvidia.com/v1/chat/completions";
const MODELO_OPENROUTER: &str = "meta-llama/llama-3.1-8b-instruct:free";
const MODELO_NVIDIA: &str = "nvidia/nemotron-3.5-lightning-30b-a3b";
const MAX_HISTORIAL: usize = 12;

#[derive(Clone)]
struct Config {
    provider: String,
    base_url: String,
    api_key: String,
    modelo: String,
}

impl Config {
    fn configurada(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}

#[derive(Clone)]
struct Mensaje {
    rol: String,
    contenido: String,
}

pub struct Asistente {
    historial: Mutex<Vec<Mensaje>>,
}

impl Asistente {
    pub fn new() -> Self {
        Self {
            historial: Mutex::new(Vec::new()),
        }
    }

    pub fn config_para_ui(&self, dir: &Path) -> ConfigAsistente {
        let config = self.configuracion(dir);
        let configurada = config.configurada();
        ConfigAsistente {
            provider: config.provider,
            modelo: config.modelo,
            configurada,
        }
    }

    pub fn instrucciones(&self, dir: &Path) -> String {
        let ruta = dir.join("ia.properties");
        format!(
            "El Asistente IA no está configurado.\nCrea el archivo {} con:\n\nprovider=openrouter\napiKey=TU_API_KEY\n\nO exporta OPENROUTER_API_KEY (o NVIDIA_API_KEY con provider=nvidia) y reinicia la aplicación.\nObtén una key en openrouter.ai/keys o build.nvidia.com.",
            ruta.display()
        )
    }

    pub fn preguntar(&self, dir: &Path, snapshot: &str, pregunta: &str) -> Result<String> {
        let config = self.configuracion(dir);
        let pregunta = pregunta.trim();
        if pregunta.is_empty() {
            return Err(AppError::Validation(
                "La pregunta no puede estar vacía".into(),
            ));
        }
        if !config.configurada() {
            return Err(AppError::Validation(self.instrucciones(dir)));
        }

        let mut mensajes = vec![Mensaje {
            rol: "system".into(),
            contenido: format!(
                "Eres el Asistente IA de StockBox, una tienda de tecnología. Ayudas al vendedor a recomendar y comparar productos.\n\
                 REGLAS:\n- Responde ÚNICAMENTE con productos del inventario listado abajo. Nunca inventes productos, marcas ni precios.\n\
                 - Si te piden recomendación, haz preguntas breves si hace falta y propón de 1 a 3 opciones reales.\n\
                 - Si te piden comparación, compara precio de venta, marca, categoría, descripción y stock; termina con un veredicto claro.\n\
                 - Si un producto no existe o no hay stock, dilo con franqueza y ofrece una alternativa disponible.\n\
                 - Cita siempre precio de venta y stock disponible.\n- No reveles ni menciones precios de costo ni márgenes.\n\
                 - Responde en español, breve y directo (máximo unas 120 palabras salvo que pidan detalle).\n\n\
                 INVENTARIO ACTUAL (id | nombre | marca | categoría | descripción | precio venta | stock):\n{snapshot}"
            ),
        }];
        {
            let historial = self
                .historial
                .lock()
                .map_err(|_| AppError::Validation("Historial bloqueado".into()))?;
            mensajes.extend(historial.iter().cloned());
        }
        mensajes.push(Mensaje {
            rol: "user".into(),
            contenido: pregunta.to_string(),
        });

        let valores: Vec<Value> = mensajes
            .iter()
            .map(|mensaje| json!({ "role": mensaje.rol, "content": mensaje.contenido }))
            .collect();
        let mut cuerpo = json!({
            "model": config.modelo,
            "temperature": 0.4,
            "max_tokens": 4096,
            "stream": false,
            "messages": valores,
        });
        if config.provider == "nvidia" {
            cuerpo["chat_template_kwargs"] = json!({ "enable_thinking": false });
        }

        let cliente = Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .map_err(|error| {
                AppError::Validation(format!("No se pudo crear el cliente HTTP: {error}"))
            })?;
        let mut peticion = cliente
            .post(&config.base_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.api_key));
        if config.provider == "openrouter" {
            peticion = peticion
                .header("HTTP-Referer", "https://github.com/jorgeTTPD/StockBox")
                .header("X-Title", "StockBox");
        }
        let respuesta = peticion.json(&cuerpo).send().map_err(|error| {
            AppError::Validation(format!(
                "No se pudo conectar con el servicio de IA: {error}"
            ))
        })?;
        let estado = respuesta.status().as_u16();
        let texto = respuesta
            .text()
            .map_err(|error| AppError::Validation(error.to_string()))?;

        let json: Value = serde_json::from_str(&texto).map_err(|_| {
            AppError::Validation(match estado {
                401 | 403 => "La API key fue rechazada. Revisa tu configuración.".into(),
                429 => {
                    "Límite de peticiones alcanzado. Espera un momento e intenta de nuevo.".into()
                }
                _ if !(200..300).contains(&estado) => {
                    format!("El servicio respondió con HTTP {estado}.")
                }
                _ => format!("El servicio no devolvió JSON válido (HTTP {estado})."),
            })
        })?;

        let contenido = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();
        if contenido.is_empty() {
            return Err(AppError::Validation(
                "El servicio devolvió una respuesta sin texto.".into(),
            ));
        }

        let mut historial = self
            .historial
            .lock()
            .map_err(|_| AppError::Validation("Historial bloqueado".into()))?;
        historial.push(Mensaje {
            rol: "user".into(),
            contenido: pregunta.to_string(),
        });
        historial.push(Mensaje {
            rol: "assistant".into(),
            contenido: contenido.clone(),
        });
        while historial.len() > MAX_HISTORIAL {
            historial.remove(0);
        }
        Ok(contenido)
    }

    pub fn limpiar(&self) -> Result<()> {
        let mut historial = self
            .historial
            .lock()
            .map_err(|_| AppError::Validation("Historial bloqueado".into()))?;
        historial.clear();
        Ok(())
    }

    pub fn tiene_historial(&self) -> bool {
        self.historial
            .lock()
            .map(|historial| !historial.is_empty())
            .unwrap_or(false)
    }

    fn configuracion(&self, dir: &Path) -> Config {
        let propiedades = cargar_propiedades(dir);
        let provider_env = if std::env::var("OPENROUTER_API_KEY").is_ok() {
            "openrouter"
        } else if std::env::var("NVIDIA_API_KEY").is_ok() {
            "nvidia"
        } else {
            ""
        };
        let provider = propiedades
            .get("provider")
            .map(String::as_str)
            .unwrap_or(provider_env);
        let provider = if provider.is_empty() {
            "openrouter"
        } else {
            provider
        };
        let nvidia = provider.eq_ignore_ascii_case("nvidia");
        let variable = if nvidia {
            "NVIDIA_API_KEY"
        } else {
            "OPENROUTER_API_KEY"
        };
        let api_key = std::env::var(variable)
            .unwrap_or_default()
            .trim()
            .to_string();
        let api_key = if api_key.is_empty() {
            propiedades.get("apiKey").cloned().unwrap_or_default()
        } else {
            api_key
        };
        let base_url = propiedades
            .get("baseUrl")
            .cloned()
            .unwrap_or_else(|| (if nvidia { URL_NVIDIA } else { URL_OPENROUTER }).to_string());
        let modelo = propiedades.get("model").cloned().unwrap_or_else(|| {
            (if nvidia {
                MODELO_NVIDIA
            } else {
                MODELO_OPENROUTER
            })
            .to_string()
        });
        Config {
            provider: if nvidia { "nvidia" } else { "openrouter" }.to_string(),
            base_url,
            api_key,
            modelo,
        }
    }
}

impl Default for Asistente {
    fn default() -> Self {
        Self::new()
    }
}

fn cargar_propiedades(dir: &Path) -> HashMap<String, String> {
    let mut resultado = HashMap::new();
    let contenido = match fs::read_to_string(dir.join("ia.properties")) {
        Ok(contenido) => contenido,
        Err(_) => return resultado,
    };
    for linea in contenido.lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }
        if let Some((clave, valor)) = linea.split_once('=') {
            resultado.insert(clave.trim().to_string(), valor.trim().to_string());
        }
    }
    resultado
}

pub fn snapshot_inventario(conn: &rusqlite::Connection) -> Result<String> {
    let mut sentencia = conn.prepare(
        "SELECT id_producto, nombre, marca, categoria, descripcion, precio_venta, stock FROM productos ORDER BY nombre",
    )?;
    let filas = sentencia.query_map([], |fila| {
        let descripcion: String = fila.get(4)?;
        let descripcion = descripcion.replace('\n', " ").replace('\r', " ");
        let recortada: String = descripcion.chars().take(140).collect();
        Ok(format!(
            "{} | {} | Marca: {} | Cat: {} | Desc: {} | ${} | Stock: {}",
            fila.get::<_, String>(0)?,
            fila.get::<_, String>(1)?,
            fila.get::<_, String>(2)?,
            fila.get::<_, String>(3)?,
            recortada,
            fila.get::<_, f64>(5)?,
            fila.get::<_, i64>(6)?,
        ))
    })?;
    let mut lineas = Vec::new();
    for fila in filas {
        lineas.push(fila?);
    }
    Ok(lineas.join("\n"))
}
