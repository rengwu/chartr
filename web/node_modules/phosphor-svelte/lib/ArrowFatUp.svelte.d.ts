import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatUp: Component<IconComponentProps, {}, "">;
type ArrowFatUp = ReturnType<typeof ArrowFatUp>;
export default ArrowFatUp;
