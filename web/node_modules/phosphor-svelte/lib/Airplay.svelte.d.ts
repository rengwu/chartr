import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirplayIcon` instead.
 *
 * @example
 * ```svelte
 * <Airplay color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Airplay: Component<IconComponentProps, {}, "">;
type Airplay = ReturnType<typeof Airplay>;
export default Airplay;
