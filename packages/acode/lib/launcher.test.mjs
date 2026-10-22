import { describe, expect, it } from "vitest";
import {
  binaryName,
  detectLibc,
  missingPlatformMessage,
  platformKey,
  platformPackage,
  sha256,
  SUPPORTED,
} from "./launcher.mjs";

describe("launcher", () => {
  it("mapea plataforma, arquitectura y libc", () => {
    expect(platformKey("linux", "x64", "musl")).toBe("linux-x64-musl");
    expect(platformKey("linux", "arm64", "gnu")).toBe("linux-arm64-gnu");
    expect(platformKey("win32", "arm64", null)).toBe("win32-arm64");
    expect(platformKey("darwin", "x64", null)).toBe("darwin-x64");
    expect(["gnu", "musl", null]).toContain(detectLibc());
  });

  it("resuelve el paquete opcional por producto", () => {
    expect(platformPackage("acode", "linux-x64-gnu")).toBe("@ancleto/code-linux-x64-gnu");
    expect(platformPackage("acmd", "darwin-arm64")).toBe("@ancleto/cmd-darwin-arm64");
  });

  it("calcula checksums y nombres de binario", () => {
    expect(sha256(Buffer.from("abc"))).toBe(
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    expect(binaryName("acode", "win32")).toBe("acode.exe");
    expect(binaryName("acode", "linux")).toBe("acode");
  });

  it("explica qué hacer cuando falta la plataforma", () => {
    const message = missingPlatformMessage("acode", "freebsd-x64");
    expect(message).toContain("freebsd-x64");
    expect(message).toContain("Instalación manual");
    expect(message).toContain(SUPPORTED.acode[0]);
  });
});
