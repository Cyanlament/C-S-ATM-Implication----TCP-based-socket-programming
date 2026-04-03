import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("atmApi", {
  connect: (host: string, port: number) =>
    ipcRenderer.invoke("atm-connect", { host, port }) as Promise<{ ok: boolean; error?: string }>,
  send: (command: string) =>
    ipcRenderer.invoke("atm-send", { command }) as Promise<{ ok: boolean; response?: string; error?: string }>,
  disconnect: () =>
    ipcRenderer.invoke("atm-disconnect") as Promise<{ ok: boolean; error?: string }>,
});
