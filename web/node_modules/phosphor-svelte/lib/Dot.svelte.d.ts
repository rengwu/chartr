import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotIcon` instead.
 *
 * @example
 * ```svelte
 * <Dot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Dot: Component<IconComponentProps, {}, "">;
type Dot = ReturnType<typeof Dot>;
export default Dot;
