import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HairDryerIcon` instead.
 *
 * @example
 * ```svelte
 * <HairDryer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HairDryer: Component<IconComponentProps, {}, "">;
type HairDryer = ReturnType<typeof HairDryer>;
export default HairDryer;
