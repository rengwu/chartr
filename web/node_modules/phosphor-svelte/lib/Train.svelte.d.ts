import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrainIcon` instead.
 *
 * @example
 * ```svelte
 * <Train color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Train: Component<IconComponentProps, {}, "">;
type Train = ReturnType<typeof Train>;
export default Train;
