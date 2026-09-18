import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EjectIcon` instead.
 *
 * @example
 * ```svelte
 * <Eject color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Eject: Component<IconComponentProps, {}, "">;
type Eject = ReturnType<typeof Eject>;
export default Eject;
