use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};
use std::path::PathBuf;
use stockbox_core::db::Db;
use stockbox_core::models::Estadisticas;
use stockbox_core::servicios::admin;

struct StockBox {
    titulo: SharedString,
    datos: Estadisticas,
}

impl Render for StockBox {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .bg(rgb(0x0f172a))
            .size_full()
            .justify_center()
            .items_center()
            .text_color(rgb(0xe6edf7))
            .child(
                div()
                    .text_3xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(self.titulo.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .child("GPUI · Rust + SQLite · núcleo compartido"),
            )
            .child(
                div()
                    .mt_4()
                    .flex()
                    .gap_8()
                    .child(format!("Productos: {}", self.datos.productos))
                    .child(format!("Stock crítico: {}", self.datos.stock_critico))
                    .child(format!("Ventas hoy: {}", self.datos.ventas_hoy)),
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
                    titulo: "StockBox".into(),
                    datos,
                })
            },
        )
        .unwrap();
    });
}
