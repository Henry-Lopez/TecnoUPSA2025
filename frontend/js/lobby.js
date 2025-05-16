import { post } from "./api.js";

document.addEventListener("DOMContentLoaded", () => {
    const $   = (id) => document.getElementById(id);
    const log = (m)  => { const b = $("resultado"); if (b) b.textContent = m; };

    /* ── Sesión ---------------------------------------------------------- */
    const user = JSON.parse(localStorage.getItem("rb_user") || "null");
    if (!user) { window.location.href = "registro.html"; return; }

    $("lbl-user").textContent = `${user.nombre_usuario} (#${user.id_usuario})`;

    /* ── Botón: Buscar/Iniciar partida ---------------------------------- */
    $("btn-partida").addEventListener("click", async () => {
        const rival = parseInt($("usuario-2").value.trim(), 10);
        if (!rival) { log("⚠️ Ingresa el ID del rival."); return; }

        try {
            log("🔄 Creando / buscando partida…");
            const partida = await post("/partida", {
                id_usuario_1: user.id_usuario,
                id_usuario_2: rival,
            });

            // Guarda datos en localStorage para el juego
            localStorage.setItem("rb_pid", partida.id_partida);
            localStorage.setItem("rb_uid", user.id_usuario);

            log("✅ Partida lista, cargando juego…");
            window.location.href = "/game/index.html";  // ← juego WebAssembly
        } catch (e) {
            log(`❌ ${e.message}`);
        }
    });

    /* ── Botón: Ver estadísticas ---------------------------------------- */
    $("btn-estadisticas").addEventListener("click", () => {
        window.location.href = "/estadisticas.html";


    });

    /* ── (Opcional) Botón: Ver partidas anteriores ---------------------- */
    $("btn-partidas")?.addEventListener("click", () => {
        alert("🔧 Próximamente: historial de partidas");
        // Puedes luego hacer window.location.href = "partidas.html";
    });
});
