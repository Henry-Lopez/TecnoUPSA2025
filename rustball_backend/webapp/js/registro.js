/*  Módulo encargado del formulario de registro  */
import { post } from "./api.js";

console.log("🔍 registro.js cargado — esperando al DOM…");

/* ------------------------------------------------------------------ */
/*  Ejecutamos código cuando el DOM ya está disponible                 */
/* ------------------------------------------------------------------ */
document.addEventListener("DOMContentLoaded", () => {
    console.log("🔍 DOM completamente cargado.");

    /* Helpers */
    const $   = (id) => document.getElementById(id);           // sin '#'
    const log = (msg) => { const box = $("resultado"); if (box) box.textContent = msg; };

    /* Referencias a inputs y botón */
    const btnRegistrar = $("btn-registrar");                    // ✅ id sin #

    if (!btnRegistrar) {
        console.error("✖ No se encontró el botón con id='btn-registrar'");
        log("Error interno: botón no disponible");
        return;
    }
    console.log("✅ Botón de registro encontrado.");

    /* Acción de registro */
    btnRegistrar.addEventListener("click", async () => {
        const nombre = $("reg-nombre").value.trim();
        const correo = $("reg-correo").value.trim();
        const pass   = $("reg-contra").value.trim();

        if (!nombre || !correo || !pass) {
            log("⚠️ Completa todos los campos.");
            return;
        }

        try {
            log("🔄 Enviando datos…");

            const user = await post("/registro", {
                nombre_usuario: nombre,
                correo,
                contrasena: pass,
            });

            console.log("✅ Registro exitoso:", user);
            localStorage.setItem("rb_user", JSON.stringify(user));

            log("✅ Registro exitoso. Redirigiendo…");
            setTimeout(() => (window.location.href = "lobby.html"), 800);
        } catch (e) {
            console.error("❌ Error de registro:", e);
            log(`❌ ${e.message || "Error al registrar usuario"}`);
        }
    });
});
