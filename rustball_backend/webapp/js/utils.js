/**
 * Guarda en localStorage todos los datos necesarios para entrar a una partida
 * y redirige automáticamente al juego.
 *
 * @param {number} idPartida - ID de la partida
 * @param {number} id1 - ID del primer jugador
 * @param {number} id2 - ID del segundo jugador
 * @param {number} miId - ID del usuario actual (el que está entrando)
 */
export function entrarPartida(idPartida, id1, id2, miId) {
    const left = Math.min(id1, id2);
    const right = Math.max(id1, id2);

    localStorage.setItem("rb_pid", idPartida);
    localStorage.setItem("rb_uid", miId);
    localStorage.setItem("rb_id_left", left);
    localStorage.setItem("rb_id_right", right);

    window.location.href = "/game/index.html";
}
