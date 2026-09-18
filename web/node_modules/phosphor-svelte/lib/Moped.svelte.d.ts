import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MopedIcon` instead.
 *
 * @example
 * ```svelte
 * <Moped color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Moped: Component<IconComponentProps, {}, "">;
type Moped = ReturnType<typeof Moped>;
export default Moped;
