document.addEventListener("DOMContentLoaded", () => {
    const btnLogin = document.getElementById("btn-login");

    btnLogin.addEventListener("click", async () => {
        const nombre = document.getElementById("login-nombre").value.trim();
        const contrasena = document.getElementById("login-contra").value.trim();

        if (!nombre || !contrasena) {
            document.getElementById("resultado").textContent = "❌ Rellena todos los campos.";
            return;
        }

        try {
            const res = await fetch("http://localhost:3000/api/login", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ nombre_usuario: nombre, contrasena })
            });

            const data = await res.json();

            if (res.ok) {
                document.getElementById("resultado").textContent = "✅ Sesión iniciada correctamente.";
                localStorage.setItem("rb_user", JSON.stringify(data));
                window.location.href = "lobby.html";
            } else {
                document.getElementById("resultado").textContent = `❌ ${data.error || "Credenciales incorrectas"}`;
            }
        } catch (err) {
            document.getElementById("resultado").textContent = "⚠️ Error al conectar con el servidor.";
        }
    });
});
