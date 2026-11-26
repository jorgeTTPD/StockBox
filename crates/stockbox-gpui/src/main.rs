use gpui::{
    div, prelude::*, px, rgb, size, AnyElement, App, Bounds, Context, Entity, Window, WindowBounds,
    WindowOptions,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::theme::{Theme, ThemeMode};
use std::path::PathBuf;
use std::sync::Mutex;
use stockbox_core::db::Db;
use stockbox_core::models::{
    Caja, DashboardDatos, Devolucion, EmpleadoInput, Entrada, Estadisticas, Producto,
    ProductoInput, ProveedorInput, Sesion, Venta,
};
use stockbox_core::servicios::{admin, almacen, asistente, auth, catalogo, dashboard, ventas};

const ROLES: [(&str, &str); 3] = [
    ("ADMINISTRADOR", "Administrador"),
    ("CAJERO", "Cajero"),
    ("ALMACENERO", "Almacenero"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Seccion {
    Resumen,
    Inventario,
    Ventas,
    Entradas,
    Productos,
    Empleados,
    Proveedores,
    Devoluciones,
    Garantias,
    Asistente,
    Datos,
}

impl Seccion {
    fn texto(self) -> &'static str {
        match self {
            Self::Resumen => "Resumen",
            Self::Inventario => "Inventario",
            Self::Ventas => "Ventas",
            Self::Entradas => "Entradas",
            Self::Productos => "Productos",
            Self::Empleados => "Empleados",
            Self::Proveedores => "Proveedores",
            Self::Devoluciones => "Devoluciones",
            Self::Garantias => "Garantías",
            Self::Asistente => "Asistente IA",
            Self::Datos => "Datos",
        }
    }
}

struct StockBox {
    db: Mutex<Db>,
    directorio: PathBuf,
    sesion: Option<Sesion>,
    rol: usize,
    seccion: Seccion,
    ci: Entity<InputState>,
    clave: Entity<InputState>,
    campos: Vec<Entity<InputState>>,
    busqueda: Entity<InputState>,
    dev_ref: Entity<InputState>,
    dev_cant: Entity<InputState>,
    ia_pregunta: Entity<InputState>,
    asistente: asistente::Asistente,
    mensaje: String,
    detalle: String,
    ia_chat: Vec<String>,
    datos: Estadisticas,
}

impl StockBox {
    fn valor(&self, campo: &Entity<InputState>, cx: &App) -> String {
        campo.read(cx).value().to_string()
    }

    fn productos(&self) -> Vec<Producto> {
        catalogo::listar_productos(&self.db.lock().unwrap().conn, None).unwrap_or_default()
    }

    fn ventas(&self) -> Vec<Venta> {
        ventas::listar(&self.db.lock().unwrap().conn, None, None).unwrap_or_default()
    }

    fn entradas(&self) -> Vec<Entrada> {
        almacen::listar_entradas(&self.db.lock().unwrap().conn, None).unwrap_or_default()
    }

    fn secciones(&self) -> Vec<Seccion> {
        match self.sesion.as_ref().map(|sesion| sesion.rol.as_str()) {
            Some("ADMINISTRADOR") => vec![
                Seccion::Resumen,
                Seccion::Inventario,
                Seccion::Ventas,
                Seccion::Entradas,
                Seccion::Productos,
                Seccion::Empleados,
                Seccion::Proveedores,
                Seccion::Devoluciones,
                Seccion::Asistente,
                Seccion::Datos,
            ],
            Some("CAJERO") => vec![
                Seccion::Ventas,
                Seccion::Devoluciones,
                Seccion::Garantias,
                Seccion::Asistente,
            ],
            _ => vec![Seccion::Inventario, Seccion::Entradas],
        }
    }

    fn tarjeta(&self, titulo: &str, valor: String) -> AnyElement {
        div()
            .flex_1()
            .p_4()
            .rounded_lg()
            .bg(rgb(0x121b2e))
            .border_1()
            .border_color(rgb(0x263552))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x93a4c0))
                    .child(titulo.to_string()),
            )
            .child(div().text_xl().text_color(rgb(0x38bdf8)).child(valor))
            .into_any_element()
    }

    fn fila(&self, celdas: Vec<String>) -> AnyElement {
        div()
            .flex()
            .gap_3()
            .py_1()
            .border_b_1()
            .border_color(rgb(0x263552))
            .children(
                celdas
                    .into_iter()
                    .map(|celda| div().flex_1().text_sm().child(celda)),
            )
            .into_any_element()
    }

    fn cabecera(&self, celdas: &[&str]) -> AnyElement {
        div()
            .flex()
            .gap_3()
            .py_2()
            .text_xs()
            .text_color(rgb(0x93a4c0))
            .children(
                celdas
                    .iter()
                    .map(|celda| div().flex_1().child(celda.to_string())),
            )
            .into_any_element()
    }

    fn formulario(&self, etiquetas: &[&str], campos: usize, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(
                etiquetas
                    .iter()
                    .enumerate()
                    .take(campos)
                    .map(|(indice, etiqueta)| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x93a4c0))
                                    .child(etiqueta.to_string()),
                            )
                            .child(div().w(px(340.0)).child(Input::new(&self.campos[indice])))
                    }),
            )
            .child(
                Button::new("guardar")
                    .label("Guardar")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| this.guardar(cx))),
            )
            .into_any_element()
    }

    fn guardar(&mut self, cx: &mut Context<Self>) {
        let a = |i: usize, cx: &App| self.valor(&self.campos[i], cx);
        let resultado: Result<String, String> = match self.seccion {
            Seccion::Productos => almacen::crear_producto_con_entrada(
                &mut self.db.lock().unwrap().conn,
                &ProductoInput {
                    id_producto: None,
                    nombre: a(0, cx),
                    marca: a(1, cx),
                    categoria: a(2, cx),
                    descripcion: a(3, cx),
                    precio_entrada: a(4, cx).parse().unwrap_or(0.0),
                    precio_venta: a(5, cx).parse().unwrap_or(0.0),
                    stock: a(6, cx).parse().unwrap_or(0),
                },
                None,
                self.sesion.as_ref().map(|s| s.ci.as_str()).unwrap_or(""),
            )
            .map_err(|error| error.to_string()),
            Seccion::Empleados => admin::crear_empleado(
                &self.db.lock().unwrap().conn,
                &EmpleadoInput {
                    ci: a(0, cx),
                    nombre: a(1, cx),
                    rol: a(2, cx),
                    contrasena: a(3, cx),
                },
                self.sesion.as_ref().map(|s| s.ci.as_str()).unwrap_or(""),
            )
            .map(|_| "Empleado creado".into())
            .map_err(|error| error.to_string()),
            Seccion::Proveedores => catalogo::crear_proveedor(
                &self.db.lock().unwrap().conn,
                &ProveedorInput {
                    id_proveedor: None,
                    nombre: a(1, cx),
                    contacto: a(2, cx),
                },
            )
            .map_err(|error| error.to_string()),
            _ => Ok(String::new()),
        };
        self.mensaje = match resultado {
            Ok(texto) => {
                if texto.is_empty() {
                    "Guardado".into()
                } else {
                    texto
                }
            }
            Err(error) => error,
        };
        cx.notify();
    }

    fn vista_resumen(&self) -> AnyElement {
        let db = self.db.lock().unwrap();
        let resumen = dashboard::datos(&db.conn, "ULTIMOS_7").unwrap_or_else(|_| DashboardDatos {
            desde: String::new(),
            hasta: String::new(),
            resumen: stockbox_core::models::DashboardResumen {
                total: 0.0,
                cantidad_ventas: 0,
                unidades: 0,
                ticket_promedio: 0.0,
                productos: 0,
                stock_critico: 0,
                valor_inventario: 0.0,
            },
            ventas_por_dia: vec![],
            ventas_por_categoria: vec![],
            top_productos: vec![],
            stock_por_categoria: vec![],
        });
        let caja: Caja = admin::estado_caja(&db.conn).unwrap_or(Caja {
            estado: "—".into(),
            fecha_apertura: String::new(),
            fecha_cierre: String::new(),
            ci_empleado: String::new(),
        });
        let maximo = resumen
            .ventas_por_dia
            .iter()
            .map(|p| p.total)
            .fold(0.0, f64::max)
            .max(1.0);
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.tarjeta("Total 7 días", format!("${:.2}", resumen.resumen.total)))
                    .child(self.tarjeta(
                        "Ticket promedio",
                        format!("${:.2}", resumen.resumen.ticket_promedio),
                    ))
                    .child(self.tarjeta("Unidades", resumen.resumen.unidades.to_string()))
                    .child(
                        self.tarjeta("Stock crítico", resumen.resumen.stock_critico.to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.tarjeta(
                        "Valor inventario",
                        format!("${:.2}", resumen.resumen.valor_inventario),
                    ))
                    .child(self.tarjeta("Caja", caja.estado)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(0x121b2e))
                    .child(div().text_base().child("Ventas por día"))
                    .children(resumen.ventas_por_dia.iter().map(|punto| {
                        div()
                            .flex()
                            .gap_3()
                            .items_center()
                            .child(div().w(px(90.0)).text_xs().child(punto.fecha.clone()))
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(10.0))
                                    .rounded_full()
                                    .bg(rgb(0x172238))
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded_full()
                                            .bg(rgb(0x38bdf8))
                                            .w(px((4.0 + 300.0 * punto.total / maximo) as f32)),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(90.0))
                                    .text_right()
                                    .text_xs()
                                    .child(format!("${:.0}", punto.total)),
                            )
                    })),
            )
            .into_any_element()
    }

    fn vista_inventario(&self, cx: &mut Context<Self>) -> AnyElement {
        let termino = self.valor(&self.busqueda, cx);
        let productos = catalogo::listar_productos(&self.db.lock().unwrap().conn, Some(&termino))
            .unwrap_or_default();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().w(px(340.0)).child(Input::new(&self.busqueda)))
                    .child(
                        Button::new("buscar")
                            .label("Buscar")
                            .on_click(cx.listener(|_, _, _, _| {})),
                    ),
            )
            .child(self.cabecera(&["ID", "Producto", "Marca", "Categoría", "Precio", "Stock"]))
            .children(productos.iter().take(80).map(|producto| {
                self.fila(vec![
                    producto.id_producto.clone(),
                    producto.nombre.clone(),
                    producto.marca.clone(),
                    producto.categoria.clone(),
                    format!("${:.2}", producto.precio_venta),
                    producto.stock.to_string(),
                ])
            }))
            .into_any_element()
    }

    fn vista_ventas(&self, cx: &mut Context<Self>) -> AnyElement {
        let ventas = self.ventas();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.cabecera(&["Venta", "Orden", "Cliente", "Estado", "Total", "Fecha", ""]))
            .children(ventas.iter().take(80).enumerate().map(|(indice, venta)| {
                let orden = venta.id_orden.clone();
                div()
                    .flex()
                    .gap_3()
                    .py_1()
                    .border_b_1()
                    .border_color(rgb(0x263552))
                    .child(div().flex_1().text_sm().child(venta.id_venta.clone()))
                    .child(div().flex_1().text_sm().child(orden.clone()))
                    .child(div().flex_1().text_sm().child(venta.nombre_cliente.clone()))
                    .child(div().flex_1().text_sm().child(venta.estado.clone()))
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .child(format!("${:.2}", venta.total)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .child(venta.fecha_venta.chars().take(10).collect::<String>()),
                    )
                    .child(
                        Button::new(("factura", indice))
                            .label("Factura")
                            .on_click(cx.listener(move |this, _, _, _| {
                                this.detalle =
                                    ventas::factura(&this.db.lock().unwrap().conn, &orden)
                                        .unwrap_or_else(|error| error.to_string());
                            })),
                    )
            }))
            .into_any_element()
    }

    fn vista_entradas(&self) -> AnyElement {
        let entradas = self.entradas();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.cabecera(&["ID", "Producto", "Cantidad", "Precio", "Proveedor", "Fecha"]))
            .children(entradas.iter().take(80).map(|entrada| {
                self.fila(vec![
                    entrada.id_entrada.clone(),
                    entrada.nombre_producto.clone(),
                    entrada.cantidad.to_string(),
                    format!("${:.2}", entrada.precio_unitario),
                    entrada.id_proveedor.clone(),
                    entrada.fecha_entrada.clone(),
                ])
            }))
            .into_any_element()
    }

    fn vista_productos(&self, cx: &mut Context<Self>) -> AnyElement {
        let productos = self.productos();
        div()
            .flex()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .w(px(420.0))
                    .child(div().text_lg().child("Nuevo producto"))
                    .child(self.formulario(
                        &[
                            "Nombre",
                            "Marca",
                            "Categoría",
                            "Descripción",
                            "Precio entrada",
                            "Precio venta",
                            "Stock",
                        ],
                        7,
                        cx,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .flex_1()
                    .child(self.cabecera(&["ID", "Producto", "Marca", "Stock"]))
                    .children(productos.iter().take(60).map(|producto| {
                        self.fila(vec![
                            producto.id_producto.clone(),
                            producto.nombre.clone(),
                            producto.marca.clone(),
                            producto.stock.to_string(),
                        ])
                    })),
            )
            .into_any_element()
    }

    fn vista_empleados(&self, cx: &mut Context<Self>) -> AnyElement {
        let empleados = admin::listar_empleados(&self.db.lock().unwrap().conn).unwrap_or_default();
        div()
            .flex()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .w(px(360.0))
                    .child(div().text_lg().child("Nuevo empleado"))
                    .child(self.formulario(&["CI", "Nombre", "Rol", "Contraseña"], 4, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .flex_1()
                    .child(self.cabecera(&["CI", "Nombre", "Rol"]))
                    .children(empleados.iter().map(|empleado| {
                        self.fila(vec![
                            empleado.ci.clone(),
                            empleado.nombre.clone(),
                            empleado.rol.clone(),
                        ])
                    })),
            )
            .into_any_element()
    }

    fn vista_proveedores(&self, cx: &mut Context<Self>) -> AnyElement {
        let proveedores =
            catalogo::listar_proveedores(&self.db.lock().unwrap().conn).unwrap_or_default();
        div()
            .flex()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .w(px(360.0))
                    .child(div().text_lg().child("Nuevo proveedor"))
                    .child(self.formulario(&["ID (opcional)", "Nombre", "Contacto"], 3, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .flex_1()
                    .child(self.cabecera(&["ID", "Nombre", "Contacto"]))
                    .children(proveedores.iter().map(|proveedor| {
                        self.fila(vec![
                            proveedor.id_proveedor.clone(),
                            proveedor.nombre.clone(),
                            proveedor.contacto.clone(),
                        ])
                    })),
            )
            .into_any_element()
    }

    fn vista_devoluciones(&self, cx: &mut Context<Self>) -> AnyElement {
        let lista: Vec<Devolucion> =
            ventas::listar_devoluciones(&self.db.lock().unwrap().conn).unwrap_or_default();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().w(px(240.0)).child(Input::new(&self.dev_ref)))
                    .child(div().w(px(120.0)).child(Input::new(&self.dev_cant)))
                    .child(
                        Button::new("buscar-venta")
                            .label("Buscar venta")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let referencia = this.valor(&this.dev_ref, cx);
                                this.detalle = match ventas::buscar_linea(
                                    &this.db.lock().unwrap().conn,
                                    &referencia,
                                ) {
                                    Ok(Some(linea)) => format!(
                                        "{} x{} | ${:.2} c/u | {}",
                                        linea.nombre_producto,
                                        linea.cantidad,
                                        linea.precio_unitario,
                                        linea.estado
                                    ),
                                    Ok(None) => "No se encontró la venta".into(),
                                    Err(error) => error.to_string(),
                                };
                                cx.notify();
                            })),
                    )
                    .child(Button::new("devolver").label("Devolver").danger().on_click(
                        cx.listener(|this, _, _, cx| {
                            let referencia = this.valor(&this.dev_ref, cx);
                            let cantidad = this.valor(&this.dev_cant, cx).parse().unwrap_or(0);
                            let ci = this
                                .sesion
                                .as_ref()
                                .map(|s| s.ci.clone())
                                .unwrap_or_default();
                            this.mensaje = match ventas::devolver_venta(
                                &mut this.db.lock().unwrap().conn,
                                &referencia,
                                cantidad,
                                &ci,
                            ) {
                                Ok(id) => format!("Devolución {id} registrada"),
                                Err(error) => error.to_string(),
                            };
                            cx.notify();
                        }),
                    )),
            )
            .child(self.cabecera(&["ID", "Venta", "Producto", "Unidades", "Total"]))
            .children(lista.iter().take(50).map(|devolucion| {
                self.fila(vec![
                    devolucion.id_devolucion.clone(),
                    devolucion.id_venta.clone(),
                    devolucion.nombre_producto.clone(),
                    devolucion.cantidad.to_string(),
                    format!("${:.2}", devolucion.total),
                ])
            }))
            .into_any_element()
    }

    fn vista_garantias(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().w(px(340.0)).child(Input::new(&self.dev_ref)))
                    .child(
                        Button::new("consultar-garantia")
                            .label("Consultar")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                let referencia = this.valor(&this.dev_ref, cx);
                                this.detalle = ventas::consultar_garantia(
                                    &this.db.lock().unwrap().conn,
                                    &referencia,
                                )
                                .unwrap_or_default()
                                .join("\n");
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x93a4c0))
                    .child("Busca por código de garantía, ID de venta u orden."),
            )
            .into_any_element()
    }

    fn vista_asistente(&self, cx: &mut Context<Self>) -> AnyElement {
        let config = self.asistente.config_para_ui(&self.directorio);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_sm().text_color(rgb(0x93a4c0)).child(format!(
                "Proveedor: {} · Modelo: {} · {}",
                config.provider,
                config.modelo,
                if config.configurada {
                    "configurado"
                } else {
                    "sin key: usa ia.properties o OPENROUTER_API_KEY"
                },
            )))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.ia_chat.iter().map(|mensaje| {
                        div()
                            .p_3()
                            .rounded_lg()
                            .bg(rgb(0x172238))
                            .text_sm()
                            .child(mensaje.clone())
                    })),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().w(px(500.0)).child(Input::new(&self.ia_pregunta)))
                    .child(Button::new("enviar-ia").label("Enviar").primary().on_click(
                        cx.listener(|this, _, _, cx| {
                            let pregunta = this.valor(&this.ia_pregunta, cx);
                            if pregunta.trim().is_empty() {
                                return;
                            }
                            this.ia_chat.push(format!("Tú: {pregunta}"));
                            let snapshot =
                                asistente::snapshot_inventario(&this.db.lock().unwrap().conn)
                                    .unwrap_or_default();
                            match this
                                .asistente
                                .preguntar(&this.directorio, &snapshot, &pregunta)
                            {
                                Ok(respuesta) => {
                                    this.ia_chat.push(format!("StockBox: {respuesta}"))
                                }
                                Err(error) => this.ia_chat.push(format!("Error: {error}")),
                            }
                            cx.notify();
                        }),
                    ))
                    .child(
                        Button::new("limpiar-ia")
                            .label("Limpiar")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let _ = this.asistente.limpiar();
                                this.ia_chat.clear();
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element()
    }

    fn vista_datos(&self) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().child("Datos y respaldos"))
            .child(div().text_sm().text_color(rgb(0x93a4c0)).child(format!(
                "Base de datos: {}. Los CSV se importan/exportan desde la versión Tauri.",
                self.directorio.join("stockbox.sqlite").display()
            )))
            .child(self.cabecera(&["Concepto", "Total"]))
            .children(vec![
                self.fila(vec!["Productos".into(), self.datos.productos.to_string()]),
                self.fila(vec![
                    "Stock crítico".into(),
                    self.datos.stock_critico.to_string(),
                ]),
                self.fila(vec![
                    "Garantías vigentes".into(),
                    self.datos.garantias_vigentes.to_string(),
                ]),
            ])
            .into_any_element()
    }
}

impl Render for StockBox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fondo = rgb(0x0f172a);
        let texto = rgb(0xe6edf7);
        let suave = rgb(0x93a4c0);

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
                        .children(ROLES.iter().enumerate().map(|(indice, opcion)| {
                            Button::new(("rol", indice))
                                .label(opcion.1)
                                .when(indice == self.rol, |boton| boton.primary())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.rol = indice;
                                    this.mensaje.clear();
                                    cx.notify();
                                }))
                        })),
                )
                .child(div().w(px(320.0)).child(Input::new(&self.ci)))
                .child(div().w(px(320.0)).child(Input::new(&self.clave)))
                .child(
                    Button::new("entrar")
                        .label("Entrar")
                        .primary()
                        .on_click(cx.listener(|this, _, _, cx| {
                            let ci = this.valor(&this.ci, cx);
                            let clave = this.valor(&this.clave, cx);
                            match auth::verificar(
                                &this.db.lock().unwrap().conn,
                                &ci,
                                &clave,
                                ROLES[this.rol].0,
                            ) {
                                Ok(sesion) => {
                                    this.sesion = Some(sesion);
                                    this.mensaje.clear();
                                    this.seccion = match ROLES[this.rol].0 {
                                        "ADMINISTRADOR" => Seccion::Resumen,
                                        "CAJERO" => Seccion::Ventas,
                                        _ => Seccion::Inventario,
                                    };
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
                )
                .into_any_element();
        }

        let sesion = self.sesion.as_ref().unwrap();
        let secciones = self.secciones();
        let contenido = match self.seccion {
            Seccion::Resumen => self.vista_resumen(),
            Seccion::Inventario => self.vista_inventario(cx),
            Seccion::Ventas => self.vista_ventas(cx),
            Seccion::Entradas => self.vista_entradas(),
            Seccion::Productos => self.vista_productos(cx),
            Seccion::Empleados => self.vista_empleados(cx),
            Seccion::Proveedores => self.vista_proveedores(cx),
            Seccion::Devoluciones => self.vista_devoluciones(cx),
            Seccion::Garantias => self.vista_garantias(cx),
            Seccion::Asistente => self.vista_asistente(cx),
            Seccion::Datos => self.vista_datos(),
        };

        div()
            .flex()
            .size_full()
            .bg(fondo)
            .text_color(texto)
            .child(
                div()
                    .w(px(210.0))
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .bg(rgb(0x101a2e))
                    .border_r_1()
                    .border_color(rgb(0x263552))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("▣ StockBox"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(suave)
                            .child(format!("{} · {}", sesion.nombre, sesion.rol)),
                    )
                    .children(secciones.iter().enumerate().map(|(indice, seccion)| {
                        let destino = *seccion;
                        Button::new(("nav", indice))
                            .label(seccion.texto())
                            .when(destino == self.seccion, |boton| boton.primary())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.seccion = destino;
                                cx.notify();
                            }))
                    }))
                    .child(
                        Button::new("salir")
                            .label("Cerrar sesión")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sesion = None;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(self.seccion.texto().to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x34d399))
                                    .child("SQLite activo"),
                            ),
                    )
                    .child(contenido)
                    .when(!self.detalle.is_empty(), |columna| {
                        columna.child(
                            div()
                                .p_4()
                                .rounded_lg()
                                .bg(rgb(0x0c1528))
                                .border_1()
                                .border_color(rgb(0x263552))
                                .child(
                                    div()
                                        .text_sm()
                                        .whitespace_normal()
                                        .child(self.detalle.clone()),
                                )
                                .child(Button::new("cerrar-detalle").label("Cerrar").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.detalle.clear();
                                        cx.notify();
                                    }),
                                )),
                        )
                    })
                    .when(!self.mensaje.is_empty(), |columna| {
                        columna.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x34d399))
                                .child(self.mensaje.clone()),
                        )
                    }),
            )
            .into_any_element()
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

    gpui_platform::application().run(|cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let ci = cx.new(|cx| InputState::new(window, cx).placeholder("CI del empleado"));
                let clave = cx.new(|cx| InputState::new(window, cx).placeholder("Contraseña"));
                let campos = vec![
                    cx.new(|cx| InputState::new(window, cx).placeholder("Nombre")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Marca")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Categoría")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Descripción")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Precio entrada")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Precio venta")),
                    cx.new(|cx| InputState::new(window, cx).placeholder("Stock")),
                ];
                let busqueda =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Buscar producto"));
                let dev_ref = cx.new(|cx| InputState::new(window, cx).placeholder("ID de venta"));
                let dev_cant = cx.new(|cx| InputState::new(window, cx).placeholder("Cantidad"));
                let ia_pregunta =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Tu pregunta"));
                cx.new(|_| StockBox {
                    db: Mutex::new(db),
                    directorio,
                    sesion: None,
                    rol: 0,
                    seccion: Seccion::Resumen,
                    ci,
                    clave,
                    campos,
                    busqueda,
                    dev_ref,
                    dev_cant,
                    ia_pregunta,
                    asistente: asistente::Asistente::new(),
                    mensaje: String::new(),
                    detalle: String::new(),
                    ia_chat: Vec::new(),
                    datos,
                })
            },
        )
        .unwrap();
    });
}
