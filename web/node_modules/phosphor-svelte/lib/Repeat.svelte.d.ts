import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RepeatIcon` instead.
 *
 * @example
 * ```svelte
 * <Repeat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Repeat: Component<IconComponentProps, {}, "">;
type Repeat = ReturnType<typeof Repeat>;
export default Repeat;
