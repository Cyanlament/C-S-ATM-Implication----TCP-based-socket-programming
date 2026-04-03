import net from "node:net";

class LineClient {
  private socket: net.Socket;
  private buffer = "";
  private pending: Array<{ resolve: (line: string) => void; reject: (error: Error) => void }> = [];

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
        p?.resolve(line);
      }
    });

    this.socket.on("error", (error) => {
      while (this.pending.length > 0) {
        this.pending.shift()?.reject(error);
      }
    });

    this.socket.on("close", () => {
      while (this.pending.length > 0) {
        this.pending.shift()?.reject(new Error("socket closed"));
      }
    });
  }

  static connect(host: string, port: number): Promise<LineClient> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection({ host, port }, () => {
        resolve(new LineClient(socket));
      });
      socket.once("error", reject);
    });
  }

  send(command: string): Promise<string> {
    return new Promise((resolve, reject) => {
      this.pending.push({ resolve, reject });
      this.socket.write(`${command}\n`, "utf-8", (err) => {
        if (err) {
          reject(err);
        }
      });
    });
  }

  close(): void {
    this.socket.end();
    this.socket.destroy();
  }
}

async function runCase(host: string, port: number, user: string, pass: string, amount: number): Promise<void> {
  console.log(`[CASE1] normal flow on ${host}:${port}`);
  const c1 = await LineClient.connect(host, port);
  for (const cmd of [
    `HELO ${user}`,
    `PASS ${pass}`,
    "BALA",
    `WDRA ${amount}`,
    "BALA",
    "BYE",
  ]) {
    const resp = await c1.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c1.close();

  console.log("\n[CASE2] wrong password");
  const c2 = await LineClient.connect(host, port);
  for (const cmd of [`HELO ${user}`, "PASS wrong_password", "BYE"]) {
    const resp = await c2.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c2.close();

  console.log("\n[CASE3] insufficient funds");
  const c3 = await LineClient.connect(host, port);
  for (const cmd of [`HELO ${user}`, `PASS ${pass}`, "WDRA 9999999", "BYE"]) {
    const resp = await c3.send(cmd);
    console.log(`>> ${cmd}`);
    console.log(`<< ${resp}`);
  }
  c3.close();
}

async function main(): Promise<void> {
  const host = process.argv[2] ?? "127.0.0.1";
  const port = Number(process.argv[3] ?? "2525");
  const user = process.argv[4] ?? "10001";
  const pass = process.argv[5] ?? "111111";
  const amount = Number(process.argv[6] ?? "100");

  await runCase(host, port, user, pass, amount);
}

main().catch((error) => {
  console.error(`test case failed: ${(error as Error).message}`);
  process.exit(1);
});
