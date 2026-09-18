import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FireTruckIcon` instead.
 *
 * @example
 * ```svelte
 * <FireTruck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FireTruck: Component<IconComponentProps, {}, "">;
type FireTruck = ReturnType<typeof FireTruck>;
export default FireTruck;
