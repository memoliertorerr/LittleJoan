// @ts-check
// The focus timer's panel: a ring that empties as the phase runs, Start / Pause, Reset and Skip,
// and a short Web Audio chime when a phase ends — only while the panel is open: Little Joan has
// no background alarms. The timer (with its end time) is kept in littleJoan.storage, so a
// reopened panel shows the time left. The rules are logic.js.
import { DURATIONS, LABELS, ROUNDS, chime, endedMessage, format, initial, keyAction, next, progress, reset, restore, running, save, tick, timeLeft, toggle } from "./logic.js";

/** @typedef {import("./logic.js").Timer} Timer */
/** @typedef {import("./logic.js").Phase} Phase */

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const app = element("app");
const phaseText = element("phase");
const leftText = element("left");
const roundText = element("round");
const endsText = element("ends");
const doneText = element("done");
const arc = /** @type {SVGCircleElement} */ (/** @type {unknown} */ (element("arc")));
const dots = element("dots");
const toggleButton = element("toggle");
const message = element("message");
const CIRCUMFERENCE = 2 * Math.PI * 88;
const clock = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });

/** @type {Timer} */
let timer = initial();
/** @type {AudioContext | null} */
let audio = null;

/** Web Audio may only start after a key press or click: the first Start prepares it. */
function prepareAudio() {
  try {
    audio ??= new AudioContext();
    if (audio.state === "suspended") {
      void audio.resume();
    }
  } catch {
    audio = null;
  }
}

/** @param {Phase} ended */
function ring(ended) {
  if (!audio) {
    return;
  }
  const at = audio.currentTime + 0.05;
  for (const note of chime(ended)) {
    const tone = audio.createOscillator();
    const level = audio.createGain();
    tone.type = "sine";
    tone.frequency.value = note.frequency;
    level.gain.setValueAtTime(0.0001, at + note.at);
    level.gain.exponentialRampToValueAtTime(0.22, at + note.at + 0.02);
    level.gain.exponentialRampToValueAtTime(0.0001, at + note.at + note.length);
    tone.connect(level).connect(audio.destination);
    tone.start(at + note.at);
    tone.stop(at + note.at + note.length + 0.05);
  }
}

function persist() {
  littleJoan.storage.set("timer", save(timer)).catch(() => {
    // Without storage the timer lasts while the panel is open.
  });
}

/** @param {string} text */
function say(text) {
  message.textContent = text;
}

function render() {
  const now = Date.now();
  const left = timeLeft(timer, now);
  const on = running(timer);
  const paused = !on && timer.remaining < DURATIONS[timer.phase];
  app.dataset.phase = timer.phase;
  app.dataset.running = String(on);
  phaseText.textContent = LABELS[timer.phase];
  leftText.textContent = format(left);
  leftText.setAttribute("datetime", `PT${Math.ceil(left / 1000)}S`);
  roundText.textContent = timer.phase === "long" ? "Cycle done" : `Round ${timer.round} of ${ROUNDS}`;
  endsText.textContent = on && timer.endsAt !== null ? `Ends at ${clock.format(timer.endsAt)}` : paused ? "Paused" : "";
  doneText.textContent = timer.completed === 1 ? "1 session done" : `${timer.completed} sessions done`;
  arc.style.strokeDashoffset = String(CIRCUMFERENCE * progress(timer, now));
  toggleButton.textContent = on ? "Pause" : paused ? "Resume" : "Start";
  [...dots.children].forEach((dot, index) => {
    dot.classList.toggle("done", index < timer.round - (timer.phase === "focus" ? 1 : 0));
    dot.classList.toggle("now", index === timer.round - 1 && timer.phase === "focus");
  });
  document.title = on ? `${format(left)} · ${LABELS[timer.phase]}` : "Focus timer";
}

/** @param {Timer} changed */
function update(changed) {
  timer = changed;
  persist();
  render();
}

function onToggle() {
  prepareAudio();
  say("");
  update(toggle(timer, Date.now()));
}

function onReset() {
  say("");
  update(reset(timer));
}

function onSkip() {
  say(`Skipped to the ${LABELS[next(timer).phase].toLowerCase()}.`);
  update(next(timer));
}

setInterval(() => {
  const result = tick(timer, Date.now());
  if (result.ended) {
    ring(result.ended.phase);
    say(endedMessage(result.ended.phase, result.timer));
    update(result.timer);
  } else if (running(timer)) {
    render();
  }
}, 250);

toggleButton.addEventListener("click", onToggle);
element("reset").addEventListener("click", onReset);
element("skip").addEventListener("click", onSkip);
window.addEventListener(
  "keydown",
  (event) => {
    const action = keyAction(event);
    if (!action) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    if (action === "toggle") {
      onToggle();
    } else if (action === "reset") {
      onReset();
    } else if (action === "skip") {
      onSkip();
    } else {
      void littleJoan.close();
    }
  },
  true,
);

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  arc.style.strokeDasharray = String(CIRCUMFERENCE);
  try {
    const restored = restore(await littleJoan.storage.get("timer"), Date.now());
    timer = restored.timer;
    if (restored.ended) {
      say(`${LABELS[restored.ended.phase]} ended at ${clock.format(restored.ended.at)}, while this panel was closed.`);
      persist();
    }
  } catch {
    // Without storage the timer lasts while the panel is open.
  }
  render();
  toggleButton.focus();
}

void start();
