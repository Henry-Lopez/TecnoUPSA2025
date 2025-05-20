import initWasm, * as wasm from "./rustball.js";

let socket = null;

async function main() {
    const pid = localStorage.getItem("rb_pid");
    const uid = Number(localStorage.getItem("rb_uid"));

    if (!pid || !uid) {
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
    globalThis.sendOverWS = function(msg) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            socket.send(msg);
        } else {
            console.warn("🔌 WebSocket no disponible para enviar:", msg);
        }
    };

    // Setea el snapshot
    wasm.set_game_state(JSON.stringify(snap), uid);

    // Conectar WebSocket
    initWebSocket(pid, uid);
}

function initWebSocket(partidaId, userId) {
    socket = new WebSocket(`ws://127.0.0.1:3000/ws/${partidaId}/${userId}`);

    socket.onopen = () => {
        console.log("🟢 WebSocket conectado");
    };

    socket.onmessage = (event) => {
        const data = event.data;
        console.log("📨 Mensaje WS:", data);

        // ✅ Llamar a la función exportada desde Rust/WASM
        if (wasm && wasm.receive_ws_message) {
            wasm.receive_ws_message(data);
        }
    };

    socket.onerror = (err) => {
        console.error("❌ WebSocket error:", err);
    };

    socket.onclose = () => {
        console.warn("🔴 WebSocket cerrado");
    };
}

main();
