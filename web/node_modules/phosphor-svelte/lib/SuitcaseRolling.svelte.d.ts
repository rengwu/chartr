import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SuitcaseRollingIcon` instead.
 *
 * @example
 * ```svelte
 * <SuitcaseRolling color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SuitcaseRolling: Component<IconComponentProps, {}, "">;
type SuitcaseRolling = ReturnType<typeof SuitcaseRolling>;
export default SuitcaseRolling;
