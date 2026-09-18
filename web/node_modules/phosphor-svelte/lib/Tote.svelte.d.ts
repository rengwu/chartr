import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToteIcon` instead.
 *
 * @example
 * ```svelte
 * <Tote color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tote: Component<IconComponentProps, {}, "">;
type Tote = ReturnType<typeof Tote>;
export default Tote;
