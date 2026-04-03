export {};

declare global {
  interface Window {
    atmApi: {
      connect: (host: string, port: number) => Promise<{ ok: boolean; error?: string }>;
      send: (command: string) => Promise<{ ok: boolean; response?: string; error?: string }>;
      disconnect: () => Promise<{ ok: boolean; error?: string }>;
    };
  }
}
