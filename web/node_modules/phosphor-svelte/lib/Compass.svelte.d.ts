import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CompassIcon` instead.
 *
 * @example
 * ```svelte
 * <Compass color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Compass: Component<IconComponentProps, {}, "">;
type Compass = ReturnType<typeof Compass>;
export default Compass;
