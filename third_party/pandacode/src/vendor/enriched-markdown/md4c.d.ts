// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 PandaCode contributors
// Source: https://github.com/aicodeking/pandacode/blob/e9f89891fd23518848765aef8219dbf39feac6e0/src/vendor/enriched-markdown/md4c.d.ts
// Modified by AncletoCode: Ninguna

/** md4c compiled to WebAssembly, from react-native-enriched-markdown's web build. */
declare const createMd4cModule: () => Promise<{ cwrap: (name: string, returns: string, args: string[]) => (...args: (string | number)[]) => string }>;
export default createMd4cModule;
