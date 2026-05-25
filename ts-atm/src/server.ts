import fs from "node:fs";
import net, { Socket } from "node:net";
import path from "node:path";
import {
  amountResponse,
  parseRequest,
  RESP_AUTH_REQUIRE,
  RESP_BYE,
  RESP_ERROR,
  RESP_OK,
  Request,
} from "./common/protocol";

type SessionState = "INIT" | "AUTH_REQUIRED" | "LOGGED_IN";

type Session = {
  state: SessionState;
  currentUserId: string | null;
};

type UserPasswords = Record<string, string>;
type UserBalances = Record<string, number>;

const ROOT = path.resolve(__dirname, "../..");
const USERS_FILE = path.join(ROOT, "users.txt");
const BALANCES_FILE = path.join(ROOT, "balances.txt");
const LOG_DIR = path.join(ROOT, "logs");

function resolvePort(raw: string | undefined, fallback = 2525): number {
  if (!raw) {
    return fallback;
  }

  if (!/^\d+$/.test(raw.trim())) {
    console.error(`Invalid port ${raw}; using default port ${fallback}.`);
    return fallback;
  }

  const port = Number.parseInt(raw.trim(), 10);
  if (!Number.isFinite(port) || port <= 0 || port > 65535) {
    console.error(`Port out of range ${raw}; using default port ${fallback}.`);
    return fallback;
  }

  return port;
}

function ensureLogDir(): void {
  fs.mkdirSync(LOG_DIR, { recursive: true });
}

function appendLog(filename: string, message: string): void {
  ensureLogDir();
  const stamp = Math.floor(Date.now() / 1000);
  fs.appendFileSync(path.join(LOG_DIR, filename), `[${stamp}] ${message}\n`, "utf-8");
}

function loadUsers(): UserPasswords {
  const users: UserPasswords = {};
  const raw = fs.readFileSync(USERS_FILE, "utf-8");

  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    const parts = trimmed.split(/\s+/);
    if (parts.length === 2) {
      users[parts[0]] = parts[1];
    }
  }

  return users;
}

function loadBalances(): UserBalances {
  const balances: UserBalances = {};
  const raw = fs.readFileSync(BALANCES_FILE, "utf-8");

  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    const parts = trimmed.split(/\s+/);
    if (parts.length !== 2) {
      continue;
    }

    const amount = Number(parts[1]);
    if (Number.isFinite(amount)) {
      balances[parts[0]] = amount;
    }
  }

  return balances;
}

function saveBalances(balances: UserBalances): void {
  const lines = Object.keys(balances)
    .sort()
    .map((userId) => `${userId} ${balances[userId].toFixed(2)}`);
  fs.writeFileSync(BALANCES_FILE, `${lines.join("\n")}\n`, "utf-8");
}

const userPasswords = loadUsers();
const userBalances = loadBalances();

function sendLine(socket: Socket, line: string): void {
  socket.write(`${line}\n`);
}

function handleRequest(
  request: Request,
  socket: Socket,
  session: Session,
  peer: string,
): void {
  switch (request.kind) {
    case "HELO":
      if (userPasswords[request.userId]) {
        session.state = "AUTH_REQUIRED";
        session.currentUserId = request.userId;
        sendLine(socket, RESP_AUTH_REQUIRE);
      } else {
        session.state = "INIT";
        session.currentUserId = null;
        appendLog("exception.log", `${peer} unknown user id: ${request.userId}`);
        sendLine(socket, RESP_ERROR);
      }
      return;

    case "PASS":
      if (session.state !== "AUTH_REQUIRED" || !session.currentUserId) {
        appendLog("exception.log", `${peer} PASS in invalid state`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      if (userPasswords[session.currentUserId] === request.password) {
        session.state = "LOGGED_IN";
        sendLine(socket, RESP_OK);
      } else {
        appendLog("exception.log", `${peer} password failed for ${session.currentUserId}`);
        sendLine(socket, RESP_ERROR);
      }
      return;

    case "BALA":
      if (session.state !== "LOGGED_IN" || !session.currentUserId) {
        appendLog("exception.log", `${peer} BALA in invalid state`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      if (!(session.currentUserId in userBalances)) {
        appendLog("exception.log", `${peer} missing balance for ${session.currentUserId}`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      sendLine(socket, amountResponse(userBalances[session.currentUserId]));
      return;

    case "WDRA":
      if (session.state !== "LOGGED_IN" || !session.currentUserId) {
        appendLog("exception.log", `${peer} WDRA in invalid state`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      if (!Number.isFinite(request.amount) || request.amount <= 0) {
        appendLog("exception.log", `${peer} invalid withdraw amount: ${request.amount}`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      if (!(session.currentUserId in userBalances)) {
        appendLog("exception.log", `${peer} missing balance for ${session.currentUserId}`);
        sendLine(socket, RESP_ERROR);
        return;
      }

      if (userBalances[session.currentUserId] >= request.amount) {
        const before = userBalances[session.currentUserId];
        userBalances[session.currentUserId] -= request.amount;
        saveBalances(userBalances);
        appendLog(
          "withdraw.log",
          `${peer} user=${session.currentUserId} withdraw=${request.amount.toFixed(2)} before=${before.toFixed(2)} after=${userBalances[session.currentUserId].toFixed(2)}`,
        );
        sendLine(socket, RESP_OK);
      } else {
        appendLog(
          "exception.log",
          `${peer} insufficient funds user=${session.currentUserId} request=${request.amount.toFixed(2)} balance=${userBalances[session.currentUserId].toFixed(2)}`,
        );
        sendLine(socket, RESP_ERROR);
      }
      return;

    case "QUIT":
      session.state = "INIT";
      session.currentUserId = null;
      sendLine(socket, RESP_BYE);
      socket.end();
      return;
  }
}

function attachLineHandler(socket: Socket, peer: string): void {
  const session: Session = {
    state: "INIT",
    currentUserId: null,
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

const port = resolvePort(process.argv[2] ?? process.env.ATM_PORT);
const server = net.createServer((socket) => {
  const peer = `${socket.remoteAddress ?? "unknown"}:${socket.remotePort ?? 0}`;
  appendLog("server.log", `client connected: ${peer}`);
  attachLineHandler(socket, peer);
});

server.on("error", (error: Error) => {
  appendLog("exception.log", `server error: ${error.message}`);
});

server.listen(port, "0.0.0.0", () => {
  appendLog("server.log", `server listening on 0.0.0.0:${port}`);
  console.log(`ATM server started on 0.0.0.0:${port}`);
});
