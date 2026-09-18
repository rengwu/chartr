import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BicycleIcon` instead.
 *
 * @example
 * ```svelte
 * <Bicycle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bicycle: Component<IconComponentProps, {}, "">;
type Bicycle = ReturnType<typeof Bicycle>;
export default Bicycle;
