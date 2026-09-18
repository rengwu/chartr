import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PoliceCarIcon` instead.
 *
 * @example
 * ```svelte
 * <PoliceCar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PoliceCar: Component<IconComponentProps, {}, "">;
type PoliceCar = ReturnType<typeof PoliceCar>;
export default PoliceCar;
