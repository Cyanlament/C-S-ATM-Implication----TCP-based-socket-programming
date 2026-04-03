const hostInput = document.getElementById("host") as HTMLInputElement;
const portInput = document.getElementById("port") as HTMLInputElement;
const userIdInput = document.getElementById("userId") as HTMLInputElement;
const passwordInput = document.getElementById("password") as HTMLInputElement;
const amountInput = document.getElementById("amount") as HTMLInputElement;
const statusBox = document.getElementById("status") as HTMLDivElement;
const logBox = document.getElementById("logBox") as HTMLTextAreaElement;
const autoFlowBtn = document.getElementById("autoFlowBtn") as HTMLButtonElement;
const clearLogBtn = document.getElementById("clearLogBtn") as HTMLButtonElement;

const connectBtn = document.getElementById("connectBtn") as HTMLButtonElement;
const disconnectBtn = document.getElementById("disconnectBtn") as HTMLButtonElement;
const heloBtn = document.getElementById("heloBtn") as HTMLButtonElement;
const passBtn = document.getElementById("passBtn") as HTMLButtonElement;
const balaBtn = document.getElementById("balaBtn") as HTMLButtonElement;
const wdraBtn = document.getElementById("wdraBtn") as HTMLButtonElement;
const byeBtn = document.getElementById("byeBtn") as HTMLButtonElement;

function setStatus(message: string, state: "idle" | "ok" | "error" = "idle"): void {
  statusBox.textContent = `状态：${message}`;
  statusBox.dataset.state = state;
}

function logLine(line: string): void {
  logBox.value += `${line}\n`;
  logBox.scrollTop = logBox.scrollHeight;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function sendCommand(command: string): Promise<void> {
  logLine(`>> ${command}`);
  const result = await window.atmApi.send(command);
  if (!result.ok) {
    setStatus(`请求失败：${result.error ?? "unknown"}`, "error");
    logLine(`[ERR] ${result.error ?? "unknown"}`);
    return;
  }

  const response = result.response ?? "";
  logLine(`<< ${response}`);

  if (response.startsWith("401")) {
    setStatus(response, "error");
    return;
  }

  if (response.startsWith("525") || response.startsWith("AMNT") || response === "BYE") {
    setStatus(response, "ok");
    return;
  }

  setStatus(response, "idle");
}

connectBtn.addEventListener("click", async () => {
  const host = hostInput.value.trim();
  const port = Number(portInput.value.trim() || "2525");

  const result = await window.atmApi.connect(host, port);
  if (result.ok) {
    setStatus(`连接成功 ${host}:${port}`, "ok");
    logLine(`[SYS] connected ${host}:${port}`);
  } else {
    setStatus(`连接失败 - ${result.error ?? "unknown"}`, "error");
    logLine(`[ERR] connect failed: ${result.error ?? "unknown"}`);
  }
});

disconnectBtn.addEventListener("click", async () => {
  const result = await window.atmApi.disconnect();
  if (result.ok) {
    setStatus("已断开连接", "idle");
    logLine("[SYS] disconnected");
  } else {
    setStatus(`断开失败 - ${result.error ?? "unknown"}`, "error");
    logLine(`[ERR] disconnect failed: ${result.error ?? "unknown"}`);
  }
});

heloBtn.addEventListener("click", async () => {
  await sendCommand(`HELO ${userIdInput.value.trim()}`);
});

passBtn.addEventListener("click", async () => {
  await sendCommand(`PASS ${passwordInput.value.trim()}`);
});

balaBtn.addEventListener("click", async () => {
  await sendCommand("BALA");
});

wdraBtn.addEventListener("click", async () => {
  await sendCommand(`WDRA ${amountInput.value.trim()}`);
});

byeBtn.addEventListener("click", async () => {
  await sendCommand("BYE");
});

autoFlowBtn.addEventListener("click", async () => {
  // 一键演示标准流程，便于课堂展示。
  const host = hostInput.value.trim();
  const port = Number(portInput.value.trim() || "2525");
  const connectResult = await window.atmApi.connect(host, port);
  if (!connectResult.ok) {
    setStatus(`自动流程连接失败 - ${connectResult.error ?? "unknown"}`, "error");
    return;
  }

  setStatus("自动流程执行中", "ok");
  const sequence = [
    `HELO ${userIdInput.value.trim()}`,
    `PASS ${passwordInput.value.trim()}`,
    "BALA",
    `WDRA ${amountInput.value.trim()}`,
    "BALA",
  ];

  for (const command of sequence) {
    await sendCommand(command);
    await delay(180);
  }
});

clearLogBtn.addEventListener("click", () => {
  logBox.value = "";
  setStatus("日志已清空", "idle");
});
