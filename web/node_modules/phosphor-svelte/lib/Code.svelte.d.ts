import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodeIcon` instead.
 *
 * @example
 * ```svelte
 * <Code color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Code: Component<IconComponentProps, {}, "">;
type Code = ReturnType<typeof Code>;
export default Code;
