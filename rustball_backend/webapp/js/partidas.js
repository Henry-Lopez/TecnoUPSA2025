import { entrarPartida } from "./utils.js";

document.addEventListener("DOMContentLoaded", async () => {
    const user = JSON.parse(localStorage.getItem("rb_user"));
    const id   = user?.id_usuario;
    if (!id) return (window.location.href = "login.html");

    try {
        const res      = await fetch(`/api/mis_partidas/${id}`);
        const partidas = await res.json();
        const cont     = document.getElementById("lista-partidas");

        if (partidas.length === 0) {
            cont.innerHTML = "<p>No tienes partidas registradas.</p>";
            return;
        }

        partidas.forEach((p) => {
            const rival = p.id_usuario_1 === id ? p.id_usuario_2 : p.id_usuario_1;
            const div   = document.createElement("div");
            div.className = "partida";
            div.innerHTML = `
                <p>Partida #${p.id_partida} vs Jugador #${rival}</p>
                <button onclick="continuar(${p.id_partida})">Continuar</button>
            `;
            cont.appendChild(div);
        });
    } catch (err) {
        console.error(err);
        alert("❌ Error al cargar partidas.");
    }
});

/* 👇 Usamos la utilidad común para entrar a una partida */
async function continuar(idPartida) {
    const user = JSON.parse(localStorage.getItem("rb_user"));

    try {
        const res  = await fetch(`/api/partida_detalle/${idPartida}`);
        const data = await res.json();

        entrarPartida(idPartida, data.id_usuario_1, data.id_usuario_2, user.id_usuario);
    } catch (err) {
        console.error("❌ Error al obtener detalles de la partida:", err);
        alert("No se pudo continuar la partida.");
    }
}

window.continuar = continuar;
