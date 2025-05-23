import initWasm, * as wasm from "./rustball.js";

let socket = null;
let reconnectInterval = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

// URL WebSocket dinámico
const WS_URL = (window.location.protocol === "https:" ? "wss://" : "ws://") + window.location.host;

async function main() {
    const pid = localStorage.getItem("rb_pid");
    const uid = Number(localStorage.getItem("rb_uid"));

    if (!pid || !uid) {
        alert("⚠️ Error: No se encontró información de partida o usuario en localStorage.");
        await initWasm();
        return;
    }

    const snap = await (await fetch(`/api/snapshot/${pid}`)).json();
    console.log("📦 Snapshot recibido:", snap);

    // Guardar IDs de jugadores
    const f1 = snap.formaciones[0]?.id_usuario || 0;
    const f2 = snap.formaciones[1]?.id_usuario || 0;
    localStorage.setItem("rb_id_left", Math.min(f1, f2));
    localStorage.setItem("rb_id_right", Math.max(f1, f2));

    // Inicializa WASM
    await initWasm();

    // ✅ Exponer función global para que Rust/WASM pueda enviar mensajes
    globalThis.sendOverWS = function (msg) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            const uid = localStorage.getItem("rb_uid");
            const wrapped = JSON.stringify({ uid_origen: Number(uid), contenido: msg });
            socket.send(wrapped);
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
    if (socket && socket.readyState !== WebSocket.CLOSED) return;

    socket = new WebSocket(`${WS_URL}/api/ws/${partidaId}/${userId}`);

    socket.onopen = () => {
        console.log("🟢 WebSocket conectado");
        reconnectAttempts = 0;
        clearInterval(reconnectInterval);
    };

    socket.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            const uidLocal = localStorage.getItem("rb_uid");

            // ✅ Ignorar mensajes propios
            if (data.uid_origen && data.uid_origen.toString() === uidLocal) return;

            // ✅ Si es un snapshot reenviado
            if (data.tipo === "snapshot") {
                console.log("📦 Snapshot reenviado recibido");
                wasm.set_game_state(JSON.stringify(data.contenido), Number(uidLocal));
                return;
            }

            // ✅ Si es jugada normal
            if (wasm && wasm.receive_ws_message) {
                wasm.receive_ws_message(
                    typeof data.contenido === "string" ? data.contenido : JSON.stringify(data.contenido)
                );
            }
        } catch (e) {
            console.error("❌ Error al procesar mensaje:", e);
        }
    };

    socket.onerror = (err) => {
        console.error("❌ WebSocket error:", err);
    };

    socket.onclose = () => {
        console.warn("🔴 WebSocket cerrado.");
        if (reconnectAttempts < maxReconnectAttempts) {
            reconnectAttempts++;
            reconnectInterval = setInterval(() => {
                console.log("🔁 Reintentando conexión WebSocket...");
                initWebSocket(partidaId, userId);
            }, 3000);
        } else {
            alert("❌ Servidor no disponible, intenta más tarde.");
        }
    };
}

main();
