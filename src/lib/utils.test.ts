import { describe, expect, it } from "vitest";
import { cn, errorMessage } from "#lib/utils.js";

describe("cn", () => {
	it("joins class names and drops falsy values", () => {
		expect(cn("p-2", false, undefined, "text-sm")).toBe("p-2 text-sm");
	});

	it("lets later Tailwind utilities override earlier conflicting ones", () => {
		expect(cn("p-2 text-sm", "p-4")).toBe("text-sm p-4");
	});
});

describe("errorMessage", () => {
	it("returns string errors unchanged", () => {
		expect(errorMessage("bad sequence")).toBe("bad sequence");
	});

	it("uses the message of Error-like objects", () => {
		expect(errorMessage({ message: "parse failed" })).toBe("parse failed");
		expect(errorMessage(new Error("boom"))).toBe("boom");
	});

	it("falls back to String() for anything else", () => {
		expect(errorMessage(42)).toBe("42");
		expect(errorMessage(null)).toBe("null");
	});
});
