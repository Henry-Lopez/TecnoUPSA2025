import initWasm, * as wasm from "./rustball.js";

let socket = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

// URL WebSocket dinámica según entorno
const WS_URL = (window.location.protocol === "https:" ? "wss://" : "ws://") + window.location.host;

async function main() {
    const pid = localStorage.getItem("rb_pid");
    const uid = Number(localStorage.getItem("rb_uid"));

    if (!pid || !uid) {
        alert("⚠️ Error: No se encontró información de partida o usuario en localStorage.");
        await initWasm();
        return;
    }

    // 🧠 Obtener snapshot de backend
    let snap;
    try {
        const res = await fetch(`/api/snapshot/${pid}`);
        snap = await res.json();
        console.log("📦 Snapshot recibido:", snap);
    } catch (e) {
        alert("❌ No se pudo obtener el snapshot del servidor.");
        console.error(e);
        return;
    }

    // Guardar IDs de jugadores en localStorage
    const f1 = snap.formaciones[0]?.id_usuario || 0;
    const f2 = snap.formaciones[1]?.id_usuario || 0;
    localStorage.setItem("rb_id_left", Math.min(f1, f2));
    localStorage.setItem("rb_id_right", Math.max(f1, f2));

    // 🧠 Inicializa WASM
    await initWasm();

    // ✅ Función global para que WASM pueda enviar mensajes
    globalThis.sendOverWS = function (msg) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            const wrapped = JSON.stringify({ uid_origen: uid, contenido: msg });
            socket.send(wrapped);
        } else {
            console.warn("🔌 WebSocket no disponible para enviar:", msg);
        }
    };

    // 🧠 Solo conectar WebSocket si el snapshot está listo
    if (snap?.estado === "playing" && snap?.proximo_turno !== null) {
        wasm.set_game_state(JSON.stringify(snap), uid);
        initWebSocket(pid, uid);
    } else {
        console.warn("⏳ Aún no se han elegido ambas formaciones. WebSocket no se conectará.");
    }
}

function initWebSocket(partidaId, userId) {
    if (socket && socket.readyState !== WebSocket.CLOSED && socket.readyState !== WebSocket.CLOSING) return;

    socket = new WebSocket(`${WS_URL}/api/ws/${partidaId}/${userId}`);

    socket.onopen = () => {
        console.log("🟢 WebSocket conectado");
        reconnectAttempts = 0;
    };

    socket.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            const uidLocal = Number(localStorage.getItem("rb_uid"));

            // ✅ Ignorar mensajes del propio usuario
            if (data.uid_origen && data.uid_origen === uidLocal) return;

            // ✅ Si es un snapshot reenviado
            if (data.tipo === "snapshot") {
                console.log("📦 Snapshot reenviado recibido");
                wasm.set_game_state(JSON.stringify(data.contenido), uidLocal);
                return;
            }

            // ✅ Si es una jugada normal
            if (wasm && wasm.receive_ws_message) {
                const contenido = typeof data.contenido === "string"
                    ? data.contenido
                    : JSON.stringify(data.contenido);
                wasm.receive_ws_message(contenido);
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
            console.log("🔁 Reintentando conexión WebSocket...");
            setTimeout(() => initWebSocket(partidaId, userId), 3000);
        } else {
            alert("❌ El servidor no está disponible. Intenta más tarde.");
        }
    };
}

main();
