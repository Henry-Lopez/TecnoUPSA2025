import initWasm, * as wasm from "./rustball.js";

let socket = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

const WS_URL = (window.location.protocol === "https:" ? "wss://" : "ws://") + window.location.host;

async function main() {
    const pid = localStorage.getItem("rb_pid");
    const uid = Number(localStorage.getItem("rb_uid"));

    if (!pid || !uid) {
        alert("⚠️ No se encontró información de partida o usuario en localStorage.");
        return;
    }

    // 🔍 Obtener snapshot
    let snap;
    try {
        const res = await fetch(`/api/snapshot/${pid}`);
        snap = await res.json();
        console.log("📦 Snapshot recibido:", snap);

        // 🔎 Verificación explícita de estructura
        if (!snap.snapshot || !Array.isArray(snap.snapshot.piezas)) {
            console.error("❌ Estructura inválida: faltan piezas en snapshot.");
            return;
        }

        if (!Array.isArray(snap.formaciones)) {
            console.error("❌ Estructura inválida: 'formaciones' no es un arreglo.");
            return;
        }

        if (typeof snap.id_partida !== "number") {
            console.error("❌ Estructura inválida: 'id_partida' no es número.");
            return;
        }

        const formaciones = snap.formaciones;
        if (formaciones.length < 2) {
            console.warn("⚠️ Aún no hay 2 formaciones.");
            return;
        }

        const f1 = formaciones[0].id_usuario || 0;
        const f2 = formaciones[1].id_usuario || 0;

        if (f1 === 0 || f2 === 0) {
            console.warn("⚠️ IDs inválidos en formaciones.");
            return;
        }

        localStorage.setItem("rb_id_left", Math.min(f1, f2));
        localStorage.setItem("rb_id_right", Math.max(f1, f2));
    } catch (e) {
        console.error("❌ Error al obtener snapshot:", e);
        alert("Error al obtener el estado de la partida.");
        return;
    }

    await initWasm();
    console.log("✅ WASM inicializado");
    console.log("🧠 Llamando a set_game_state...");

    globalThis.sendOverWS = function (msg) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            const wrapped = JSON.stringify({ uid_origen: uid, contenido: msg });
            socket.send(wrapped);
        } else {
            console.warn("🔌 WebSocket no disponible para enviar:", msg);
        }
    };

    if (snap.estado === "playing" && snap.proximo_turno != null) {
        try {
            wasm.set_game_state(JSON.stringify(snap), uid);
            console.log("🧠 set_game_state ejecutado correctamente");
        } catch (e) {
            console.error("❌ Error en set_game_state:", e);
        }

        initWebSocket(pid, uid);
    } else {
        console.warn("⏳ Esperando a que ambos jugadores elijan formación...");
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

            if (data.uid_origen === uidLocal) return;

            if (data.tipo === "snapshot") {
                console.log("📦 Snapshot reenviado recibido");
                wasm.set_game_state(JSON.stringify(data.contenido), uidLocal);
            } else if (wasm.receive_ws_message) {
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
            setTimeout(() => initWebSocket(partidaId, userId), 3000);
        } else {
            alert("❌ El servidor no está disponible. Intenta más tarde.");
        }
    };
}

main();
