import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ParallelogramIcon` instead.
 *
 * @example
 * ```svelte
 * <Parallelogram color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Parallelogram: Component<IconComponentProps, {}, "">;
type Parallelogram = ReturnType<typeof Parallelogram>;
export default Parallelogram;
