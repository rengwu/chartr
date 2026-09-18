import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SunHorizonIcon` instead.
 *
 * @example
 * ```svelte
 * <SunHorizon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SunHorizon: Component<IconComponentProps, {}, "">;
type SunHorizon = ReturnType<typeof SunHorizon>;
export default SunHorizon;
