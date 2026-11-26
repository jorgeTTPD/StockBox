use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};
use std::path::PathBuf;
use std::sync::Mutex;
use stockbox_core::db::Db;
use stockbox_core::models::{Estadisticas, Sesion};
use stockbox_core::servicios::{admin, auth};

const DEMO: [(&str, &str, &str, &str); 3] = [
    ("ADMINISTRADOR", "ADM-001", "user", "Administrador"),
    ("CAJERO", "DEMO-VEN-001", "ventas123", "Cajero"),
    ("ALMACENERO", "DEMO-ALM-001", "almacen123", "Almacenero"),
];

struct StockBox {
    db: Mutex<Db>,
    sesion: Option<Sesion>,
    rol: usize,
    mensaje: String,
    datos: Estadisticas,
}

impl Render for StockBox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fondo = rgb(0x0f172a);
        let texto = rgb(0xe6edf7);
        let suave = rgb(0x93a4c0);
        let acento = rgb(0x38bdf8);

        if self.sesion.is_none() {
            return div()
                .flex()
                .flex_col()
                .gap_4()
                .bg(fondo)
                .size_full()
                .justify_center()
                .items_center()
                .text_color(texto)
                .child(
                    div()
                        .text_3xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("StockBox"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(suave)
                        .child("GPUI · Rust + SQLite"),
                )
                .child(
                    div()
                        .mt_3()
                        .flex()
                        .gap_2()
                        .children(DEMO.iter().enumerate().map(|(indice, opcion)| {
                            let seleccionado = indice == self.rol;
                            div()
                                .id(("rol", indice))
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(if seleccionado { acento } else { rgb(0x172238) })
                                .text_color(if seleccionado { rgb(0x071426) } else { texto })
                                .child(SharedString::from(opcion.3.to_string()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.rol = indice;
                                    this.mensaje.clear();
                                    cx.notify();
                                }))
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(suave)
                        .child(format!("CI: {}", DEMO[self.rol].1)),
                )
                .child(
                    div()
                        .id("entrar")
                        .mt_2()
                        .px_6()
                        .py_2()
                        .rounded_md()
                        .bg(acento)
                        .text_color(rgb(0x071426))
                        .child("Entrar")
                        .on_click(cx.listener(|this, _, _, cx| {
                            let (rol, ci, clave, _) = DEMO[this.rol];
                            match auth::verificar(&this.db.lock().unwrap().conn, ci, clave, rol) {
                                Ok(sesion) => {
                                    if let Ok(datos) =
                                        admin::estadisticas(&this.db.lock().unwrap().conn)
                                    {
                                        this.datos = datos;
                                    }
                                    this.sesion = Some(sesion);
                                    this.mensaje.clear();
                                }
                                Err(error) => this.mensaje = error.to_string(),
                            }
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xf87171))
                        .child(self.mensaje.clone()),
                );
        }

        let sesion = self.sesion.as_ref().unwrap();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .bg(fondo)
            .size_full()
            .p_8()
            .text_color(texto)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("StockBox"),
                    )
                    .child(
                        div()
                            .id("salir")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x172238))
                            .child("Cerrar sesión")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sesion = None;
                                cx.notify();
                            })),
                    ),
            )
            .child(div().text_sm().text_color(suave).child(format!(
                "{} · {} · {}",
                sesion.nombre, sesion.ci, sesion.rol
            )))
            .child(
                div()
                    .mt_6()
                    .flex()
                    .gap_8()
                    .child(format!("Productos: {}", self.datos.productos))
                    .child(format!("Stock crítico: {}", self.datos.stock_critico))
                    .child(format!("Ventas hoy: {}", self.datos.ventas_hoy))
                    .child(format!("Entradas hoy: {}", self.datos.entradas_hoy)),
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(suave)
                    .child("Panel de ventas, almacén y administración en construcción."),
            )
    }
}

fn main() {
    let directorio = std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".local/share/stockbox-gpui"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/stockbox-gpui"));
    let mut db = Db::open(&directorio.join("stockbox.sqlite")).expect("No se pudo abrir SQLite");
    db.seed_if_empty()
        .expect("No se pudo sembrar la base de datos");
    let datos = admin::estadisticas(&db.conn).expect("No se pudo calcular el resumen");

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(720.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| StockBox {
                    db: Mutex::new(db),
                    sesion: None,
                    rol: 0,
                    mensaje: String::new(),
                    datos,
                })
            },
        )
        .unwrap();
    });
}
