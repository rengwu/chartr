import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeskIcon` instead.
 *
 * @example
 * ```svelte
 * <Desk color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Desk: Component<IconComponentProps, {}, "">;
type Desk = ReturnType<typeof Desk>;
export default Desk;
