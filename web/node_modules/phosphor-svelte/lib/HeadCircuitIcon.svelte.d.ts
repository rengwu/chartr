import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <HeadCircuitIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HeadCircuitIcon: Component<IconComponentProps, {}, "">;
type HeadCircuitIcon = ReturnType<typeof HeadCircuitIcon>;
export default HeadCircuitIcon;
