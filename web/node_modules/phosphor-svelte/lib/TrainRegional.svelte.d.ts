import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrainRegionalIcon` instead.
 *
 * @example
 * ```svelte
 * <TrainRegional color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrainRegional: Component<IconComponentProps, {}, "">;
type TrainRegional = ReturnType<typeof TrainRegional>;
export default TrainRegional;
