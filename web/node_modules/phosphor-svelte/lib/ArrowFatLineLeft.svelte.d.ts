import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLineLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLineLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLineLeft: Component<IconComponentProps, {}, "">;
type ArrowFatLineLeft = ReturnType<typeof ArrowFatLineLeft>;
export default ArrowFatLineLeft;
