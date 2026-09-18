import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PopsicleIcon` instead.
 *
 * @example
 * ```svelte
 * <Popsicle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Popsicle: Component<IconComponentProps, {}, "">;
type Popsicle = ReturnType<typeof Popsicle>;
export default Popsicle;
