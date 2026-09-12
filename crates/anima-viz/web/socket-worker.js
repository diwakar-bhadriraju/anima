// ANIMA viz socket worker: owns the WebSocket so frame delivery survives
// page throttling (background/occluded tabs defer page-owned sockets).
// The worker reconnects, batches, and forwards every frame to the page.
let ws = null;
let closed = false;

function connect() {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  ws = new WebSocket(`${proto}://${location.host}/ws`);
  ws.onopen = () => postMessage({ t: "ws-open" });
  ws.onclose = () => {
    postMessage({ t: "ws-closed" });
    if (!closed) setTimeout(connect, 1000);
  };
  ws.onerror = () => { try { ws.close(); } catch (e) {} };
  ws.onmessage = (m) => postMessage({ t: "ws-frame", data: m.data });
}

onmessage = (e) => {
  const d = e.data;
  if (d.t === "start") { closed = false; connect(); }
  else if (d.t === "send" && ws && ws.readyState === 1) ws.send(d.data);
  else if (d.t === "stop") { closed = true; if (ws) ws.close(); }
};
