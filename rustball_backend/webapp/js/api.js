// js/api.js
const BASE = "https://rustball.lat/api"; // ✅ Cambiado para entorno en producción

/* POST con body JSON */
export async function post(path, payload) {
    const res = await fetch(`${BASE}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
    });

    const contentType = res.headers.get("content-type");
    const data = contentType && contentType.includes("application/json")
        ? await res.json()
        : await res.text();

    if (!res.ok) {
        throw new Error(typeof data === "string" ? data : data?.error || res.statusText);
    }

    return data;
}

/* GET simple */
export async function get(path) {
    const res = await fetch(`${BASE}${path}`);

    const contentType = res.headers.get("content-type");
    const data = contentType && contentType.includes("application/json")
        ? await res.json()
        : await res.text();

    if (!res.ok) {
        throw new Error(typeof data === "string" ? data : data?.error || res.statusText);
    }

    return data;
}
