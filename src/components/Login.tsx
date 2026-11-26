import { useState, type FormEvent } from "react";
import { api } from "../api";
import type { Rol, Sesion } from "../types";
import { Campo, type Avisar } from "./UI";

const roles: { valor: Rol; texto: string; ci: string; clave: string }[] = [
  { valor: "ADMINISTRADOR", texto: "Administrador", ci: "ADM-001", clave: "user" },
  { valor: "CAJERO", texto: "Cajero", ci: "DEMO-VEN-001", clave: "ventas123" },
  { valor: "ALMACENERO", texto: "Almacenero", ci: "DEMO-ALM-001", clave: "almacen123" },
];

export default function Login({
  onLogin,
  avisar,
  temaOscuro,
  cambiarTema,
}: {
  onLogin: (sesion: Sesion) => void;
  avisar: Avisar;
  temaOscuro: boolean;
  cambiarTema: () => void;
}) {
  const [rol, setRol] = useState<Rol>("ADMINISTRADOR");
  const [ci, setCi] = useState("ADM-001");
  const [clave, setClave] = useState("user");
  const [verDemo, setVerDemo] = useState(false);
  const [cargando, setCargando] = useState(false);

  const seleccionar = (opcion: (typeof roles)[number]) => {
    setRol(opcion.valor);
    setCi(opcion.ci);
    setClave(opcion.clave);
  };

  const entrar = async (evento: FormEvent) => {
    evento.preventDefault();
    setCargando(true);
    try {
      onLogin(await api.iniciarSesion(ci, clave, rol));
    } catch (error) {
      avisar(String(error), "error");
    } finally {
      setCargando(false);
    }
  };

  return (
    <div className="login-fondo">
      <div className="login-tarjeta">
        <div className="login-marca">
          <div className="login-logo">▣</div>
          <div>
            <h1>StockBox</h1>
            <p>Inventario, entradas y ventas</p>
          </div>
        </div>
        <h2>Iniciar sesión</h2>
        <p className="texto-suave">Elige tu rol para acceder al sistema.</p>
        <div className="login-roles">
          {roles.map((opcion) => (
            <button
              key={opcion.valor}
              type="button"
              className={rol === opcion.valor ? "rol activo" : "rol"}
              onClick={() => seleccionar(opcion)}
            >
              {opcion.texto}
            </button>
          ))}
        </div>
        <form onSubmit={entrar} className="login-formulario">
          <Campo etiqueta="CI del empleado">
            <input value={ci} onChange={(e) => setCi(e.target.value)} autoFocus />
          </Campo>
          <Campo etiqueta="Contraseña">
            <input
              type="password"
              value={clave}
              onChange={(e) => setClave(e.target.value)}
            />
          </Campo>
          <button className="primario ancho" disabled={cargando}>
            {cargando ? "Entrando…" : "Entrar"}
          </button>
        </form>
        <div className="barra" style={{ marginTop: 12, justifyContent: "space-between" }}>
          <button className="neutro mini" onClick={cambiarTema}>
            {temaOscuro ? "☀ Tema claro" : "☾ Tema oscuro"}
          </button>
          <button className="enlace" onClick={() => setVerDemo(!verDemo)}>
            {verDemo ? "Ocultar credenciales de prueba" : "Ver credenciales de prueba"}
          </button>
        </div>
        {verDemo && (
          <div className="demo">
            {roles.map((opcion) => (
              <div key={opcion.valor}>
                <strong>{opcion.texto}</strong>: {opcion.ci} / {opcion.clave}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
