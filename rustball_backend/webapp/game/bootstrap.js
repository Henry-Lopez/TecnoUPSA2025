import initWasm, * as wasm from "./rustball.js";

let socket = null;
let reconnectInterval = null;

// Usa el dominio de producción
const HOST = "rustball.lat";
const BASE_URL = `https://${HOST}`;
const WS_URL = `wss://${HOST}`;

async function main() {
    const pid = localStorage.getItem("rb_pid");
    const uid = Number(localStorage.getItem("rb_uid"));

    if (!pid || !uid) {
        alert("⚠️ Error: No se encontró información de partida o usuario en localStorage.");
        await initWasm();
        return;
    }

    const snap = await (await fetch(`${BASE_URL}/api/snapshot/${pid}`)).json();
    console.log("📦 Snapshot recibido:", snap);

    // Guardar IDs de jugadores
    const f1 = snap.formaciones[0]?.id_usuario || 0;
    const f2 = snap.formaciones[1]?.id_usuario || 0;
    localStorage.setItem("rb_id_left", Math.min(f1, f2));
    localStorage.setItem("rb_id_right", Math.max(f1, f2));

    // Inicializa WASM
    await initWasm();

    // ✅ Exponer función global para que Rust/WASM pueda enviar mensajes
    globalThis.sendOverWS = function(msg) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            socket.send(msg);
        } else {
            console.warn("🔌 WebSocket no disponible para enviar:", msg);
        }
    };

    // 🧠 Solo conectar si ya están las formaciones
    if (snap.estado === "playing" && snap.turno_actual !== null) {
        wasm.set_game_state(JSON.stringify(snap), uid);
        initWebSocket(pid, uid);
    } else {
        console.warn("⏳ Aún no se han elegido ambas formaciones. WebSocket no se conectará.");
    }
}

function initWebSocket(partidaId, userId) {
    if (socket && socket.readyState !== WebSocket.CLOSED) {
        return; // evita doble conexión
    }

    socket = new WebSocket(`${WS_URL}/api/ws/${partidaId}/${userId}`);

    socket.onopen = () => {
        console.log("🟢 WebSocket conectado");
        clearInterval(reconnectInterval);
    };

    socket.onmessage = (event) => {
        const data = event.data;
        console.log("📨 Mensaje WS:", data);
        if (wasm && wasm.receive_ws_message) {
            wasm.receive_ws_message(data);
        }
    };

    socket.onerror = (err) => {
        console.error("❌ WebSocket error:", err);
    };

    socket.onclose = () => {
        console.warn("🔴 WebSocket cerrado. Intentando reconectar...");
        reconnectInterval = setInterval(() => {
            console.log("🔁 Reintentando conexión WebSocket...");
            initWebSocket(partidaId, userId);
        }, 3000);
    };
}

main();
