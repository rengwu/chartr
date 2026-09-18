import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PillIcon` instead.
 *
 * @example
 * ```svelte
 * <Pill color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pill: Component<IconComponentProps, {}, "">;
type Pill = ReturnType<typeof Pill>;
export default Pill;
