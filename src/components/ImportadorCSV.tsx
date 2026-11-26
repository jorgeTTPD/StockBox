import { useState } from "react";
import { api } from "../api";
import { errorTexto } from "../formato";
import type { VistaPrevia } from "../types";
import { DataTable, Modal, type Avisar, type Columna } from "./UI";

export default function ImportadorCSV({
  tipo,
  titulo,
  boton,
  avisar,
  onConfirmar,
}: {
  tipo: string;
  titulo: string;
  boton: string;
  avisar: Avisar;
  onConfirmar: (ruta: string) => Promise<void>;
}) {
  const [ruta, setRuta] = useState<string | null>(null);
  const [vista, setVista] = useState<VistaPrevia | null>(null);
  const [cargando, setCargando] = useState(false);

  const elegir = async () => {
    try {
      const archivo = await api.elegirArchivo();
      if (!archivo) return;
      setRuta(archivo);
      setVista(await api.vistaPreviaCsv(archivo, tipo));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  const confirmar = async () => {
    if (!ruta) return;
    setCargando(true);
    try {
      await onConfirmar(ruta);
      setVista(null);
      setRuta(null);
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  const columnas: Columna<string[]>[] = vista
    ? vista.columnas.map((nombre, indice) => ({
        titulo: nombre,
        valor: (fila: string[]) => fila[indice] ?? "",
      }))
    : [];

  return (
    <>
      <button className="secundario" onClick={elegir}>{boton}</button>
      {vista && (
        <Modal titulo={`Vista previa: ${titulo}`} onCerrar={() => setVista(null)}>
          <p className="texto-suave">
            {vista.total} fila(s) encontradas. Revisa el contenido antes de importar.
          </p>
          <DataTable filas={vista.filas} columnas={columnas} vacio="El archivo está vacío" />
          {vista.total > vista.filas.length && (
            <p className="texto-suave">…y {vista.total - vista.filas.length} filas más.</p>
          )}
          <div className="acciones">
            <button className="neutro" onClick={() => setVista(null)}>Cancelar</button>
            <button className="primario" onClick={confirmar} disabled={cargando}>
              {cargando ? "Importando…" : "Confirmar importación"}
            </button>
          </div>
        </Modal>
      )}
    </>
  );
}
