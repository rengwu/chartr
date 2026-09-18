import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SuitcaseRollingIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SuitcaseRollingIcon: Component<IconComponentProps, {}, "">;
type SuitcaseRollingIcon = ReturnType<typeof SuitcaseRollingIcon>;
export default SuitcaseRollingIcon;
