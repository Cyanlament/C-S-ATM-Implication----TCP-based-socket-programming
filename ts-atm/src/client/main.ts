import { app, BrowserWindow, ipcMain } from "electron";
import net from "node:net";
import path from "node:path";

type Pending = {
  resolve: (value: string) => void;
  reject: (reason?: unknown) => void;
  timer: NodeJS.Timeout;
};

const DEFAULT_HOST = "172.19.153.48";
const DEFAULT_PORT = 2525;
const CONNECT_TIMEOUT_MS = 3000;
const RESPONSE_TIMEOUT_MS = 5000;

function friendlyNetError(error: unknown): Error {
  const err = error as NodeJS.ErrnoException;
  switch (err.code) {
    case "ECONNREFUSED":
      return new Error("Connection refused. Start the server and check the port.");
    case "ETIMEDOUT":
      return new Error("Connection timeout. Check the LAN IP, firewall, and Wi-Fi network.");
    case "EHOSTUNREACH":
    case "ENETUNREACH":
      return new Error("Host unreachable. Check that both computers are on the same LAN.");
    case "ECONNRESET":
      return new Error("Connection reset by server.");
    case "EADDRNOTAVAIL":
      return new Error("Address is not available on this machine.");
    default:
      return new Error(err.code ? `Network error (${err.code})` : "Network error");
  }
}

class AtmTcpClient {
  private socket: net.Socket | null = null;
  private buffer = "";
  private pending: Pending[] = [];

  private rejectAll(error: Error): void {
    while (this.pending.length) {
      const next = this.pending.shift();
      if (next) {
        clearTimeout(next.timer);
        next.reject(error);
      }
    }
  }

  async connect(host: string, port: number): Promise<void> {
    if (this.socket && !this.socket.destroyed) {
      return;
    }

    await new Promise<void>((resolve, reject) => {
      let settled = false;
      const socket = net.createConnection({ host, port }, () => {
        settled = true;
        socket.setTimeout(0);
        this.socket = socket;
        resolve();
      });

      socket.setEncoding("utf-8");
      socket.setTimeout(CONNECT_TIMEOUT_MS, () => {
        socket.destroy(Object.assign(new Error("connect timeout"), { code: "ETIMEDOUT" }));
      });

      socket.on("data", (chunk: string) => {
        this.buffer += chunk;

        while (true) {
          const idx = this.buffer.indexOf("\n");
          if (idx < 0) {
            break;
          }

          const line = this.buffer.slice(0, idx).trim();
          this.buffer = this.buffer.slice(idx + 1);
          if (!line) {
            continue;
          }

          const next = this.pending.shift();
          if (next) {
            clearTimeout(next.timer);
            next.resolve(line);
          }
        }
      });

      socket.on("error", (error: Error) => {
        const friendly = friendlyNetError(error);
        this.rejectAll(friendly);

        if (!settled) {
          settled = true;
          reject(friendly);
        }
      });

      socket.on("close", () => {
        this.rejectAll(new Error("Connection closed."));
        this.socket = null;
      });
    });
  }

  async sendCommand(command: string): Promise<string> {
    if (!this.socket || this.socket.destroyed) {
      throw new Error("Not connected");
    }

    const socket = this.socket;

    return await new Promise<string>((resolve, reject) => {
      const timer = setTimeout(() => {
        const idx = this.pending.findIndex((p) => p.resolve === resolve);
        if (idx >= 0) {
          this.pending.splice(idx, 1);
        }
        reject(new Error("Response timeout. Check the server state and firewall."));
      }, RESPONSE_TIMEOUT_MS);

      this.pending.push({ resolve, reject, timer });
      socket.write(`${command.trim()}\n`, "utf-8", (err) => {
        if (err) {
          const idx = this.pending.findIndex((p) => p.resolve === resolve);
          if (idx >= 0) {
            const pending = this.pending.splice(idx, 1)[0];
            clearTimeout(pending.timer);
          }
          reject(friendlyNetError(err));
        }
      });
    });
  }

  async disconnect(): Promise<void> {
    if (!this.socket || this.socket.destroyed) {
      this.socket = null;
      return;
    }

    await new Promise<void>((resolve) => {
      this.socket?.end(() => resolve());
      setTimeout(resolve, 300);
    });

    this.socket?.destroy();
    this.socket = null;
    this.rejectAll(new Error("Disconnected."));
    this.buffer = "";
  }
}

const atmClient = new AtmTcpClient();

function resolveClientDefaults(): { host: string; port: number } {
  const extra = process.argv
    .slice(2)
    .filter((arg) => !arg.startsWith("--"))
    .filter((arg) => !arg.endsWith(".js") && !arg.endsWith(".exe"));

  const host = extra[0] ?? process.env.ATM_HOST ?? DEFAULT_HOST;
  const port = Number(extra[1] ?? process.env.ATM_PORT ?? String(DEFAULT_PORT));

  return {
    host,
    port: Number.isFinite(port) && port > 0 && port <= 65535 ? port : DEFAULT_PORT,
  };
}

function createWindow(): void {
  const win = new BrowserWindow({
    width: 980,
    height: 720,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  win.loadFile(path.resolve(__dirname, "../../../src/client/renderer/index.html"));
}

ipcMain.handle("atm-connect", async (_event, payload: { host: string; port: number }) => {
  try {
    await atmClient.connect(payload.host, payload.port);
    return { ok: true };
  } catch (error) {
    return { ok: false, error: (error as Error).message };
  }
});

ipcMain.handle("atm-defaults", async () => resolveClientDefaults());

ipcMain.handle("atm-send", async (_event, payload: { command: string }) => {
  try {
    const response = await atmClient.sendCommand(payload.command);
    if (response === "BYE") {
      await atmClient.disconnect();
    }
    return { ok: true, response };
  } catch (error) {
    return { ok: false, error: (error as Error).message };
  }
});

ipcMain.handle("atm-disconnect", async () => {
  try {
    await atmClient.disconnect();
    return { ok: true };
  } catch (error) {
    return { ok: false, error: (error as Error).message };
  }
});

app.whenReady().then(() => {
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
