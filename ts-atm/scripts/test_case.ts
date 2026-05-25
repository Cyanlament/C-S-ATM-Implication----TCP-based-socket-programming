import net from "node:net";

const DEFAULT_HOST = "172.19.153.48";
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
    default:
      return new Error(err.code ? `Network error (${err.code})` : "Network error");
  }
}

class LineClient {
  private socket: net.Socket;
  private buffer = "";
  private pending: Array<{
    resolve: (line: string) => void;
    reject: (error: Error) => void;
    timer: NodeJS.Timeout;
  }> = [];

  private constructor(socket: net.Socket) {
    this.socket = socket;
    this.socket.setEncoding("utf-8");

    this.socket.on("data", (chunk: string) => {
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
        const p = this.pending.shift();
        if (p) {
          clearTimeout(p.timer);
          p.resolve(line);
        }
      }
    });

    this.socket.on("error", (error) => {
      while (this.pending.length > 0) {
        const pending = this.pending.shift();
        if (pending) {
          clearTimeout(pending.timer);
          pending.reject(friendlyNetError(error));
        }
      }
    });

    this.socket.on("close", () => {
      while (this.pending.length > 0) {
        const pending = this.pending.shift();
        if (pending) {
          clearTimeout(pending.timer);
          pending.reject(new Error("Connection closed."));
        }
      }
    });
  }

  static connect(host: string, port: number): Promise<LineClient> {
    return new Promise((resolve, reject) => {
      let settled = false;
      const socket = net.createConnection({ host, port }, () => {
        settled = true;
        socket.setTimeout(0);
        resolve(new LineClient(socket));
      });
      socket.setTimeout(CONNECT_TIMEOUT_MS, () => {
        socket.destroy(Object.assign(new Error("connect timeout"), { code: "ETIMEDOUT" }));
      });
      socket.once("error", (error) => {
        if (!settled) {
          settled = true;
          reject(friendlyNetError(error));
        }
      });
    });
  }

  send(command: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        const idx = this.pending.findIndex((p) => p.resolve === resolve);
        if (idx >= 0) {
          this.pending.splice(idx, 1);
        }
        reject(new Error("Response timeout. Check the server state and firewall."));
      }, RESPONSE_TIMEOUT_MS);

      this.pending.push({ resolve, reject, timer });
      this.socket.write(`${command}\n`, "utf-8", (err) => {
        if (err) {
          reject(friendlyNetError(err));
        }
      });
    });
  }

  close(): void {
    this.socket.end();
    this.socket.destroy();
  }
}

async function runCase(
  host: string,
  port: number,
  user: string,
  pass: string,
  amount: number,
): Promise<void> {
  console.log(`[CASE1] normal flow on ${host}:${port}`);
  const c1 = await LineClient.connect(host, port);
  for (const cmd of [
    `HELO ${user}`,
    `PASS ${pass}`,
    "BALA",
    `WDRA ${amount}`,
    "BALA",
    "QUIT",
  ]) {
    const resp = await c1.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c1.close();

  console.log("\n[CASE2] wrong password");
  const c2 = await LineClient.connect(host, port);
  for (const cmd of [`HELO ${user}`, "PASS wrong_password", "QUIT"]) {
    const resp = await c2.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c2.close();

  console.log("\n[CASE3] insufficient funds");
  const c3 = await LineClient.connect(host, port);
  for (const cmd of [`HELO ${user}`, `PASS ${pass}`, "WDRA 9999999", "QUIT"]) {
    const resp = await c3.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c3.close();
}

async function main(): Promise<void> {
  const host = process.argv[2] ?? DEFAULT_HOST;
  const port = Number(process.argv[3] ?? "2525");
  const user = process.argv[4] ?? "100001";
  const pass = process.argv[5] ?? "1234";
  const amount = Number(process.argv[6] ?? "100");

  await runCase(host, port, user, pass, amount);
}

main().catch((error) => {
  console.error(`test case failed: ${(error as Error).message}`);
  process.exit(1);
});
