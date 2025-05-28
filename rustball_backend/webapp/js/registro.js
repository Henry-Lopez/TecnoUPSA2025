import { post } from "./api.js";

console.log("🔍 registro.js cargado — esperando al DOM…");

document.addEventListener("DOMContentLoaded", () => {
    const $ = (id) => document.getElementById(id);
    const log = (msg) => {
        const box = $("resultado");
        if (box) box.textContent = msg;
    };

    const btnRegistrar = $("btn-registrar");
    if (!btnRegistrar) {
        console.error("✖ No se encontró el botón con id='btn-registrar'");
        log("Error interno: botón no disponible");
        return;
    }

    btnRegistrar.addEventListener("click", async () => {
        try {
            const nombre = $("reg-nombre").value.trim();
            const correo = $("reg-correo").value.trim();
            const pass = $("reg-contra").value.trim();

            if (!nombre || !correo || !pass) {
                log("⚠️ Completa todos los campos.");
                return;
            }

            const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
            if (!emailRegex.test(correo)) {
                log("⚠️ Correo electrónico no válido.");
                return;
            }

            if (pass.length < 6) {
                log("⚠️ La contraseña debe tener al menos 6 caracteres.");
                return;
            }

            const nombreRegex = /^[a-zA-Z0-9_]+$/;
            if (!nombreRegex.test(nombre)) {
                log("⚠️ El nombre de usuario solo puede contener letras, números y guiones bajos.");
                return;
            }

            log("🔄 Enviando datos…");

            const hashedPass = CryptoJS.SHA256(pass).toString(CryptoJS.enc.Hex);

            const user = await post("/registro", {
                nombre_usuario: nombre,
                correo,
                contrasena: hashedPass,
            });

            console.log("✅ Registro exitoso:", user);
            localStorage.setItem("rb_user", JSON.stringify(user));

            log("✅ Registro exitoso. Redirigiendo…");
            setTimeout(() => (window.location.href = "lobby.html"), 800);
        } catch (e) {
            log(`❌ ${e.message || "Error al registrar usuario"}`);
            console.error(e);
        }
    });
});
