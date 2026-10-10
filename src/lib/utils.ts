import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/** Turns an error thrown by a Tauri command (string or Error-like) into display text. */
export function errorMessage(e: unknown): string {
	if (typeof e === "string") return e;
	if (e && typeof e === "object" && "message" in e) return String(e.message);
	return String(e);
}
