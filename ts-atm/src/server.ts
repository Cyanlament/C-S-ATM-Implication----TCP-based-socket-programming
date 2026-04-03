import fs from "node:fs";
import net, { Socket } from "node:net";
import path from "node:path";
import {
  amountResponse,
  parseRequest,
  RESP_AUTH_REQUIRED,
  RESP_BYE,
  RESP_ERROR,
  RESP_OK,
  Request,
} from "./common/protocol";

type Account = {
  password: string;
  balance: number;
};

type AccountsDb = Record<string, Account>;

type Session = {
  currentUserId: string | null;
  authenticated: boolean;
};

const ROOT = path.resolve(__dirname, "../..");
const DATA_FILE = path.join(ROOT, "data", "accounts.json");
const LOG_DIR = path.join(ROOT, "logs");
const PORT = Number(process.env.ATM_PORT ?? "2525");

function ensureLogDir(): void {
  fs.mkdirSync(LOG_DIR, { recursive: true });
}

function appendLog(filename: string, message: string): void {
  ensureLogDir();
  const stamp = Math.floor(Date.now() / 1000);
  // 日志像黑匣子，关键时刻能保命。
  fs.appendFileSync(path.join(LOG_DIR, filename), `[${stamp}] ${message}\n`, "utf-8");
}

function loadAccounts(): AccountsDb {
  const raw = fs.readFileSync(DATA_FILE, "utf-8");
  return JSON.parse(raw) as AccountsDb;
}

function saveAccounts(db: AccountsDb): void {
  fs.writeFileSync(DATA_FILE, JSON.stringify(db, null, 2), "utf-8");
}

let accounts = loadAccounts();

function sendLine(socket: Socket, line: string): void {
  socket.write(`${line}\n`);
}

function handleRequest(
  request: Request,
  socket: Socket,
  session: Session,
  peer: string,
): void {
  // 协议闯关从这里开始，别插队。
  if (request.kind === "HELO") {
    if (accounts[request.userId]) {
      session.currentUserId = request.userId;
      session.authenticated = false;
      sendLine(socket, RESP_AUTH_REQUIRED);
      return;
    }
    appendLog("exception.log", `${peer} unknown user id: ${request.userId}`);
    sendLine(socket, RESP_ERROR);
    return;
  }

  if (request.kind === "PASS") {
    if (!session.currentUserId) {
      appendLog("exception.log", `${peer} PASS before HELO`);
      sendLine(socket, RESP_ERROR);
      return;
    }

    const acc = accounts[session.currentUserId];
    if (acc && acc.password === request.password) {
      session.authenticated = true;
      sendLine(socket, RESP_OK);
      return;
    }

    appendLog("exception.log", `${peer} password failed for ${session.currentUserId}`);
    sendLine(socket, RESP_ERROR);
    return;
  }

  if (request.kind === "BALA") {
    if (!session.currentUserId) {
      sendLine(socket, RESP_ERROR);
      return;
    }
    if (!session.authenticated) {
      sendLine(socket, RESP_AUTH_REQUIRED);
      return;
    }

    const acc = accounts[session.currentUserId];
    if (!acc) {
      appendLog("exception.log", `${peer} user missing in BALA: ${session.currentUserId}`);
      sendLine(socket, RESP_ERROR);
      return;
    }

    sendLine(socket, amountResponse(acc.balance));
    return;
  }

  if (request.kind === "WDRA") {
    if (request.amount <= 0) {
      appendLog("exception.log", `${peer} invalid withdraw amount: ${request.amount}`);
      sendLine(socket, RESP_ERROR);
      return;
    }

    if (!session.currentUserId) {
      sendLine(socket, RESP_ERROR);
      return;
    }
    if (!session.authenticated) {
      sendLine(socket, RESP_AUTH_REQUIRED);
      return;
    }

    const acc = accounts[session.currentUserId];
    if (!acc) {
      appendLog("exception.log", `${peer} user missing in WDRA: ${session.currentUserId}`);
      sendLine(socket, RESP_ERROR);
      return;
    }

    if (acc.balance >= request.amount) {
      const before = acc.balance;
      acc.balance -= request.amount;
      // 钱包更新写回文件，账目要清清楚楚。
      saveAccounts(accounts);
      appendLog(
        "withdraw.log",
        `${peer} user=${session.currentUserId} withdraw=${request.amount.toFixed(2)} before=${before.toFixed(2)} after=${acc.balance.toFixed(2)}`,
      );
      sendLine(socket, RESP_OK);
      return;
    }

    appendLog(
      "exception.log",
      `${peer} insufficient funds user=${session.currentUserId} request=${request.amount.toFixed(2)} balance=${acc.balance.toFixed(2)}`,
    );
    sendLine(socket, RESP_ERROR);
    return;
  }

  if (request.kind === "BYE") {
    sendLine(socket, RESP_BYE);
    socket.end();
  }
}

function attachLineHandler(socket: Socket, peer: string): void {
  const session: Session = {
    currentUserId: null,
    authenticated: false,
  };

  let buffer = "";

  socket.on("data", (chunk: Buffer) => {
    buffer += chunk.toString("utf-8");

    while (true) {
      const idx = buffer.indexOf("\n");
      if (idx < 0) {
        break;
      }
      const line = buffer.slice(0, idx).trim();
      buffer = buffer.slice(idx + 1);

      if (!line) {
        continue;
      }

      const request = parseRequest(line);
      if (!request) {
        // 不认识的命令先记一笔，再温柔地回 401。
        appendLog("exception.log", `${peer} invalid request: ${line}`);
        sendLine(socket, RESP_ERROR);
        continue;
      }

      try {
        handleRequest(request, socket, session, peer);
      } catch (error) {
        appendLog("exception.log", `${peer} server exception: ${(error as Error).message}`);
        sendLine(socket, RESP_ERROR);
      }
    }
  });

  socket.on("close", () => {
    appendLog("server.log", `client closed: ${peer}`);
  });

  socket.on("error", (error: Error) => {
    appendLog("exception.log", `${peer} socket error: ${error.message}`);
  });
}

const server = net.createServer((socket) => {
  const peer = `${socket.remoteAddress ?? "unknown"}:${socket.remotePort ?? 0}`;
  appendLog("server.log", `client connected: ${peer}`);
  attachLineHandler(socket, peer);
});

server.on("error", (error: Error) => {
  appendLog("exception.log", `server error: ${error.message}`);
});

server.listen(PORT, "0.0.0.0", () => {
  appendLog("server.log", `server listening on 0.0.0.0:${PORT}`);
  // Keep one startup line in console for easier debugging.
  console.log(`ATM server started on 0.0.0.0:${PORT}`);
});
