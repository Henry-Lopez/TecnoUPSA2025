import { post, get } from "./api.js";
import { entrarPartida } from "./utils.js";

document.addEventListener("DOMContentLoaded", () => {
    const $   = (id) => document.getElementById(id);
    const log = (m)  => { const b = $("resultado"); if (b) b.textContent = m; };

    const user = JSON.parse(localStorage.getItem("rb_user") || "null");
    if (!user) { window.location.href = "registro.html"; return; }

    $("lbl-user").textContent = `${user.nombre_usuario} (#${user.id_usuario})`;

    $("btn-partida").addEventListener("click", async () => {
        const rival = parseInt($("usuario-2").value.trim(), 10);
        if (!rival) { log("⚠️ Ingresa el ID del rival."); return; }

        try {
            log("🔄 Creando / buscando partida…");
            const partida = await post("/partida", {
                id_usuario_1: user.id_usuario,
                id_usuario_2: rival,
            });

            entrarPartida(partida.id_partida, partida.id_usuario_1, partida.id_usuario_2, user.id_usuario);
        } catch (e) {
            log(`❌ ${e.message}`);
        }
    });

    $("btn-estadisticas").addEventListener("click", () => {
        window.location.href = "/estadisticas.html";
    });

    $("btn-partidas")?.addEventListener("click", () => {
        window.location.href = "/partidas.html";
    });

    async function cargarPendientes() {
        try {
            const partidas = await get(`/pendientes/${user.id_usuario}`);
            const mensajesDiv = $("mensajes");
            mensajesDiv.innerHTML = "";

            if (partidas.length === 0) {
                mensajesDiv.innerText = "No tienes retos pendientes.";
                return;
            }

            partidas.forEach(p => {
                const otro = (p.id_usuario_1 === user.id_usuario)
                    ? p.id_usuario_2
                    : p.id_usuario_1;

                const div = document.createElement("div");
                div.innerHTML = `
                    🎮 Jugador ${otro} te ha desafiado.
                    <button>Aceptar</button>
                `;

                div.querySelector("button").onclick = async () => {
                    try {
                        const res = await fetch(`/api/partida_detalle/${p.id_partida}`);
                        const data = await res.json();
                        entrarPartida(p.id_partida, data.id_usuario_1, data.id_usuario_2, user.id_usuario);
                    } catch (err) {
                        console.error("❌ Error al obtener detalles de la partida:", err);
                        alert("No se pudo continuar la partida.");
                    }
                };

                mensajesDiv.appendChild(div);
            });
        } catch (e) {
            $("mensajes").innerText = "Error al cargar retos.";
            console.error(e);
        }
    }

    cargarPendientes();
});
