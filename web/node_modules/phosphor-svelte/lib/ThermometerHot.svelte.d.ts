import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThermometerHotIcon` instead.
 *
 * @example
 * ```svelte
 * <ThermometerHot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThermometerHot: Component<IconComponentProps, {}, "">;
type ThermometerHot = ReturnType<typeof ThermometerHot>;
export default ThermometerHot;
