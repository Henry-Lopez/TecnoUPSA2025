/* Pequeño wrapper para las llamadas a /api */
const BASE = "/api";

/* POST con body JSON ------------------------------------------------------ */
export async function post(path, payload) {
    const res  = await fetch(`${BASE}${path}`, {
        method : "POST",
        headers: { "Content-Type": "application/json" },
        body   : JSON.stringify(payload),
    });

    /* intentamos parsear JSON; si falla, devolvemos texto */
    let data;
    try   { data = await res.json(); }
    catch { data = await res.text(); }

    if (!res.ok) {
        throw new Error(data?.error || res.statusText);
    }
    return data;
}

/* GET (por si lo necesitas más adelante) ---------------------------------- */
export async function get(path) {
    const res = await fetch(`${BASE}${path}`);
    let data;
    try   { data = await res.json(); }
    catch { data = await res.text(); }

    if (!res.ok) {
        throw new Error(data?.error || res.statusText);
    }
    return data;
}
