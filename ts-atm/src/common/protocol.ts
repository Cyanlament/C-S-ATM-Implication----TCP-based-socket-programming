export type Request =
  | { kind: "HELO"; userId: string }
  | { kind: "PASS"; password: string }
  | { kind: "BALA" }
  | { kind: "WDRA"; amount: number }
  | { kind: "BYE" };

export const RESP_AUTH_REQUIRED = "500 AUTH REQUIRED!";
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
      return parts[1] ? { kind: "HELO", userId: parts[1] } : null;
    case "PASS":
      return parts[1] ? { kind: "PASS", password: parts[1] } : null;
    case "BALA":
      return { kind: "BALA" };
    case "WDRA": {
      if (!parts[1]) {
        return null;
      }
      const amount = Number(parts[1]);
      if (!Number.isFinite(amount)) {
        return null;
      }
      return { kind: "WDRA", amount };
    }
    case "BYE":
      return { kind: "BYE" };
    default:
      return null;
  }
}

export function amountResponse(value: number): string {
  return `AMNT:${value.toFixed(2)}`;
}
