export type Request =
  | { kind: "HELO"; userId: string }
  | { kind: "PASS"; password: string }
  | { kind: "BALA" }
  | { kind: "WDRA"; amount: number }
  | { kind: "QUIT" };

export const RESP_AUTH_REQUIRE = "500 AUTH REQUIRE";
export const RESP_OK = "525 OK!";
export const RESP_ERROR = "401 ERROR!";
export const RESP_BYE = "BYE";

export function parseRequest(line: string): Request | null {
  const trimmed = line.trim();
  if (!trimmed) {
    return null;
  }

  const parts = trimmed.split(/\s+/);
  const cmd = parts[0].toUpperCase();

  switch (cmd) {
    case "HELO":
      return parts.length === 2 ? { kind: "HELO", userId: parts[1] } : null;
    case "PASS":
      return parts.length === 2 ? { kind: "PASS", password: parts[1] } : null;
    case "BALA":
      return parts.length === 1 ? { kind: "BALA" } : null;
    case "WDRA": {
      if (parts.length !== 2) {
        return null;
      }
      const amount = Number(parts[1]);
      if (!Number.isFinite(amount)) {
        return null;
      }
      return { kind: "WDRA", amount };
    }
    case "QUIT":
      return parts.length === 1 ? { kind: "QUIT" } : null;
    default:
      return null;
  }
}

export function amountResponse(value: number): string {
  return `AMNT:${value.toFixed(2)}`;
}
