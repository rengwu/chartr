import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CowboyHatIcon` instead.
 *
 * @example
 * ```svelte
 * <CowboyHat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CowboyHat: Component<IconComponentProps, {}, "">;
type CowboyHat = ReturnType<typeof CowboyHat>;
export default CowboyHat;
