import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShuffleAngularIcon` instead.
 *
 * @example
 * ```svelte
 * <ShuffleAngular color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShuffleAngular: Component<IconComponentProps, {}, "">;
type ShuffleAngular = ReturnType<typeof ShuffleAngular>;
export default ShuffleAngular;
