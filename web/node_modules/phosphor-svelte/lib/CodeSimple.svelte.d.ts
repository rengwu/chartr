import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodeSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <CodeSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodeSimple: Component<IconComponentProps, {}, "">;
type CodeSimple = ReturnType<typeof CodeSimple>;
export default CodeSimple;
