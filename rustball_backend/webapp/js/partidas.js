document.addEventListener("DOMContentLoaded", async () => {
    const user = JSON.parse(localStorage.getItem("rb_user"));
    const id = user?.id_usuario;
    if (!id) return (window.location.href = "login.html");

    try {
        const res = await fetch(`/api/mis_partidas/${id}`);
        const partidas = await res.json();
        const contenedor = document.getElementById("lista-partidas");

        if (partidas.length === 0) {
            contenedor.innerHTML = "<p>No tienes partidas registradas.</p>";
            return;
        }

        partidas.forEach((p) => {
            const rival = p.id_usuario_1 === id ? p.id_usuario_2 : p.id_usuario_1;
            const div = document.createElement("div");
            div.className = "partida";
            div.innerHTML = `
        <p>Partida #${p.id_partida} vs Jugador #${rival}</p>
        <button onclick="continuar(${p.id_partida})">Continuar</button>
      `;
            contenedor.appendChild(div);
        });
    } catch (err) {
        alert("❌ Error al cargar partidas.");
    }
});

function continuar(idPartida) {
    localStorage.setItem("rb_pid", idPartida);
    window.location.href = "/game/index.html"; // o como arranques la partida
}
