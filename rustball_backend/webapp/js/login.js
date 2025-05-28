import { post } from "./api.js";

console.log("🔍 login.js cargado — esperando al DOM…");

document.addEventListener("DOMContentLoaded", () => {
    const $ = (id) => document.getElementById(id);
    const log = (msg) => {
        const box = $("resultado");
        if (box) box.textContent = msg;
    };

    const btnLogin = $("btn-login");
    if (!btnLogin) {
        console.error("✖ No se encontró el botón con id='btn-login'");
        log("Error interno: botón no disponible");
        return;
    }

    btnLogin.addEventListener("click", async () => {
        try {
            const nombre = $("login-nombre").value.trim();
            const pass = $("login-contra").value.trim();

            if (!nombre || !pass) {
                log("⚠️ Completa todos los campos.");
                return;
            }

            const nombreRegex = /^[a-zA-Z0-9_]+$/;
            if (!nombreRegex.test(nombre)) {
                log("⚠️ Nombre de usuario inválido. Solo letras, números y guiones bajos.");
                return;
            }

            if (pass.length < 6) {
                log("⚠️ La contraseña debe tener al menos 6 caracteres.");
                return;
            }

            log("🔄 Enviando datos…");

            const hashedPass = CryptoJS.SHA256(pass).toString(CryptoJS.enc.Hex);

            const data = await post("/login", {
                nombre_usuario: nombre,
                contrasena: hashedPass,
            });

            console.log("✅ Login exitoso:", data);
            localStorage.setItem("rb_user", JSON.stringify(data));

            log("✅ Sesión iniciada. Redirigiendo…");
            setTimeout(() => (window.location.href = "lobby.html"), 800);
        } catch (e) {
            log("❌ Error inesperado. Intenta de nuevo.");
            console.error(e);
        }
    });
});
