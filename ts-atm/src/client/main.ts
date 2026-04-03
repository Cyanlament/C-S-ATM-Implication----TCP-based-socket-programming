import { app, BrowserWindow, ipcMain } from "electron";
import net from "node:net";
import path from "node:path";

type Pending = {
  resolve: (value: string) => void;
  reject: (reason?: unknown) => void;
};

class AtmTcpClient {
  private socket: net.Socket | null = null;
  private buffer = "";
  private pending: Pending[] = [];

  async connect(host: string, port: number): Promise<void> {
    if (this.socket && !this.socket.destroyed) {
      return;
    }

    await new Promise<void>((resolve, reject) => {
      const socket = net.createConnection({ host, port }, () => {
        this.socket = socket;
        resolve();
      });

      socket.setEncoding("utf-8");

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
            next.resolve(line);
          }
        }
      });

      socket.on("error", (error: Error) => {
        while (this.pending.length) {
          const next = this.pending.shift();
          next?.reject(error);
        }

        if (!this.socket) {
          reject(error);
        }
      });

      socket.on("close", () => {
        while (this.pending.length) {
          const next = this.pending.shift();
          next?.reject(new Error("socket closed"));
        }
        this.socket = null;
      });

      socket.once("error", reject);
      socket.once("connect", () => {
        socket.off("error", reject);
      });
    });
  }

  async sendCommand(command: string): Promise<string> {
    if (!this.socket || this.socket.destroyed) {
      throw new Error("Not connected");
    }

    const socket = this.socket;

    return await new Promise<string>((resolve, reject) => {
      this.pending.push({ resolve, reject });
      socket.write(`${command.trim()}\n`, "utf-8", (err) => {
        if (err) {
          const idx = this.pending.findIndex((p) => p.resolve === resolve);
          if (idx >= 0) {
            this.pending.splice(idx, 1);
          }
          reject(err);
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
    this.pending = [];
    this.buffer = "";
  }
}

const atmClient = new AtmTcpClient();

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
