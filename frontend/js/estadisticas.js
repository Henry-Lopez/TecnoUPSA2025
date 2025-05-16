document.addEventListener("DOMContentLoaded", async () => {
    const user = JSON.parse(localStorage.getItem("rb_user"));
    const idUsuario = user?.id_usuario;

    if (!idUsuario) {
        alert("⚠️ No se encontró el usuario en sesión");
        window.location.href = "login.html";
        return;
    }

    try {
        const response = await fetch(`/api/estadisticas/${idUsuario}`);
        if (!response.ok) throw new Error("Error al cargar estadísticas");

        const data = await response.json();

        document.getElementById("jugadas").textContent = data.partidas_jugadas ?? 0;
        document.getElementById("ganadas").textContent = data.partidas_ganadas ?? 0;
        document.getElementById("goles_favor").textContent = data.goles_a_favor ?? 0;
        document.getElementById("goles_contra").textContent = data.goles_en_contra ?? 0;
    } catch (err) {
        console.error(err);
        alert("❌ Error cargando estadísticas");
    }
});
