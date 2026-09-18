import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BusIcon` instead.
 *
 * @example
 * ```svelte
 * <Bus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bus: Component<IconComponentProps, {}, "">;
type Bus = ReturnType<typeof Bus>;
export default Bus;
