import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BedIcon` instead.
 *
 * @example
 * ```svelte
 * <Bed color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bed: Component<IconComponentProps, {}, "">;
type Bed = ReturnType<typeof Bed>;
export default Bed;
