// Network presets. r-share has no public testnet; custom nodes use the same shape.

export interface NetworkProfile {
  name: "devnet" | "custom";
  node: string;
  shard: string;
  phloLimit: number;
  phloPrice: number;
  manifestPath: string;
}

/** The configured node, without trusting the untyped Vite env index signature. */
export function configuredNode(): string | undefined {
  const value: unknown = import.meta.env.VITE_RSHARE_NODE;
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

export const DEVNET: NetworkProfile = {
  name: "devnet",
  node: configuredNode() ?? "http://localhost:40403",
  shard: "/root",
  phloLimit: 4_000_000,
  phloPrice: 1,
  manifestPath: "/devnet.json",
};

export function networkForNode(node: string): NetworkProfile {
  const isDevnet = node.includes("localhost") || node.includes("127.0.0.1");
  return isDevnet ? { ...DEVNET, node } : { ...DEVNET, name: "custom", node };
}
