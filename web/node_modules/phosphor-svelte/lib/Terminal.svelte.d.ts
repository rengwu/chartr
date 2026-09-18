import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TerminalIcon` instead.
 *
 * @example
 * ```svelte
 * <Terminal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Terminal: Component<IconComponentProps, {}, "">;
type Terminal = ReturnType<typeof Terminal>;
export default Terminal;
