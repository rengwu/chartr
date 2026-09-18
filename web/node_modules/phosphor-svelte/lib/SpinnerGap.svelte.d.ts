import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpinnerGapIcon` instead.
 *
 * @example
 * ```svelte
 * <SpinnerGap color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpinnerGap: Component<IconComponentProps, {}, "">;
type SpinnerGap = ReturnType<typeof SpinnerGap>;
export default SpinnerGap;
