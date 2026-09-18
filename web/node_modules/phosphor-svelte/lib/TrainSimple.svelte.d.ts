import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrainSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <TrainSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrainSimple: Component<IconComponentProps, {}, "">;
type TrainSimple = ReturnType<typeof TrainSimple>;
export default TrainSimple;
