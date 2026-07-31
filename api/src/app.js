// Fenix Strike client. Sections: state → pure helpers → rendering → actions → wiring.
// Backend (axum): GET /state, POST /move, POST /new, POST /ai.

const $ = (s) => document.querySelector(s);
const idx = (x, y) => y * 9 + x;      // (x, y) -> flat cell index
const toXY = (i) => [i % 9, Math.floor(i / 9)];

const DISCS = { s: 1, g: 2, k: 3 };   // disc count per piece type

let mode = 'hvh';
let state = null;
let selected = null;
let targets = new Set();
let aiQueued = false;

// --- pure helpers ---

function fenToPieces(fen) {
  const pieces = new Array(81).fill(null);
  fen.split('/').forEach((row, y) => {
    let x = 0;
    for (const ch of row) {
      if (/\d/.test(ch)) { x += +ch; continue; }
      pieces[idx(x, y)] = { type: ch.toLowerCase(), color: ch === ch.toUpperCase() ? 'r' : 'b' };
      x++;
    }
  });
  return pieces;
}

function phaseText() {
  switch (state.phase) {
    case 'Setup': return 'Setup — build 1 King + 3 Generals: click a piece, then an adjacent own piece';
    case 'Normal': return '';
    case 'ReconstructGeneral': return 'A General was captured — stack two soldiers to rebuild, or move instead';
    case 'ReconstructKing': return 'King captured! Put a soldier on an adjacent General';
    case 'ForcedCapture': return '⚠ Capture chain — must continue!';
    default: return '';
  }
}

// --- rendering ---

function render() {
  const pieces = fenToPieces(state.fen);
  const board = $('#board');
  board.innerHTML = '';
  for (let i = 0; i < 81; i++) {
    const cell = document.createElement('div');
    cell.className = 'cell ' + (((i % 9) + Math.floor(i / 9)) % 2 ? 'dark' : 'light');
    cell.dataset.i = i;
    const p = pieces[i];
    if (p) {
      for (let k = 0; k < DISCS[p.type]; k++) {
        const size = 58 - k * 9;
        const d = document.createElement('div');
        d.className = 'disc ' + p.color;
        d.style.width = size + '%';
        d.style.height = size + '%';
        d.style.bottom = (4 + k * 20) + '%';
        cell.appendChild(d);
      }
    }
    board.appendChild(cell);
  }

  // Forced capture: the chain square is fixed — auto-select it
  if (state.phase === 'ForcedCapture' && state.legal_moves.length) {
    selected = idx(state.legal_moves[0].from[0], state.legal_moves[0].from[1]);
    targets = new Set(state.legal_moves.map(m => idx(m.to[0], m.to[1])));
  }

  document.querySelectorAll('.cell').forEach(cell => {
    const i = +cell.dataset.i;
    if (i === selected) cell.classList.add('sel');
    if (targets.has(i)) cell.classList.add('tgt');
  });

  const turn = state.turn;
  $('#turn').innerHTML = `<span class="dot ${turn === 'Red' ? 'r' : 'b'}"></span> ${turn} to move · turn ${state.turn_count}`;
  $('#phase').textContent = phaseText();
  state.outcome ? showOverlay() : hideOverlay();
  aiStep();
}

function clickCell(i) {
  if (state.outcome) return;
  if (selected !== null && targets.has(i)) { sendMove(selected, i); return; }
  if (state.phase === 'ForcedCapture') { toast('Must continue the capture chain!'); return; }
  const moves = state.legal_moves.filter(m => idx(m.from[0], m.from[1]) === i);
  if (moves.length) {
    selected = i;
    targets = new Set(moves.map(m => idx(m.to[0], m.to[1])));
  } else {
    selected = null; targets.clear();
  }
  render();
}

// --- actions (backend calls) ---

async function sendMove(from, to) {
  const resp = await fetch('/move', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ from: toXY(from), to: toXY(to) })
  });
  const data = await resp.json();
  if (!resp.ok) { toast(data.error || 'Illegal move'); return; }
  selected = null; targets.clear();
  state = data;
  render();
}

function aiStep() {
  if (aiQueued || !state || state.outcome) return;
  const aiTurn = mode === 'aiBoth' || (mode === 'aiBlack' && state.turn === 'Black');
  if (!aiTurn) return;
  aiQueued = true;
  setTimeout(async () => {
    try {
      const resp = await fetch('/ai', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ depth: +$('#depth').value })
      });
      const data = await resp.json();
      if (!resp.ok) { toast(data.error || 'AI error'); return; }
      state = data;
      render();
    } finally { aiQueued = false; aiStep(); }
  }, 350);
}

function newGame(m) {
  mode = m;
  selected = null; targets.clear();
  fetch('/new', { method: 'POST' }).then(r => r.json()).then(d => { state = d; render(); });
  document.querySelectorAll('#controls button').forEach(b => b.classList.remove('active'));
  ({ 'hvh': $('#btn-hvh'), 'aiBlack': $('#btn-ai'), 'aiBoth': $('#btn-both') })[m].classList.add('active');
}

function showOverlay() {
  if (document.querySelector('.overlay')) return;
  const o = state.outcome;
  let txt;
  if (o.winner) {
    txt = `<div class="big" style="color:${o.winner === 'Red' ? '#f0655a' : '#9aa7b5'}">${o.winner} wins!</div>`;
    txt += `<div class="sub">${o.reason === 'king_lost' ? 'King captured and not reconstructed' : 'Threefold repetition'}</div>`;
  } else {
    txt = `<div class="big">Draw</div><div class="sub">Threefold repetition</div>`;
  }
  const ov = document.createElement('div');
  ov.className = 'overlay';
  ov.innerHTML = `<div class="card">${txt}<div style="margin-top:18px"><button onclick="newGame('${mode}')">Play again</button></div></div>`;
  document.body.appendChild(ov);
}

function hideOverlay() {
  document.querySelectorAll('.overlay').forEach(o => o.remove());
}

let toastTimer;
function toast(msg) {
  const t = $('#toast');
  t.textContent = msg;
  t.style.display = 'block';
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.style.display = 'none', 2500);
}

// --- wiring ---

$('#board').addEventListener('click', e => {
  const cell = e.target.closest('.cell');
  if (cell) clickCell(+cell.dataset.i);
});
$('#btn-hvh').onclick = () => newGame('hvh');
$('#btn-ai').onclick = () => newGame('aiBlack');
$('#btn-both').onclick = () => newGame('aiBoth');

fetch('/state').then(r => r.json()).then(d => { state = d; render(); });
