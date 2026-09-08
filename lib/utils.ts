import { cn as cnBase } from "cn";

export function cn(...inputs: Parameters<typeof cnBase>): string {
  return cnBase(...inputs);
}
